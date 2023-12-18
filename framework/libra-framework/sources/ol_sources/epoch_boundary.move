
module diem_framework::epoch_boundary {
    use ol_framework::slow_wallet;
    use ol_framework::musical_chairs;
    use ol_framework::proof_of_fee;
    use ol_framework::stake;
    use ol_framework::libra_coin::LibraCoin;
    use ol_framework::rewards;
    use ol_framework::jail;
    use ol_framework::safe;
    use ol_framework::burn;
    use ol_framework::donor_voice;
    use ol_framework::fee_maker;
    use ol_framework::tower_state;
    use ol_framework::infra_escrow;
    use ol_framework::oracle;
    use ol_framework::ol_account;
    use ol_framework::testnet;
    use diem_framework::reconfiguration;
    use diem_framework::transaction_fee;
    use diem_framework::system_addresses;
    use diem_framework::coin::{Self, Coin};
    use std::vector;
    use std::error;
    use std::string;

    use diem_std::debug::print;

    friend diem_framework::diem_governance;
    friend diem_framework::block; // for testnet only

    //////// ERRORS ////////
    /// The transaction fee coin has not been initialized
    const ETX_FEES_NOT_INITIALIZED: u64 = 0;

    /// Epoch trigger only implemented on mainnet
    const ETRIGGER_EPOCH_MAINNET: u64 = 1;

    /// Epoch is not ready for reconfiguration
    const ETRIGGER_NOT_READY: u64 = 2;

    /// Epoch number mismat
    const ENOT_SAME_EPOCH: u64 = 3;

    /////// Constants ////////
    /// how many PoF baseline rewards to we set aside for the miners.
    /// equivalent reward of one seats of the validator set
    const ORACLE_PROVIDERS_SEATS: u64 = 1;

    // the VM can set the boundary bit to allow reconfiguration
    struct BoundaryBit has key {
      ready: bool,
      closing_epoch: u64,
    }

    // I just checked in, to see what condition my condition was in.
    struct BoundaryStatus has key, drop {
      security_bill_count: u64,
      security_bill_amount: u64,
      security_bill_success: bool,

      dd_accounts_count: u64,
      dd_accounts_amount: u64,
      dd_accounts_success: bool,

      set_fee_makers_success: bool,
      system_fees_collected: u64,
      // Process Outgoing
      outgoing_vals_paid: vector<address>,
      outgoing_total_reward: u64,
      outgoing_nominal_reward_to_vals: u64,
      outgoing_entry_fee: u64,
      outgoing_clearing_percent: u64,
      outgoing_vals_success: bool, // TODO

      // Oracle / Tower
      tower_state_success: bool, // TODO
      oracle_budget: u64,
      oracle_pay_count: u64,
      oracle_pay_amount: u64,
      oracle_pay_success: bool,

      epoch_burn_fees: u64,
      epoch_burn_success: bool,

      slow_wallet_drip_amount: u64,
      slow_wallet_drip_success: bool,
      // Process Incoming
      // musical chairs
      incoming_compliant: vector<address>,
      incoming_compliant_count: u64,
      incoming_seats_offered: u64,

      // proof of fee
      incoming_all_bidders: vector<address>,
      incoming_only_qualified_bidders: vector<address>,
      incoming_auction_winners: vector<address>,
      incoming_filled_seats: u64,
      incoming_fees: u64,
      incoming_fees_success: bool,

      // reconfiguration
      incoming_post_failover_check: vector<address>,
      incoming_vals_missing_configs: vector<address>,
      incoming_actual_vals: vector<address>,
      incoming_final_set_size: u64,
      incoming_reconfig_success: bool,

      infra_subsidize_amount: u64, // TODO
      infra_subsidize_success: bool, // TODO

      pof_thermo_success: bool,
      pof_thermo_increase: bool,
      pof_thermo_amount: u64,
    }

    public fun initialize(framework: &signer) {
      if (!exists<BoundaryStatus>(@ol_framework)){
        move_to(framework, reset());
      };

      if (!exists<BoundaryBit>(@ol_framework)) {
        move_to(framework, BoundaryBit {
          ready: false,
          closing_epoch: 0,
        });
      }
    }

    fun reset(): BoundaryStatus {
      BoundaryStatus {
          security_bill_count: 0,
          security_bill_amount: 0,
          security_bill_success: false,

          dd_accounts_count: 0,
          dd_accounts_amount: 0,
          dd_accounts_success: false,

          set_fee_makers_success: false,
          system_fees_collected: 0,

          // Process Outgoing
          outgoing_vals_paid: vector::empty(),
          outgoing_total_reward: 0,
          outgoing_vals_success: false,
          outgoing_nominal_reward_to_vals: 0,
          outgoing_entry_fee: 0,
          outgoing_clearing_percent: 0,

          // Oracle / Tower
          tower_state_success: false,
          oracle_budget: 0,
          oracle_pay_count: 0,
          oracle_pay_amount: 0,
          oracle_pay_success: false,
          epoch_burn_fees: 0,
          epoch_burn_success: false,

          slow_wallet_drip_amount: 0,
          slow_wallet_drip_success: false,
          // Process Incoming
          incoming_compliant: vector::empty(),
          incoming_compliant_count: 0,
          incoming_seats_offered: 0,
          incoming_filled_seats: 0,
          incoming_auction_winners: vector::empty(),
          incoming_all_bidders: vector::empty(),
          incoming_only_qualified_bidders: vector::empty(),
          incoming_final_set_size: 0,
          incoming_fees: 0,
          incoming_fees_success: false,

          incoming_post_failover_check: vector::empty(),
          incoming_vals_missing_configs: vector::empty(),
          incoming_actual_vals: vector::empty(),
          incoming_reconfig_success: false,

          infra_subsidize_amount: 0,
          infra_subsidize_success: false,

          pof_thermo_success: false,
          pof_thermo_increase: false,
          pof_thermo_amount: 0,
        }
    }


    /// flip the bit to allow the epoch to be reconfigured on any transaction
    public(friend) fun enable_epoch_trigger(vm_signer: &signer, closing_epoch: u64)
    acquires BoundaryBit {

      if (!exists<BoundaryBit>(@ol_framework)) {
        move_to(vm_signer, BoundaryBit {
          closing_epoch: closing_epoch,
          ready: true,
        })
      } else {
        let state = borrow_global_mut<BoundaryBit>(@ol_framework);
        state.closing_epoch = closing_epoch;
        state.ready = true;
      }
    }
    /// Once epoch boundary time has passed, and the BoundaryBit set to true
    /// any user can trigger the epoch boundary.
    /// Why do this? It's preferable that the VM never trigger any function.
    /// An abort by the VM will cause a network halt. The same abort, if called
    /// by a user, would not cause a halt.
    public(friend) fun trigger_epoch(root: &signer) acquires BoundaryBit,
    BoundaryStatus {
      // must be mainnet
      assert!(!testnet::is_not_mainnet(), ETRIGGER_EPOCH_MAINNET);
      // must get root permission from governance.move
      system_addresses::assert_ol(root);
      let _ = can_trigger(); // will abort if false

      // update the state and flip the Bit
      let state = borrow_global_mut<BoundaryBit>(@ol_framework);
      state.ready = false;

      epoch_boundary(root, state.closing_epoch, 0);
    }

    #[view]
    /// check to see if the epoch Boundary Bit is true
    public fun can_trigger(): bool acquires BoundaryBit {
      let state = borrow_global_mut<BoundaryBit>(@ol_framework);
      assert!(state.ready, ETRIGGER_NOT_READY);
      assert!(state.closing_epoch == reconfiguration::get_current_epoch(),
      ENOT_SAME_EPOCH);
      true
    }

    // Contains all of 0L's business logic for end of epoch.
    // This removed business logic from reconfiguration.move
    // and prevents dependency cycling.
    public(friend) fun epoch_boundary(root: &signer, closing_epoch: u64,
    _epoch_round: u64) acquires BoundaryStatus {
        print(&string::utf8(b"EPOCH BOUNDARY BEGINS"));
        system_addresses::assert_ol(root);

        let status = borrow_global_mut<BoundaryStatus>(@ol_framework);

        print(&string::utf8(b"status reset"));
        *status = reset();
        // bill root service fees;

        print(&string::utf8(b"root_service_billing"));
        root_service_billing(root, status);

        print(&string::utf8(b"process_donor_voice_accounts"));
        // run the transactions of donor directed accounts
        let (count, amount, success) = donor_voice::process_donor_voice_accounts(root, closing_epoch);
        status.dd_accounts_count = count;
        status.dd_accounts_amount = amount;
        status.dd_accounts_success = success;

        print(&string::utf8(b"tower_state::reconfig"));
        // reset fee makers tracking
        status.set_fee_makers_success = fee_maker::epoch_reset_fee_maker(root);


        print(&string::utf8(b"tower_state::reconfig"));
        // randomize the Tower/Oracle difficulty
        tower_state::reconfig(root);
        // TODO: there isn't much to checkhere.
        status.tower_state_success = true;

        print(&string::utf8(b"musical_chairs::stop_the_music"));
        let (compliant_vals, n_seats) = musical_chairs::stop_the_music(root, closing_epoch);
        status.incoming_compliant_count = vector::length(&compliant_vals);
        status.incoming_compliant = compliant_vals;
        status.incoming_seats_offered = n_seats;

        print(&string::utf8(b"settle_accounts"));

        settle_accounts(root, compliant_vals, status);

        print(&string::utf8(b"slow_wallet::on_new_epoch"));
        // drip coins
        let (s_success, s_amount) = slow_wallet::on_new_epoch(root);

        status.slow_wallet_drip_amount = s_amount;
        status.slow_wallet_drip_success = s_success;

        // ======= THE BOUNDARY =======
        // And to know yourself
        // is to be yourself
        // keeps you walking through these tears.

        print(&string::utf8(b"process_incoming_validators"));
        process_incoming_validators(root, status, compliant_vals, n_seats);

        print(&string::utf8(b"subsidize_from_infra_escrow"));
        let (i_success, i_fee) = subsidize_from_infra_escrow(root);
        status.infra_subsidize_amount = i_fee;
        status.infra_subsidize_success = i_success;

        print(&string::utf8(b"reward_thermostat"));
        let (t_success, t_increase, t_amount) =
        proof_of_fee::reward_thermostat(root);
        status.pof_thermo_success = t_success;
        status.pof_thermo_increase = t_increase;
        status.pof_thermo_amount = t_amount;

        print(&string::utf8(b"EPOCH BOUNDARY END"));
        print(status);
        reconfiguration::reconfigure();
  }

  /// withdraw coins and settle accounts for validators and oracles
  /// returns the list of compliant_vals
  fun settle_accounts(root: &signer, compliant_vals: vector<address>, status: &mut BoundaryStatus): vector<address> {
        assert!(transaction_fee::is_fees_collection_enabled(), error::invalid_state(ETX_FEES_NOT_INITIALIZED));

        if (transaction_fee::system_fees_collected() > 0) {
          let all_fees = transaction_fee::root_withdraw_all(root);
          status.system_fees_collected = coin::value(&all_fees);

          // Nominal fee set by the PoF thermostat
          let (nominal_reward_to_vals, entry_fee, clearing_percent, _ ) = proof_of_fee::get_consensus_reward();
          status.outgoing_nominal_reward_to_vals = nominal_reward_to_vals;
          status.outgoing_entry_fee = entry_fee;
          status.outgoing_clearing_percent = clearing_percent;

          // validators get the gross amount of the reward, since they already paid to enter. This results in a net payment equivalent to:
          // nominal_reward_to_vals - entry_fee.
          let (compliant_vals, total_reward) = process_outgoing_validators(root, &mut all_fees, nominal_reward_to_vals, compliant_vals);

          status.outgoing_vals_paid = compliant_vals;
          status.outgoing_total_reward = total_reward;

          // check that the sustem fees collect were greater than reward
          status.outgoing_vals_success = (status.system_fees_collected >= total_reward);
          // check the the total actually deposited/paid is the expected amount
          if (nominal_reward_to_vals > 0) { // check for zero
            status.outgoing_vals_success = total_reward == (vector::length(&compliant_vals) * nominal_reward_to_vals)
          };

          // since we reserved some fees to go to the oracle miners
          // we take the NET REWARD of the validators, since it is the equivalent of what the validator would earn net of entry fee.
            if (nominal_reward_to_vals > entry_fee) {
                let net_val_reward = nominal_reward_to_vals - entry_fee;

                if (coin::value(&all_fees) > net_val_reward) {
                    let oracle_budget = coin::extract(&mut all_fees, net_val_reward);
                    status.oracle_budget = coin::value(&oracle_budget);

                    let (count, amount) = oracle::epoch_boundary(root, &mut oracle_budget);
                    status.oracle_pay_count = count;
                    status.oracle_pay_amount = amount;
                    status.oracle_pay_success = (amount > 0);
                    // in case there is any dust left
                    ol_account::merge_coins(&mut all_fees, oracle_budget);
                };
            };

          // remainder gets burnt according to fee maker preferences
          let (b_success, b_fees) = burn::epoch_burn_fees(root, &mut all_fees);
          status.epoch_burn_success = b_success;
          status.epoch_burn_fees = b_fees;

          // coin can finally be destroyed. Up to here we have been extracting from a mutable.
          // It's possible there might be some dust, that should get burned
          burn::burn_and_track(all_fees);
        };

        compliant_vals
  }


  /// process the payments for performant validators
  /// jail the non performant
  /// NOTE: receives from reconfiguration.move a mutable borrow of a coin to pay reward
  /// NOTE: burn remaining fees from transaction fee account happens in reconfiguration.move (it's not a validator_universe concern)
  // Returns (compliant_vals, reward_deposited)
  fun process_outgoing_validators(root: &signer, reward_budget: &mut Coin<LibraCoin>, reward_per: u64, compliant_vals: vector<address>): (vector<address>, u64){
    system_addresses::assert_ol(root);
    let vals = stake::get_current_validators();
    let reward_deposited = 0;

    let i = 0;
    while (i < vector::length(&vals)) {
      let addr = vector::borrow(&vals, i);
      let performed = vector::contains(&compliant_vals, addr);
      if (!performed) {
        jail::jail(root, *addr);
      } else {
        // vector::push_back(&mut compliant_vals, *addr);
        if (coin::value(reward_budget) > reward_per) {
          let user_coin = coin::extract(reward_budget, reward_per);
          reward_deposited = reward_deposited + coin::value(&user_coin);
          rewards::process_single(root, *addr, user_coin, 1);
        }
      };

      i = i + 1;
    };

    return (compliant_vals, reward_deposited)
  }

  fun process_incoming_validators(root: &signer, status: &mut BoundaryStatus, compliant_vals: vector<address>, n_seats: u64) {
    system_addresses::assert_ol(root);


    // check amount of fees expected
    let (auction_winners, all_bidders, only_qualified_bidders, entry_fee) = proof_of_fee::end_epoch(root, &compliant_vals, n_seats);
    status.incoming_filled_seats = vector::length(&auction_winners);
    status.incoming_all_bidders = all_bidders;
    status.incoming_only_qualified_bidders = only_qualified_bidders;
    status.incoming_auction_winners = auction_winners;


    let post_failover_check = stake::check_failover_rules(auction_winners, n_seats);
    status.incoming_post_failover_check = post_failover_check;

    // showtime! try to reconfigure
    let (actual_set, vals_missing_configs, success) = stake::maybe_reconfigure(root, post_failover_check);
    status.incoming_vals_missing_configs = vals_missing_configs;
    status.incoming_actual_vals = actual_set;
    status.incoming_reconfig_success = success;

    let (_expected_fees, fees_paid, fee_success) = proof_of_fee::charge_epoch_fees(root, actual_set, entry_fee);
    status.incoming_fees = fees_paid;
    status.incoming_fees_success = fee_success;
    // make sure musical chairs doesn't keep incrementing if we are persistently
    // offering more seats than can be filled
    let final_set_size = vector::length(&actual_set);
    musical_chairs::set_current_seats(root, final_set_size);
    status.incoming_final_set_size = final_set_size;
  }

  // set up rewards subsidy for coming epoch
  fun subsidize_from_infra_escrow(root: &signer): (bool, u64) {
      system_addresses::assert_ol(root);
      let (reward_per, _, _, _ ) = proof_of_fee::get_consensus_reward();
      let vals = stake::get_current_validators();
      let count_vals = vector::length(&vals);
      count_vals = count_vals + ORACLE_PROVIDERS_SEATS;
      let total_epoch_budget = count_vals * reward_per;
      infra_escrow::epoch_boundary_collection(root,
      total_epoch_budget)
  }

  // all services the root collective security is billing for
  fun root_service_billing(vm: &signer, status: &mut BoundaryStatus) {
    let (security_bill_count, security_bill_amount, security_bill_success) = safe::root_security_fee_billing(vm);
    status.security_bill_count = security_bill_count;
    status.security_bill_amount = security_bill_amount;
    status.security_bill_success = security_bill_success;
  }

  //////// GETTERS ////////
  #[view]
  public fun get_reconfig_success(): bool acquires BoundaryStatus {
    borrow_global<BoundaryStatus>(@ol_framework).incoming_reconfig_success
  }
  #[view]
  public fun get_actual_vals(): vector<address> acquires BoundaryStatus {
    borrow_global<BoundaryStatus>(@ol_framework).incoming_actual_vals
  }

  #[view]
  public fun get_qualified_bidders(): vector<address> acquires BoundaryStatus {
    borrow_global<BoundaryStatus>(@ol_framework).incoming_only_qualified_bidders
  }

  #[view]
  public fun get_auction_winners(): vector<address> acquires BoundaryStatus {
    borrow_global<BoundaryStatus>(@ol_framework).incoming_auction_winners
  }

  #[view]
  public fun get_seats_offered():u64 acquires BoundaryStatus {
    borrow_global<BoundaryStatus>(@ol_framework).incoming_seats_offered
  }

  #[test_only]
  public fun ol_reconfigure_for_test(vm: &signer, closing_epoch: u64,
  epoch_round: u64) acquires BoundaryStatus {
      use diem_framework::system_addresses;

      system_addresses::assert_ol(vm);
      epoch_boundary(vm, closing_epoch, epoch_round);
  }

  #[test_only]
  public fun test_set_boundary_ready(vm: &signer, closing_epoch: u64) acquires
  BoundaryBit {
      // don't check for "testnet" here, otherwise we can't test the production settings
      enable_epoch_trigger(vm, closing_epoch);
  }

  #[test_only]
  public fun test_trigger(vm: &signer) acquires BoundaryStatus, BoundaryBit {
      // don't check for "testnet" here, otherwise we can't test the production settings
      trigger_epoch(vm);
  }
}
