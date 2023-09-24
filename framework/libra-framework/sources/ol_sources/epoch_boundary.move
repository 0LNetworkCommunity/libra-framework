
module diem_framework::epoch_boundary {
    use ol_framework::slow_wallet;
    use ol_framework::musical_chairs;
    use ol_framework::proof_of_fee;
    use ol_framework::stake;
    use ol_framework::gas_coin::GasCoin;
    use ol_framework::rewards;
    use ol_framework::jail;
    use ol_framework::cases;
    use ol_framework::safe;
    use ol_framework::burn;
    use ol_framework::donor_directed;
    use ol_framework::fee_maker;
    use ol_framework::tower_state;
    use ol_framework::infra_escrow;
    use ol_framework::oracle;
    use ol_framework::testnet;
    use diem_framework::transaction_fee;
    use diem_framework::system_addresses;
    use diem_framework::coin::{Self, Coin};
    use std::vector;
    use std::error;

    // use diem_std::debug::print;

    friend diem_framework::block;

    /// how many PoF baseline rewards to we set aside for the miners.
    /// equivalent reward of one seats of the validator set
    const ORACLE_PROVIDERS_SEATS: u64 = 1;

    /// The transaction fee coin has not been initialized
    const ETX_FEES_NOT_INITIALIZED: u64 = 0;


    // I just checked in, to see what condition my condition was in.
    struct BoundaryStatus has key {
      security_bill_count: u64,
      security_bill_amount: u64,
      security_bill_success: bool,

      dd_accounts_count: u64,
      dd_accounts_amount: u64,
      dd_accounts_success: bool,

      set_fee_makers_success: bool,
      tower_state_success: bool,
      system_fees_collected: u64,
      nominal_reward_to_vals: u64,
      clearing_price: u64,
      // Process Outgoing
      process_outgoing_success: bool,
      outgoing_vals: vector<address>,
      oracle_budget: u64,
      oracle_pay_success: bool,
      epoch_burn_fees: u64,
      epoch_burn_success: bool,
      slow_wallet_drip: bool,
      // Process Incoming
      outgoing_compliant_count: u64,
      incoming_seats_offered: u64,
      incoming_filled_seats: u64,
      incoming_expected_vals: vector<address>,
      incoming_post_failover_check: vector<address>,
      incoming_actual_vals: vector<address>,
      incoming_reconfig_success: bool,

      incoming_fees: u64,
      incoming_fees_success: bool,


      infra_subsize_amount: u64,
      infra_subsizize_success: bool,
    }

    public fun initialize(framework: &signer) {
      if (!exists<BoundaryStatus>(@ol_framework)){
        move_to(framework, BoundaryStatus {
          security_bill_count: 0,
          security_bill_amount: 0,
          security_bill_success: false,

          dd_accounts_count: 0,
          dd_accounts_amount: 0,
          dd_accounts_success: false,

          set_fee_makers_success: false,
          tower_state_success: false,
          system_fees_collected: 0,
          nominal_reward_to_vals: 0,
          clearing_price: 0,
          // Process Outgoing
          process_outgoing_success: false,
          outgoing_vals: vector::empty(),
          oracle_budget: 0,
          oracle_pay_success: false,
          epoch_burn_fees: 0,
          epoch_burn_success: false,
          slow_wallet_drip: false,
          // Process Incoming
          outgoing_compliant_count: 0,
          incoming_seats_offered: 0,
          incoming_filled_seats: 0,
          incoming_expected_vals: vector::empty(),
          incoming_post_failover_check: vector::empty(),
          incoming_actual_vals: vector::empty(),
          incoming_reconfig_success: false,

          incoming_fees: 0,
          incoming_fees_success: false,


          infra_subsize_amount: 0,
          infra_subsizize_success: false,
        });
      }
    }

    // Contains all of 0L's business logic for end of epoch.
    // This removed business logic from reconfiguration.move
    // and prevents dependency cycling.
    public(friend) fun epoch_boundary(root: &signer, closing_epoch: u64, epoch_round: u64) acquires BoundaryStatus {
        system_addresses::assert_ol(root);

        let status = borrow_global_mut<BoundaryStatus>(@ol_framework);
        // bill root service fees;
        root_service_billing(root, status);
        // run the transactions of donor directed accounts
        let (count, amount, success) = donor_directed::process_donor_directed_accounts(root, closing_epoch);
        status.dd_accounts_count = count;
        status.dd_accounts_amount = amount;
        status.dd_accounts_success = success;

        // reset fee makers tracking
        fee_maker::epoch_reset_fee_maker(root);
        // randomize the Tower/Oracle difficulty
        tower_state::reconfig(root);

        settle_accounts(root, closing_epoch, epoch_round, status);

        // drip coins
        slow_wallet::on_new_epoch(root);

        // ======= THIS IS APPROXIMATELY THE BOUNDARY =====
        process_incoming_validators(root, status);

        subsidize_from_infra_escrow(root);
  }

  // TODO: instrument all of this
  /// withdraw coins and settle accounts for validators and oracles
  fun settle_accounts(root: &signer, closing_epoch: u64, epoch_round: u64, _status: &mut BoundaryStatus) {
        assert!(transaction_fee::is_fees_collection_enabled(), error::invalid_state(ETX_FEES_NOT_INITIALIZED));

        if (transaction_fee::system_fees_collected() > 0) {
          let all_fees = transaction_fee::root_withdraw_all(root);

          // Nominal fee set by the PoF thermostat
          let (nominal_reward_to_vals, clearning_price_to_oracle, _ ) = proof_of_fee::get_consensus_reward();

          // validators get the gross amount of the reward, since they already paid to enter. This results in a net payment equivalidant to the
          // clearing_price.
          process_outgoing_validators(root, &mut all_fees, nominal_reward_to_vals, closing_epoch, epoch_round);

          // since we reserved some fees to go to the oracle miners
          // we take the clearing_price, since it is the equivalent of what a
          // validator would earn net of entry fee.
          let oracle_budget = coin::extract(&mut all_fees, clearning_price_to_oracle);
          oracle::epoch_boundary(root, &mut oracle_budget);
          // in case there is any dust left
          coin::merge(&mut all_fees, oracle_budget);

          // remainder gets burnt according to fee maker preferences
          burn::epoch_burn_fees(root, &mut all_fees);
          // there might be some dust, that should get burned
          coin::user_burn(all_fees);
        };
  }


  /// process the payments for performant validators
  /// jail the non performant
  /// NOTE: receives from reconfiguration.move a mutable borrow of a coin to pay reward
  /// NOTE: burn remaining fees from transaction fee account happens in reconfiguration.move (it's not a validator_universe concern)
  fun process_outgoing_validators(root: &signer, reward_budget: &mut Coin<GasCoin>, reward_per: u64, closing_epoch: u64, epoch_round: u64): vector<address> {
    system_addresses::assert_ol(root);


    let vals = stake::get_current_validators();

    let compliant_vals = vector::empty<address>();
    let i = 0;
    while (i < vector::length(&vals)) {
      let addr = vector::borrow(&vals, i);

      let (performed, _, _, _) = cases::get_validator_grade(*addr);
      // Failover. if we had too few blocks in an epoch, everyone should pass
      // except for testing
      if (!testnet::is_testnet() && (epoch_round < 1000)) performed = true;

      if (!performed && closing_epoch > 1) { // issues around genesis
        jail::jail(root, *addr);
      } else {
        vector::push_back(&mut compliant_vals, *addr);
        if (coin::value(reward_budget) > reward_per) {
          let user_coin = coin::extract(reward_budget, reward_per);
          rewards::process_single(root, *addr, user_coin, 1);
        }
      };

      i = i + 1;
    };

    return compliant_vals
  }

  fun process_incoming_validators(root: &signer, status: &mut BoundaryStatus) {
    system_addresses::assert_ol(root);

    let (compliant, n_seats) = musical_chairs::stop_the_music(root);
    status.outgoing_compliant_count = vector::length(&compliant);
    status.incoming_seats_offered = n_seats;
    // check amount of fees expected
    let proposed_new_validators = proof_of_fee::end_epoch(root, &compliant, n_seats);
    status.incoming_expected_vals = proposed_new_validators;

    // showtime! try to reconfigure
    let (actual_set, post_failover_check, success) = stake::maybe_reconfigure(root, proposed_new_validators);
    status.incoming_post_failover_check = post_failover_check;
    status.incoming_reconfig_success = success;

    // make sure musical chairs doesn't keep incrementing if we are persistently
    // offering more seats than can be filled
    let filled_seats = vector::length(&actual_set);
    status.incoming_filled_seats = filled_seats;
    musical_chairs::set_current_seats(root, filled_seats);

  }

  // set up rewards subsidy for coming epoch
  fun subsidize_from_infra_escrow(root: &signer) {
      system_addresses::assert_ol(root);
      let (reward_per, _, _ ) = proof_of_fee::get_consensus_reward();
      let vals = stake::get_current_validators();
      let count_vals = vector::length(&vals);
      count_vals = count_vals + ORACLE_PROVIDERS_SEATS;
      let total_epoch_budget = count_vals * reward_per;
      infra_escrow::epoch_boundary_collection(root, total_epoch_budget);
  }

  // all services the root collective security is billing for
  fun root_service_billing(vm: &signer, status: &mut BoundaryStatus) {
    let (security_bill_count, security_bill_amount, security_bill_success) = safe::root_security_fee_billing(vm);
    status.security_bill_count = security_bill_count;
    status.security_bill_amount = security_bill_amount;
    status.security_bill_success = security_bill_success;
  }


  #[test_only]
  public fun ol_reconfigure_for_test(vm: &signer, closing_epoch: u64, epoch_round: u64) acquires BoundaryStatus {
      use diem_framework::system_addresses;

      system_addresses::assert_ol(vm);
      epoch_boundary(vm, closing_epoch, epoch_round);
  }

}