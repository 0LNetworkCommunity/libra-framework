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
  use diem_framework::validator_universe;
  use ol_framework::jail;
  use ol_framework::slow_wallet;
  use ol_framework::vouch;
  use diem_framework::transaction_fee;
  use ol_framework::epoch_helper;
  use diem_framework::stake;
  use diem_framework::system_addresses;
  use ol_framework::globals;
  // use diem_std::debug::print;

  friend ol_framework::epoch_boundary;

  /// The nominal reward for each validator in each epoch.
  const GENESIS_BASELINE_REWARD: u64 = 1000000;

  //////// ERRORS /////////
  /// Not an active validator
  const ENOT_AN_ACTIVE_VALIDATOR: u64 = 1;
  /// Bid is above the maximum percentage of the total reward
  const EBID_ABOVE_MAX_PCT: u64 = 2;
  /// Retracted your bid too many times
  const EABOVE_RETRACT_LIMIT: u64 = 3; // Potential update
  /// validator is not configured
  const EVALIDATOR_NOT_CONFIGURED: u64 = 11;
  /// not a slow wallet
  const EWALLET_NOT_SLOW: u64 = 12;
  /// validator is jailed
  const EIS_JAILED: u64 = 13;
  /// no enough vouches
  const ETOO_FEW_VOUCHES: u64 = 14;
  /// bid is zero
  const EBID_IS_ZERO: u64 = 15;
  /// bid has expired
  const EBID_EXPIRED: u64 = 16;
  /// not enough coin balance
  const ELOW_UNLOCKED_COIN_BALANCE: u64 = 17;


  // A struct on the validators account which indicates their
  // latest bid (and epoch)
  struct ProofOfFeeAuction has key {
    bid: u64,
    epoch_expiration: u64,
    last_epoch_retracted: u64,
    // TODO: show past 5 bids
  }

  struct ConsensusReward has key {
    nominal_reward: u64,
    net_reward: u64,
    entry_fee: u64,
    clearing_bid: u64,
    median_win_bid: u64,
    median_history: vector<u64>,
  }
  public fun init_genesis_baseline_reward(vm: &signer) {
    system_addresses::assert_ol(vm);

    if (!exists<ConsensusReward>(@ol_framework)) {
      move_to<ConsensusReward>(
        vm,
        ConsensusReward {
          nominal_reward: GENESIS_BASELINE_REWARD,
          net_reward: GENESIS_BASELINE_REWARD,
          entry_fee: 0,
          clearing_bid: 0,
          median_win_bid: 0,
          median_history: vector::empty<u64>(),
        }
      );
    }
  }

  // on a migration genesis for mainnet the genesis reward needs to be calculated
  // from supply data.
  public fun genesis_migrate_reward(vm: &signer, nominal_reward: u64) acquires
  ConsensusReward {
    system_addresses::assert_ol(vm); // either 0x1 or 0x0

    let state = borrow_global_mut<ConsensusReward>(@ol_framework);
    state.nominal_reward = nominal_reward;
    state.net_reward = nominal_reward; // just for info purposes. It gets calculated
    // on first epoch change.
  }

  public fun init(account_sig: &signer) {

    let acc = signer::address_of(account_sig);

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

  /// Consolidates all the logic for the epoch boundary/
  /// Includes: getting the sorted bids,
  /// filling the seats (determined by MusicalChairs), and getting a price.
  /// and finally charging the validators for their bid (everyone pays the lowest)
  /// for audit instrumentation returns: final set size, auction winners, all the bidders, (including not-qualified), and all qualified bidders.
  /// we also return the auction entry price (clearing price)
  /// (final_set_size, auction_winners, all_bidders, only_qualified_bidders, actually_paid, entry_fee)
  public(friend) fun end_epoch(
    vm: &signer,
    outgoing_compliant_set: &vector<address>,
    final_set_size: u64
  ): (vector<address>, vector<address>, vector<address>, u64) acquires ProofOfFeeAuction, ConsensusReward {
      system_addresses::assert_ol(vm);

      let all_bidders = get_bidders(false);
      let only_qualified_bidders = get_bidders(true);
      // The set size as determined by musical chairs is a target size
      // but the actual final size depends on how much can we expand the set
      // without adding too many unproven nodes (which we don't know if they are prepared to validate, and risk halting th network).

      // This is the core of the mechanism, the uniform price auction
      // the winners of the auction will be the validator set.
      // other lists are created for audit purposes of the BoundaryStatus
      let (auction_winners, entry_fee, _clearing_bid, _proven, _unproven) = fill_seats_and_get_price(vm, final_set_size, &only_qualified_bidders, outgoing_compliant_set);


      (auction_winners, all_bidders, only_qualified_bidders, entry_fee)
  }

  /// The fees are charged seperate from the auction and seating loop
  /// this is because there are edge conditions in which the winners
  /// may not be the ones seated (once we consider failover rules).
  /// returns the expected amount of fees, the fees that were actually able to be withdrawn, and success, if the expected and withdrawn are the same.
  public(friend) fun charge_epoch_fees(vm: &signer, auction_winners: vector<address>, price: u64): (u64, u64, bool) {
      let expected_fees = vector::length(&auction_winners) * price;

      let actually_paid = transaction_fee::vm_multi_collect(vm, &auction_winners, price);

      let fee_success = actually_paid == expected_fees;
      (expected_fees, actually_paid, fee_success)
  }





  //////// CONSENSUS CRITICAL ////////
  // Get the validator universe sorted by bid
  // By default this will return a FILTERED list of validators
  // which excludes validators which cannot pass the audit.
  // Leaving the unfiltered option for testing purposes, and any future use.
  // The function returns the ordered bidders and their bids
  // TODO: there's a known issue when many validators have the exact same
  // bid, the preferred node  will be the one LAST included in the validator universe.


  #[view]
  public fun get_bidders(remove_unqualified: bool): vector<address> acquires ProofOfFeeAuction, ConsensusReward {
    let eligible_validators = validator_universe::get_eligible_validators();
    let (bidders, _) = sort_vals_impl(&eligible_validators, remove_unqualified);
    bidders
  }

  #[view]
  // same as get bidders, but returns the bid
  public fun get_bidders_and_bids(remove_unqualified: bool): (vector<address>, vector<u64>) acquires ProofOfFeeAuction, ConsensusReward {
    let eligible_validators = validator_universe::get_eligible_validators();
    sort_vals_impl(&eligible_validators, remove_unqualified)
  }
  // returns two lists: ordered bidder addresss and the list of bids bid
  fun sort_vals_impl(eligible_validators: &vector<address>, remove_unqualified: bool): (vector<address>, vector<u64>) acquires ProofOfFeeAuction, ConsensusReward {
    // let eligible_validators = validator_universe::get_eligible_validators();
    let length = vector::length<address>(eligible_validators);

    // vector to store each address's node_weight
    let bids = vector::empty<u64>();
    let filtered_vals = vector::empty<address>();
    let k = 0;
    while (k < length) {
      // TODO: Ensure that this address is an active validator

      let cur_address = *vector::borrow<address>(eligible_validators, k);
      let (bid, _expire) = current_bid(cur_address);

      let (_, qualified) = audit_qualification(cur_address);
      if (remove_unqualified && !qualified) {
        k = k + 1;
        continue
      };
      vector::push_back<u64>(&mut bids, bid);
      vector::push_back<address>(&mut filtered_vals, cur_address);
      k = k + 1;
    };

    // Sorting the accounts vector based on value (weights).
    // Bubble sort algorithm
    let len_filtered = vector::length<address>(&filtered_vals);
    // if there's only one person (testing)
    if (len_filtered < 2) return (filtered_vals, bids);
    let i = 0;
    while (i < len_filtered){
      let j = 0;
      while(j < len_filtered-i-1){

        let value_j = *(vector::borrow<u64>(&bids, j));

        let value_jp1 = *(vector::borrow<u64>(&bids, j+1));
        if(value_j > value_jp1){

          vector::swap<u64>(&mut bids, j, j+1);

          vector::swap<address>(&mut filtered_vals, j, j+1);
        };
        j = j + 1;

      };
      i = i + 1;

    };


    // Reverse to have sorted order - high to low.
    vector::reverse(&mut filtered_vals);
    vector::reverse(&mut bids);

    return (filtered_vals, bids)
  }

  #[view]
  /// checks if the validator has enough vouchers in the current set (prior to entry)
  /// @params incoming address to be evaluated
  /// @returns (above_threshold, count_validators) if
  public fun get_valid_vouchers_in_set(incoming_addr: address): (bool, u64) {
      let val_set = stake::get_current_validators();
      let (frens_in_val_set, _found) = vouch::true_friends_in_list(incoming_addr, &val_set);
      let threshold = globals::get_validator_vouch_threshold();
      let count_in_set = vector::length(&frens_in_val_set);

      (count_in_set >= threshold, count_in_set)
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

  /// Showtime.
  /// This is where we take all the bidders and seat them.
  /// We also need to check here for the safe size of the validator set.
  /// This function assumes we have already filtered out ineligible validators.
  /// but we will check again here.
  /// we return:
  // a. the list of winning validators (the validator set)
  // b. the entry fee paid
  // c. the clearing bid (percentage paid)
  // d. the list of proven nodes added, for audit and instrumentation
  // e. the list of unproven, for audit and instrumentation
  public fun fill_seats_and_get_price(
    vm: &signer,
    final_set_size: u64,
    sorted_vals_by_bid: &vector<address>,
    proven_nodes: &vector<address>
  ): (vector<address>, u64, u64, vector<address>, vector<address>) acquires ProofOfFeeAuction, ConsensusReward {
    system_addresses::assert_ol(vm);

    // NOTE: this is duplicate work, but we are double checking we are getting a proper sort.
    let (sorted_vals_by_bid, _) = sort_vals_impl(sorted_vals_by_bid, true);

    // Now we can seat the validators based on the algo:
    // A. seat the highest bidding 2/3 proven nodes of previous epoch
    // B. seat the remainder 1/3 of highest bidding validators which may or MA NOT have participated in the previous epoch. Note: We assume jailed validators are not in qualified bidder list anyways, but we should check again
    // The way to achieve this with minimal looping, is by going through the list and adding every bidder, but once the quota of unproven nodes is full, only proven nodes can be added.

    // TODO: include jail reputation
    // B1. first, seat any vals with jail reputation < 2.
    // B2. then, if there are still seats, seat the remainder of the unproven vals with any jail reputation.
    let unproven_quota = final_set_size / 3;
    let proposed_validators = vector::empty<address>();

    let audit_add_proven_vals = vector::empty<address>();
    let audit_add_unproven_vals = vector::empty<address>();

    let num_unproven_added = 0;
    let i = 0u64;
    while (
      (vector::length(&proposed_validators) < final_set_size) && // until seats full
      (i < vector::length(&sorted_vals_by_bid))
    ) {
      let val = vector::borrow(&sorted_vals_by_bid, i);
      // check if a proven node
      // NOTE: if the top bidders all all "proven" nodes, then there will
      // be no reason to add an unproven. Unproven nodes will only
      // be picked if they have bids higher than the bottom 1/3 bids of the proven nodes
      if (vector::contains(proven_nodes, val)) {
        vector::push_back(&mut proposed_validators, *val);
        vector::push_back(&mut audit_add_proven_vals, *val);
      } else {
        // for unproven nodes, push it to list if we haven't hit limit
        if (num_unproven_added < unproven_quota ) {
          // TODO: check jail reputation
          vector::push_back(&mut proposed_validators, *val);
          vector::push_back(&mut audit_add_unproven_vals, *val);

          num_unproven_added = num_unproven_added + 1;
        };
      };
      i = i + 1;
    };

    // Save history
    set_history(vm, &proposed_validators);

    // We failed to seat anyone.
    // let epoch_boundary.move deal with this.
    if (vector::is_empty(&proposed_validators)) return (proposed_validators, 0, 0, audit_add_proven_vals, audit_add_unproven_vals);

    // Find the clearing price which all validators will pay
    let lowest_bidder = vector::borrow(&proposed_validators, vector::length(&proposed_validators) - 1);

    let (lowest_bid_pct, _) = current_bid(*lowest_bidder);

    // update the clearing price
    let cr = borrow_global_mut<ConsensusReward>(@ol_framework);
    cr.clearing_bid = lowest_bid_pct;

    if (lowest_bid_pct > 0) {
      cr.entry_fee = fixed_point32::multiply_u64(cr.nominal_reward, bid_as_fixedpoint(lowest_bid_pct));

      if (cr.nominal_reward > cr.entry_fee)  {
        cr.net_reward = cr.nominal_reward - cr.entry_fee;
      } else {
        // shoudn't be reachable, but here for completion
        cr.net_reward = cr.nominal_reward
      };
    } else {
      cr.entry_fee = 0;
      cr.net_reward = cr.nominal_reward;
    };


    return (proposed_validators, cr.entry_fee, cr.clearing_bid, audit_add_proven_vals, audit_add_unproven_vals)
  }

  #[view]
  /// consolidate all the checks for a validator to be seated
  public fun audit_qualification(val: address): (vector<u64>, bool) acquires ProofOfFeeAuction, ConsensusReward {

      let errors = vector::empty<u64>();
      // Safety check: node has valid configs
      if (!stake::stake_pool_exists(val)) vector::push_back(&mut errors, EVALIDATOR_NOT_CONFIGURED); // 11
      // is a slow wallet
      if (!slow_wallet::is_slow(val)) vector::push_back(&mut errors, EWALLET_NOT_SLOW); // 12
      // we can't seat validators that were just jailed
      // NOTE: epoch reconfigure needs to reset the jail
      // before calling the proof of fee.
      if (jail::is_jailed(val)) vector::push_back(&mut errors, EIS_JAILED); //13

      // we can't seat validators who don't have minimum viable vouches
      let (is_above_thresh, _count) = get_valid_vouchers_in_set(val);
      if (!is_above_thresh) vector::push_back(&mut errors, ETOO_FEW_VOUCHES); // 14

      // check if current BIDS are valid
      let (bid_pct, expire) = current_bid(val);
      if (bid_pct == 0) vector::push_back(&mut errors, EBID_IS_ZERO); // 15
      // Skip if the bid expired. belt and suspenders, this should have been checked in the sorting above.
      // TODO: make this it's own function so it can be publicly callable, it's useful generally, and for debugging.

      if (epoch_helper::get_current_epoch() > expire) vector::push_back(&mut errors, EBID_EXPIRED); // 16
      // skip the user if they don't have sufficient UNLOCKED funds
      // or if the bid expired.
      let unlocked_coins = slow_wallet::unlocked_amount(val);
      let (_, entry_fee,  _, _) = get_consensus_reward();

      if (unlocked_coins < entry_fee) vector::push_back(&mut errors, ELOW_UNLOCKED_COIN_BALANCE); // 17

      (errors, vector::length(&errors) == 0) // friend of ours
  }

  fun bid_as_fixedpoint(bid_pct: u64): fixed_point32::FixedPoint32 {
    fixed_point32::create_from_rational(bid_pct, 1000)
  }
  /// Adjust the reward at the end of the epoch
  /// as described in the paper, the epoch reward needs to be adjustable
  /// given that the implicit bond needs to be sufficient, eg 5-10x the reward.
  /// @return Tuple (bool, bool, u64)
  /// 0: did the thermostat run,
  /// 1: did it increment, or decrease, bool
  /// 2: how much
  /// if the thermostat returns (false, false, 0), it means there was an error running
  public fun reward_thermostat(vm: &signer): (bool, bool, u64) acquires ConsensusReward {
    system_addresses::assert_ol(vm);
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


    if (cr.nominal_reward > 0) {


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
          let less_ten_pct = (cr.nominal_reward / 10);
          cr.nominal_reward = cr.nominal_reward - less_ten_pct;
          return (true, false, less_ten_pct)

        } else if (epochs_above > short_window) {
          // decrease the reward by 5%
          let less_five_pct = (cr.nominal_reward / 20);
          cr.nominal_reward = cr.nominal_reward - less_five_pct;

          return (true, false, less_five_pct)
        }
      };
      // return early since we can't increase and decrease simultaneously


        // if validators are bidding low percentages
        // it means the nominal reward is not high enough.
        // That is the validator's opportunity cost is not met within a
        // range where the bond is meaningful.
        // For example: if the bids for the epoch's reward is 50% of the  value, that means the potential profit, is the same as the potential loss.
        // At a 25% bid (potential loss), the profit is thus 75% of the value, which means the implicit bond is 25/75, or 1/3 of the bond, the risk favors the validator. This means among other things, that an attacker can pay for the cost of the attack with the profits. See paper, for more details.

        // we need to INCREASE the reward, so that the bond is more meaningful.


        if (epochs_below > long_window) {
          // increase the reward by 10%
          let increase_ten_pct = (cr.nominal_reward / 10);
          cr.nominal_reward = cr.nominal_reward + increase_ten_pct;
          return (true, true, increase_ten_pct)
        } else if (epochs_below > short_window) {


          // increase the reward by 5%
          let increase_five_pct = (cr.nominal_reward / 20);
          cr.nominal_reward = cr.nominal_reward + increase_five_pct;
          return (true, true, increase_five_pct)
        };

        // we ran the thermostat but no change was made.
        return (true, false, 0)
    };

    // nominal reward is zero, there's a problem
    return (false, false, 0)
  }

  /// find the median bid to push to history
  // this is needed for reward_thermostat
  public fun set_history(vm: &signer, proposed_validators: &vector<address>) acquires ProofOfFeeAuction, ConsensusReward {
    system_addresses::assert_ol(vm);

    let median_bid = get_median(proposed_validators);
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

  fun get_median(proposed_validators: &vector<address>):u64 acquires ProofOfFeeAuction {
    // TODO: the list is sorted above, so
    // we assume the median is the middle element
    let len = vector::length(proposed_validators);
    if (len == 0) {
      return 0
    };
    let median_bidder = if (len > 2) {
      vector::borrow(proposed_validators, len/2)
    } else {
      vector::borrow(proposed_validators, 0)
    };
    let (median_bid, _) = current_bid(*median_bidder);
    return median_bid
  }

  //////////////// GETTERS ////////////////
  // get the current bid for a validator


  /// get the baseline reward from ConsensusReward
  /// returns (reward, entry_fee, clearing_bid, median_win_bid)
  public fun get_consensus_reward(): (u64, u64, u64, u64) acquires ConsensusReward {
    let b = borrow_global<ConsensusReward>(@ol_framework);
    return (b.nominal_reward, b.entry_fee, b.clearing_bid, b.median_win_bid)
  }

  // CONSENSUS CRITICAL
  // ALL EYES ON THIS
  // Proof of Fee returns the current bid of the validator during the auction for upcoming epoch seats.
  // returns (current bid, expiration epoch)
  #[view]
  public fun current_bid(node_addr: address): (u64, u64) acquires ProofOfFeeAuction {
    if (exists<ProofOfFeeAuction>(node_addr)) {
      let pof = borrow_global<ProofOfFeeAuction>(node_addr);
      let e = epoch_helper::get_current_epoch();
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
      return (epoch_helper::get_current_epoch() >= when_retract,  when_retract)
    };
    return (false, 0)
  }

  // Get the top N validators by bid, this is FILTERED by default
  public fun top_n_accounts(account: &signer, n: u64, unfiltered: bool): vector<address> acquires ProofOfFeeAuction, ConsensusReward {
      system_addresses::assert_vm(account);

      let eligible_validators = get_bidders(unfiltered);
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
    let this_epoch = epoch_helper::get_current_epoch();

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
    nominal_reward: u64,
    clearing_bid: u64,
    median_win_bid: u64,
    median_history: vector<u64>,
  ) acquires ConsensusReward {
    testnet::assert_testnet(vm);

    let cr = borrow_global_mut<ConsensusReward>(@ol_framework );
    cr.nominal_reward = nominal_reward;
    cr.clearing_bid = clearing_bid;
    cr.median_win_bid = median_win_bid;
    cr.median_history = median_history;

  }

  #[test(vm = @ol_framework)]
  fun meta_mock_reward(vm: signer) acquires ConsensusReward {
    use diem_framework::chain_id;

    init_genesis_baseline_reward(&vm);

    chain_id::initialize_for_test(&vm, 4);

    test_mock_reward(
      &vm,
      100,
      50,
      33,
      vector::singleton(33),
    );

    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

  }

  #[test(vm = @ol_framework)]
  public entry fun thermostat_unit_happy(vm: signer)  acquires ConsensusReward {
    use diem_framework::chain_id;
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
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

    reward_thermostat(&vm);

    // This is the happy case. No changes since the rewards were within range
    // the whole time.
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

  }

  // Scenario: The reward is too low during 5 days (short window). People are not bidding very high.
  #[test(vm = @ol_framework)]
  fun thermostat_increase_short(vm: signer) acquires ConsensusReward {
    use diem_framework::chain_id;

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
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 105, 1003);
    assert!(clear_percent == 50, 1004);
    assert!(median_bid == 33, 1005);

  }


  // Scenario: The reward is too low during 5 days (short window). People are not bidding very high.
  #[test(vm = @ol_framework)]
  fun thermostat_increase_long(vm: signer) acquires ConsensusReward {
    use diem_framework::chain_id;

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
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 110, 1003);
    assert!(clear_percent == 50, 1004);
    assert!(median_bid == 33, 1005);

  }


  // Scenario: The reward is too high during 5 days (short window). People are bidding over 95% of the baseline fee.
  #[test(vm = @ol_framework)]
  fun thermostat_decrease_short(vm: signer) acquires ConsensusReward {
    use diem_framework::chain_id;

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
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 95, 1000);
    assert!(clear_percent == 50, 1004);
    assert!(median_bid == 33, 1005);

  }

    // Scenario: The reward is too low during 5 days (short window). People are not bidding very high.
  #[test(vm = @ol_framework)]
  fun thermostat_decrease_long(vm: signer) acquires ConsensusReward {
    use diem_framework::chain_id;

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
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 100, 1000);
    assert!(clear_percent == 50, 1001);
    assert!(median_bid == 33, 1002);

    reward_thermostat(&vm);

    // In the decrease case during a short period, we decrease by 5%
    // No other parameters of consensus reward should change on calling this function.
    let(reward, _, clear_percent, median_bid)  = get_consensus_reward();
    assert!(reward == 90, 1003);
    assert!(clear_percent == 50, 1004);
    assert!(median_bid == 33, 1005);

  }

  // #[test(vm = @ol_framework)]
  // fun pof_set_retract(vm: signer) {
  //     use diem_framework::account;

  //     validator_universe::initialize(&vm);

  //     let sig = account::create_signer_for_test(@0x123);
  //     let (_sk, pk, pop) = stake::generate_identity();
  //     stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);

  //     validator_universe::is_in_universe(@0x123);

  // }
}
