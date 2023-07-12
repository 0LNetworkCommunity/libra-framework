/////////////////////////////////////////////////////////////////////////
// 0L Module
// Proof of Fee
/////////////////////////////////////////////////////////////////////////
// NOTE: this module replaces NodeWeight.move, which becomes redundant since
// all validators have equal weight in consensus.
// TODO: the bubble sort functions were lifted directly from NodeWeight, needs checking.
///////////////////////////////////////////////////////////////////////////


module ol_framework::proof_of_fee {
  use std::error;
  use std::signer;
  use std::vector;
  use std::fixed_point32;
  use aptos_framework::validator_universe;
  use ol_framework::jail;
  use ol_framework::slow_wallet;
  use ol_framework::vouch;
  use aptos_framework::transaction_fee;
  use aptos_framework::reconfiguration;
  use aptos_framework::stake;
  use aptos_framework::system_addresses;

  // use aptos_std::debug::print;

  /// The nominal reward for each validator in each epoch.
  const GENESIS_BASELINE_REWARD: u64 = 1000000;

  //////// ERRORS /////////
  /// Not and active validator
  const ENOT_AN_ACTIVE_VALIDATOR: u64 = 190001;
  /// Bid is above the maximum percentage of the total reward
  const EBID_ABOVE_MAX_PCT: u64 = 190002;
  /// Retracted your bid too many times
  const EABOVE_RETRACT_LIMIT: u64 = 190003; // Potential update

  // A struct on the validators account which indicates their
  // latest bid (and epoch)
  struct ProofOfFeeAuction has key {
    bid: u64,
    epoch_expiration: u64,
    last_epoch_retracted: u64,
    // TODO: show past 5 bids
  }

  struct ConsensusReward has key {
    value: u64,
    clearing_price: u64,
    median_win_bid: u64,
    median_history: vector<u64>,
  }
  public fun init_genesis_baseline_reward(vm: &signer) {
    if (signer::address_of(vm) != @ol_framework) return;

    if (!exists<ConsensusReward>(@ol_framework)) {
      move_to<ConsensusReward>(
        vm,
        ConsensusReward {
          value: GENESIS_BASELINE_REWARD,
          clearing_price: 0,
          median_win_bid: 0,
          median_history: vector::empty<u64>(),
        }
      );
    }
  }

  public fun init(account_sig: &signer) {

    let acc = signer::address_of(account_sig);

    // TODO: V7
    // assert!(validator_universe::is_in_universe(acc), error::permission_denied(ENOT_AN_ACTIVE_VALIDATOR));

    if (!exists<ProofOfFeeAuction>(acc)) {
      move_to<ProofOfFeeAuction>(
      account_sig,
        ProofOfFeeAuction {
          bid: 0,
          epoch_expiration: 0,
          last_epoch_retracted: 0,
        }
      );
    }
  }

  /// consolidates all the logic for the epoch boundary
  // includes getting the sorted bids
  // filling the seats (determined by MusicalChairs), and getting a price.
  // and finally charging the validators for their bid (everyone pays the lowest)
  public fun end_epoch(
    vm: &signer,
    outgoing_compliant_set: &vector<address>,
    n_musical_chairs: u64
  ): vector<address> acquires ProofOfFeeAuction, ConsensusReward {
      system_addresses::assert_ol(vm);

      let sorted_bids = get_sorted_vals(false);
      let (auction_winners, price) = fill_seats_and_get_price(vm, n_musical_chairs, &sorted_bids, outgoing_compliant_set);

      transaction_fee::vm_multi_pay_fee(vm, &auction_winners, price);

      auction_winners
  }





  //////// CONSENSUS CRITICAL ////////
  // Get the validator universe sorted by bid
  // By default this will return a FILTERED list of validators
  // which excludes validators which cannot pass the audit.
  // Leaving the unfiltered option for testing purposes, and any future use.
  // TODO: there's a known issue when many validators have the exact same
  // bid, the preferred node  will be the one LAST included in the validator universe.
  public fun get_sorted_vals(remove_unqualified: bool): vector<address> acquires ProofOfFeeAuction, ConsensusReward {
    let eligible_validators = validator_universe::get_eligible_validators();
    sort_vals_impl(eligible_validators, remove_unqualified)
  }

  fun sort_vals_impl(eligible_validators: vector<address>, remove_unqualified: bool): vector<address> acquires ProofOfFeeAuction, ConsensusReward {
    // let eligible_validators = validator_universe::get_eligible_validators();
    let length = vector::length<address>(&eligible_validators);

    // vector to store each address's node_weight
    let weights = vector::empty<u64>();
    let filtered_vals = vector::empty<address>();
    let k = 0;
    while (k < length) {
      // TODO: Ensure that this address is an active validator

      let cur_address = *vector::borrow<address>(&eligible_validators, k);
      let (bid, _expire) = current_bid(cur_address);
      if (remove_unqualified && !audit_qualification(&cur_address)) {
        k = k + 1;
        continue
      };
      vector::push_back<u64>(&mut weights, bid);
      vector::push_back<address>(&mut filtered_vals, cur_address);
      k = k + 1;
    };

    // Sorting the accounts vector based on value (weights).
    // Bubble sort algorithm
    let len_filtered = vector::length<address>(&filtered_vals);
    if (len_filtered < 2) return filtered_vals;
    let i = 0;
    while (i < len_filtered){
      let j = 0;
      while(j < len_filtered-i-1){

        let value_j = *(vector::borrow<u64>(&weights, j));

        let value_jp1 = *(vector::borrow<u64>(&weights, j+1));
        if(value_j > value_jp1){

          vector::swap<u64>(&mut weights, j, j+1);

          vector::swap<address>(&mut filtered_vals, j, j+1);
        };
        j = j + 1;

      };
      i = i + 1;

    };


    // Reverse to have sorted order - high to low.
    vector::reverse<address>(&mut filtered_vals);

    return filtered_vals
  }

  // Here we place the bidders into their seats.
  // The order of the bids will determine placement.
  // One important aspect of picking the next validator set:
  // it should have 2/3rds of known good ("proven") validators
  // from the previous epoch. Otherwise the unproven nodes, who
  // may not be ready for consensus, might be offline and cause a halt.
  // Validators can be inattentive and have bids that qualify, but their nodes
  // are not ready.
  // So the selection algorithm needs to stop filling seats with "unproven"
  // validators if the max unproven nodes limit is hit (1/3).

  // The paper does not specify what happens with the "Jail reputation"
  // of a validator. E.g. if a validator has a bid with no expiry
  // but has a bad jail reputation does this penalize in the ordering?
  // This is a potential issue again with inattentive validators who
  // have have a high bid, but again they fail repeatedly to finalize an epoch
  // successfully. Their bids should not penalize validators who don't have
  // a streak of jailed epochs. So of the 1/3 unproven nodes, we'll first seat the validators with Jail.consecutive_failure_to_rejoin < 2, and after that the remainder.

  // There's some code implemented which is not enabled in the current form.
  // Unsealed auctions are tricky. The Proof Of Fee
  // paper states that since the bids are not private, we need some
  // constraint to minimize shill bids, "bid shading" or other strategies
  // which allow validators to drift from their private valuation.
  // As such per epoch the validator is only allowed to revise their bids /
  // down once. To do this in practice they need to retract a bid (sit out
  // the auction), and then place a new bid.
  // A validator can always leave the auction, but if they rejoin a second time in the epoch, then they've committed a bid until the next epoch.
  // So retracting should be done with care.  The ergonomics are not great.
  // The preference would be not to have this constraint if on the margins
  // the ergonomics brings more bidders than attackers.
  // After more experience in the wild, the network may decide to
  // limit bid retracting.


  // The Validator must qualify on a number of metrics: have funds in their Unlocked account to cover bid, have miniumum viable vouches, and not have been jailed in the previous round.


  // This function assumes we have already filtered out ineligible validators.
  // but we will check again here.
  public fun fill_seats_and_get_price(
    vm: &signer,
    set_size: u64,
    sorted_vals_by_bid: &vector<address>,
    proven_nodes: &vector<address>
  ): (vector<address>, u64) acquires ProofOfFeeAuction, ConsensusReward {
    if (signer::address_of(vm) != @ol_framework) return (vector::empty<address>(), 0);

    let seats_to_fill = vector::empty<address>();

    // check the max size of the validator set.
    // there may be too few "proven" validators to fill the set with 2/3rds proven nodes of the stated set_size.
    let proven_len = vector::length(proven_nodes);

    // check if the proven len plus unproven quota will
    // be greater than the set size. Which is the expected.
    // Otherwise the set will need to be smaller than the
    // declared size, because we will have to fill with more unproven nodes.
    let one_third_of_max = proven_len/2;
    let safe_set_size = proven_len + one_third_of_max;

    let (set_size, max_unproven) = if (safe_set_size < set_size) {
      (safe_set_size, safe_set_size/3)
    } else {
      // happy case, unproven bidders are a smaller minority
      (set_size, set_size/3)
    };

    // Now we can seat the validators based on the algo above:
    // 1. seat the proven nodes of previous epoch
    // 2. seat validators who did not participate in the previous epoch:
    // 2a. seat the vals with jail reputation < 2
    // 2b. seat the remainder of the unproven vals with any jail reputation.

    let num_unproven_added = 0;
    let i = 0u64;
    while (
      (vector::length(&seats_to_fill) < set_size) &&
      (i < vector::length(sorted_vals_by_bid))
    ) {
      let val = vector::borrow(sorted_vals_by_bid, i);
      // check if a proven node
      if (vector::contains(proven_nodes, val)) {
        vector::push_back(&mut seats_to_fill, *val);
      } else {
        // for unproven nodes, push it to list if we haven't hit limit
        if (num_unproven_added < max_unproven ) {
          // TODO: check jail reputation
          vector::push_back(&mut seats_to_fill, *val);
          num_unproven_added = num_unproven_added + 1;
        };
      };
      i = i + 1;
    };

    // Set history
    set_history(vm, &seats_to_fill);

    // we failed to seat anyone.
    // let EpochBoundary deal with this.
    if (vector::is_empty(&seats_to_fill)) {


      return (seats_to_fill, 0)
    };

    // Find the clearing price which all validators will pay
    let lowest_bidder = vector::borrow(&seats_to_fill, vector::length(&seats_to_fill) - 1);

    let (lowest_bid_pct, _) = current_bid(*lowest_bidder);



    // update the clearing price
    let cr = borrow_global_mut<ConsensusReward>(@ol_framework);
    cr.clearing_price = lowest_bid_pct;

    return (seats_to_fill, lowest_bid_pct)
  }

  // consolidate all the checks for a validator to be seated
  public fun audit_qualification(val: &address): bool acquires ProofOfFeeAuction, ConsensusReward {

      // print(&1001);
      // Safety check: node has valid configs
      if (!stake::stake_pool_exists(*val)) return false;
// print(&1002);
      // is a slow wallet
      if (!slow_wallet::is_slow(*val)) return false;
// print(&1003);
      // we can't seat validators that were just jailed
      // NOTE: epoch reconfigure needs to reset the jail
      // before calling the proof of fee.
      if (jail::is_jailed(*val)) return false;
// print(&1004);
      // we can't seat validators who don't have minimum viable vouches

      if (!vouch::unrelated_buddies_above_thresh(*val)) return false;
      let (bid_pct, expire) = current_bid(*val);
// print(&1005);
      if (bid_pct < 1) return false;
// print(&1006);
      // Skip if the bid expired. belt and suspenders, this should have been checked in the sorting above.
      // TODO: make this it's own function so it can be publicly callable, it's useful generally, and for debugging.



      if (reconfiguration::get_current_epoch() > expire) return false;
      // skip the user if they don't have sufficient UNLOCKED funds
      // or if the bid expired.
// print(&1007);
      let unlocked_coins = slow_wallet::unlocked_amount(*val);
      let (baseline_reward, _, _) = get_consensus_reward();
      let coin_required = fixed_point32::multiply_u64(baseline_reward, fixed_point32::create_from_rational(bid_pct, 1000));
// print(&1008);
      if (unlocked_coins < coin_required) return false;
// print(&1009);
      // friend of ours
      true
  }
  // Adjust the reward at the end of the epoch
  // as described in the paper, the epoch reward needs to be adjustable
  // given that the implicit bond needs to be sufficient, eg 5-10x the reward.
  public fun reward_thermostat(vm: &signer) acquires ConsensusReward {
    if (signer::address_of(vm) != @ol_framework) {
      return
    };
    // check the bid history
    // if there are 5 days above 95% adjust the reward up by 5%
    // adjust by more if it has been 10 days then, 10%
    // if there are 5 days below 50% adjust the reward down.
    // adjust by more if it has been 10 days then 10%

    let bid_upper_bound = 0950;
    let bid_lower_bound = 0500;

    let short_window: u64 = 5;
    let long_window: u64 = 10;

    let cr = borrow_global_mut<ConsensusReward>(@ol_framework);


    let len = vector::length<u64>(&cr.median_history);
    let i = 0;

    let epochs_above = 0;
    let epochs_below = 0;
    while (i < 16 && i < len) { // max ten days, but may have less in history, filling set should truncate the history at 15 epochs.

      let avg_bid = *vector::borrow<u64>(&cr.median_history, i);

      if (avg_bid > bid_upper_bound) {
        epochs_above = epochs_above + 1;
      } else if (avg_bid < bid_lower_bound) {
        epochs_below = epochs_below + 1;
      };

      i = i + 1;
    };


    if (cr.value > 0) {


      // TODO: this is an initial implementation, we need to
      // decide if we want more granularity in the reward adjustment
      // Note: making this readable for now, but we can optimize later
      if (epochs_above > epochs_below) {

        // if (epochs_above > short_window) {

        // check for zeros.
        // TODO: put a better safety check here

        // If the Validators are bidding near 100% that means
        // the reward is very generous, i.e. their opportunity
        // cost is met at small percentages. This means the
        // implicit bond is very high on validators. E.g.
        // at 1% median bid, the implicit bond is 100x the reward.
        // We need to DECREASE the reward


        if (epochs_above > long_window) {

          // decrease the reward by 10%

          cr.value = cr.value - (cr.value / 10);
          return // return early since we can't increase and decrease simultaneously
        } else if (epochs_above > short_window) {
          // decrease the reward by 5%

          cr.value = cr.value - (cr.value / 20);


          return // return early since we can't increase and decrease simultaneously
        }
      };


        // if validators are bidding low percentages
        // it means the nominal reward is not high enough.
        // That is the validator's opportunity cost is not met within a
        // range where the bond is meaningful.
        // For example: if the bids for the epoch's reward is 50% of the  value, that means the potential profit, is the same as the potential loss.
        // At a 25% bid (potential loss), the profit is thus 75% of the value, which means the implicit bond is 25/75, or 1/3 of the bond, the risk favors the validator. This means among other things, that an attacker can pay for the cost of the attack with the profits. See paper, for more details.

        // we need to INCREASE the reward, so that the bond is more meaningful.


        if (epochs_below > long_window) {


          // increase the reward by 10%
          cr.value = cr.value + (cr.value / 10);
        } else if (epochs_below > short_window) {


          // increase the reward by 5%
          cr.value = cr.value + (cr.value / 20);
        };
      // };
    };
  }

  /// find the median bid to push to history
  // this is needed for reward_thermostat
  public fun set_history(vm: &signer, seats_to_fill: &vector<address>) acquires ProofOfFeeAuction, ConsensusReward {
    if (signer::address_of(vm) != @ol_framework) {
      return
    };


    let median_bid = get_median(seats_to_fill);
    // push to history
    let cr = borrow_global_mut<ConsensusReward>(@ol_framework);
    cr.median_win_bid = median_bid;
    if (vector::length(&cr.median_history) < 10) {

      vector::push_back(&mut cr.median_history, median_bid);
    } else {

      vector::remove(&mut cr.median_history, 0);
      vector::push_back(&mut cr.median_history, median_bid);
    };
  }

  fun get_median(seats_to_fill: &vector<address>):u64 acquires ProofOfFeeAuction {
    // TODO: the list is sorted above, so
    // we assume the median is the middle element
    let len = vector::length(seats_to_fill);
    if (len == 0) {
      return 0
    };
    let median_bidder = if (len > 2) {
      vector::borrow(seats_to_fill, len/2)
    } else {
      vector::borrow(seats_to_fill, 0)
    };
    let (median_bid, _) = current_bid(*median_bidder);
    return median_bid
  }

  //////////////// GETTERS ////////////////
  // get the current bid for a validator


  /// get the baseline reward from ConsensusReward
  /// returns (reward, clearing_price, median_win_bid)
  public fun get_consensus_reward(): (u64, u64, u64) acquires ConsensusReward {
    let b = borrow_global<ConsensusReward>(@ol_framework);
    return (b.value, b.clearing_price, b.median_win_bid)
  }

  // CONSENSUS CRITICAL
  // ALL EYES ON THIS
  // Proof of Fee returns the current bid of the validator during the auction for upcoming epoch seats.
  // returns (current bid, expiration epoch)
  public fun current_bid(node_addr: address): (u64, u64) acquires ProofOfFeeAuction {
    if (exists<ProofOfFeeAuction>(node_addr)) {
      let pof = borrow_global<ProofOfFeeAuction>(node_addr);
      let e = reconfiguration::get_current_epoch();
      // check the expiration of the bid
      // the bid is zero if it expires.
      // The expiration epoch number is inclusive of the epoch.
      // i.e. the bid expires on e + 1.
      if (pof.epoch_expiration >= e || pof.epoch_expiration == 0) {
        return (pof.bid, pof.epoch_expiration)
      };
      return (0, pof.epoch_expiration)
    };
    return (0, 0)
  }

  // which epoch did they last retract a bid?
  public fun is_already_retracted(node_addr: address): (bool, u64) acquires ProofOfFeeAuction {
    if (exists<ProofOfFeeAuction>(node_addr)) {
      let when_retract = *&borrow_global<ProofOfFeeAuction>(node_addr).last_epoch_retracted;
      return (reconfiguration::get_current_epoch() >= when_retract,  when_retract)
    };
    return (false, 0)
  }

  // Get the top N validators by bid, this is FILTERED by default
  public fun top_n_accounts(account: &signer, n: u64, unfiltered: bool): vector<address> acquires ProofOfFeeAuction, ConsensusReward {
      system_addresses::assert_vm(account);

      let eligible_validators = get_sorted_vals(unfiltered);
      let len = vector::length<address>(&eligible_validators);
      if(len <= n) return eligible_validators;

      let diff = len - n;
      while(diff > 0){
        vector::pop_back(&mut eligible_validators);
        diff = diff - 1;
      };

      eligible_validators
  }


  ////////// SETTERS //////////
  // validator can set a bid. See transaction script below.
  // the validator can set an "expiry epoch:  for the bid.
  // Zero means never expires.
  // Bids are denomiated in percentages, with ONE decimal place..
  // i.e. 0123 = 12.3%
  // Provisionally 110% is the maximum bid. Which could be reviewed.
  public fun set_bid(account_sig: &signer, bid: u64, expiry_epoch: u64) acquires ProofOfFeeAuction {

    let acc = signer::address_of(account_sig);
    if (!exists<ProofOfFeeAuction>(acc)) {
      init(account_sig);
    };

    // bid must be below 110%
    assert!(bid <= 1100, error::out_of_range(EBID_ABOVE_MAX_PCT));

    let pof = borrow_global_mut<ProofOfFeeAuction>(acc);
    pof.epoch_expiration = expiry_epoch;
    pof.bid = bid;
  }


  /// Note that the validator will not be bidding on any future
  /// epochs if they retract their bid. The must set a new bid.
  public fun retract_bid(account_sig: &signer) acquires ProofOfFeeAuction {

    let acc = signer::address_of(account_sig);
    if (!exists<ProofOfFeeAuction>(acc)) {
      init(account_sig);
    };


    let pof = borrow_global_mut<ProofOfFeeAuction>(acc);
    let this_epoch = reconfiguration::get_current_epoch();

    //////// LEAVE COMMENTED. Code for a potential upgrade. ////////
    // See above discussion for retracting of bids.
    //
    // already retracted this epoch
    // assert!(this_epoch > pof.last_epoch_retracted, error::ol_tx(EABOVE_RETRACT_LIMIT));
    //////// LEAVE COMMENTED. Code for a potential upgrade. ////////


    pof.epoch_expiration = 0;
    pof.bid = 0;
    pof.last_epoch_retracted = this_epoch;
  }

  ////////// TRANSACTION APIS //////////
  //. manually init the struct, fallback in case of migration fail
  public entry fun init_bidding(sender: signer) {
    init(&sender);
  }

  /// update the bid for the sender
  public entry fun pof_update_bid(sender: signer, bid: u64, epoch_expiry: u64) acquires ProofOfFeeAuction {
    // update the bid, initializes if not already.
    set_bid(&sender, bid, epoch_expiry);
  }

  /// retract bid
  public entry fun pof_retract_bid(sender: signer) acquires ProofOfFeeAuction {
    // retract a bid
    retract_bid(&sender);
  }

  //////// TEST HELPERS ////////
  #[test_only]
  use ol_framework::testnet;

  #[test_only]
  public fun test_set_val_bids(vm: &signer, vals: &vector<address>, bids: &vector<u64>, expiry: &vector<u64>) acquires ProofOfFeeAuction {
    testnet::assert_testnet(vm);

    let len = vector::length(vals);
    let i = 0;
    while (i < len) {
      let bid = vector::borrow(bids, i);
      let exp = vector::borrow(expiry, i);
      let addr = vector::borrow(vals, i);
      test_set_one_bid(vm, addr, *bid, *exp);
      i = i + 1;
    };
  }

  #[test_only]
  public fun test_set_one_bid(vm: &signer, val: &address, bid:  u64, exp: u64) acquires ProofOfFeeAuction {
    testnet::assert_testnet(vm);
    let pof = borrow_global_mut<ProofOfFeeAuction>(*val);
    pof.epoch_expiration = exp;
    pof.bid = bid;
  }

  #[test_only]
  public fun test_mock_reward(
    vm: &signer,
    value: u64,
    clearing_price: u64,
    median_win_bid: u64,
    median_history: vector<u64>,
  ) acquires ConsensusReward {
    testnet::assert_testnet(vm);

    let cr = borrow_global_mut<ConsensusReward>(@ol_framework );
    cr.value = value;
    cr.clearing_price = clearing_price;
    cr.median_win_bid = median_win_bid;
    cr.median_history = median_history;

  }

  #[test(vm = @ol_framework)]
  fun meta_mock_reward(vm: signer) acquires ConsensusReward {
    use aptos_framework::chain_id;

    init_genesis_baseline_reward(&vm);

    chain_id::initialize_for_test(&vm, 4);

    test_mock_reward(
      &vm,
      100,
      50,
      33,
      vector::singleton(33),
    );

    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

  }

  #[test(vm = @ol_framework)]
  public entry fun thermostat_unit_happy(vm: signer)  acquires ConsensusReward {
    use aptos_framework::chain_id;
    // use ol_framework::mock;

    init_genesis_baseline_reward(&vm);

    chain_id::initialize_for_test(&vm, 4);

    let start_value = 0510; // 51% of baseline reward
    let median_history = vector::empty<u64>();

    let i = 0;
    while (i < 10) {
      let factor = i * 10;
      let value = start_value + factor;

      vector::push_back(&mut median_history, value);
      i = i + 1;
    };


    test_mock_reward(
      &vm,
      100,
      50,
      33,
      median_history,
    );

    // no changes until we run the thermostat.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

    reward_thermostat(&vm);

    // This is the happy case. No changes since the rewards were within range
    // the whole time.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

  }

  // Scenario: The reward is too low during 5 days (short window). People are not bidding very high.
  #[test(vm = @ol_framework)]
  fun thermostat_increase_short(vm: signer) acquires ConsensusReward {
    use aptos_framework::chain_id;

    init_genesis_baseline_reward(&vm);
    chain_id::initialize_for_test(&vm, 4);

    let start_value = 0200; // 20% of baseline fee.
    let median_history = vector::empty<u64>();

    // we need between 5 and 10 epochs to be a short "window"
    let i = 0;
    while (i < 7) {
      vector::push_back(&mut median_history, start_value);
      i = i + 1;
    };


    test_mock_reward(
      &vm,
      100,
      50,
      33,
      median_history,
    );

    // no changes until we run the thermostat.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 105, 1003);
    assert!(clearing == 50, 1004);
    assert!(median == 33, 1005);

  }


  // Scenario: The reward is too low during 5 days (short window). People are not bidding very high.
  #[test(vm = @ol_framework)]
  fun thermostat_increase_long(vm: signer) acquires ConsensusReward {
    use aptos_framework::chain_id;

    init_genesis_baseline_reward(&vm);
    chain_id::initialize_for_test(&vm, 4);

    let start_value = 0200; // 20% of baseline fee.
    let median_history = vector::empty<u64>();

    // we need at least 10 epochs above the 95% range to be a "long window"
    let i = 0;
    while (i < 12) {
      // let factor = i * 10;
      // let value = start_value + factor;

      vector::push_back(&mut median_history, start_value);
      i = i + 1;
    };


    test_mock_reward(
      &vm,
      100,
      50,
      33,
      median_history,
    );

    // no changes until we run the thermostat.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 110, 1003);
    assert!(clearing == 50, 1004);
    assert!(median == 33, 1005);

  }


  // Scenario: The reward is too high during 5 days (short window). People are bidding over 95% of the baseline fee.
  #[test(vm = @ol_framework)]
  fun thermostat_decrease_short(vm: signer) acquires ConsensusReward {
    use aptos_framework::chain_id;

    init_genesis_baseline_reward(&vm);
    chain_id::initialize_for_test(&vm, 4);

    let start_value = 0950; // 96% of baseline fee.
    let median_history = vector::empty<u64>();

    // we need between 5 and 10 epochs to be a short "window"
    let i = 0;
    while (i < 7) {
      let factor = i * 10;
      let value = start_value + factor;
      vector::push_back(&mut median_history, value);
      i = i + 1;
    };


    test_mock_reward(
      &vm,
      100,
      50,
      33,
      median_history,
    );

    // no changes until we run the thermostat.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 95, 1000);
    assert!(clearing == 50, 1004);
    assert!(median == 33, 1005);

  }

    // Scenario: The reward is too low during 5 days (short window). People are not bidding very high.
  #[test(vm = @ol_framework)]
  fun thermostat_decrease_long(vm: signer) acquires ConsensusReward {
    use aptos_framework::chain_id;

    init_genesis_baseline_reward(&vm);
    chain_id::initialize_for_test(&vm, 4);

    let start_value = 0960; // 96% of baseline fee.
    let median_history = vector::empty<u64>();

    // we need at least 10 epochs above the 95% range to be a "long window"
    let i = 0;
    while (i < 12) {
      // let factor = i * 10;
      // let value = start_value + factor;

      vector::push_back(&mut median_history, start_value);
      i = i + 1;
    };


    test_mock_reward(
      &vm,
      100,
      50,
      33,
      median_history,
    );

    // no changes until we run the thermostat.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 100, 1000);
    assert!(clearing == 50, 1001);
    assert!(median == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let (value, clearing, median) = get_consensus_reward();
    assert!(value == 90, 1003);
    assert!(clearing == 50, 1004);
    assert!(median == 33, 1005);

  }

  // #[test(vm = @ol_framework)]
  // fun pof_set_retract(vm: signer) {
  //     use aptos_framework::account;

  //     validator_universe::initialize(&vm);

  //     let sig = account::create_signer_for_test(@0x123);
  //     let (_sk, pk, pop) = stake::generate_identity();
  //     stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);

  //     validator_universe::is_in_universe(@0x123);

  // }
}
