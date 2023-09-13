
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


    // Contains all of 0L's business logic for end of epoch.
    // This removed business logic from reconfiguration.move
    // and prevents dependency cycling.
    public(friend) fun epoch_boundary(root: &signer, closing_epoch: u64) {
        system_addresses::assert_ol(root);
        // bill root service fees;
        root_service_billing(root);
        // run the transactions of donor directed accounts
        donor_directed::process_donor_directed_accounts(root, closing_epoch);
        // reset fee makers tracking
        fee_maker::epoch_reset_fee_maker(root);
        // randomize the Tower/Oracle difficulty
        tower_state::reconfig(root);

        assert!(transaction_fee::is_fees_collection_enabled(), error::invalid_state(ETX_FEES_NOT_INITIALIZED));

        if (transaction_fee::system_fees_collected() > 0) {
          let all_fees = transaction_fee::root_withdraw_all(root);

          // Nominal fee set by the PoF thermostat
          let (nominal_reward_to_vals, clearning_price_to_oracle, _ ) = proof_of_fee::get_consensus_reward();

          // validators get the gross amount of the reward, since they already paid to enter. This results in a net payment equivalidant to the
          // clearing_price.
          process_outgoing_validators(root, &mut all_fees, nominal_reward_to_vals);

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

        process_incoming_validators(root);

        subsidize_from_infra_escrow(root);
    }

  /// process the payments for performant validators
  /// jail the non performant
  /// NOTE: receives from reconfiguration.move a mutable borrow of a coin to pay reward
  /// NOTE: burn remaining fees from transaction fee account happens in reconfiguration.move (it's not a validator_universe concern)
  fun process_outgoing_validators(root: &signer, reward_budget: &mut Coin<GasCoin>, reward_per: u64): vector<address> {
    system_addresses::assert_ol(root);


    let vals = stake::get_current_validators();

    let compliant_vals = vector::empty<address>();
    let i = 0;
    while (i < vector::length(&vals)) {
      let addr = vector::borrow(&vals, i);
      let (performed, _, _, _) = cases::get_validator_grade(*addr);

      if (!performed) {
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

  fun process_incoming_validators(root: &signer) {
    system_addresses::assert_ol(root);


    // TODO: this needs to be a friend function, but it's in a different namespace, so we are gating it with vm signer, which is what was done previously. Which means hacking block.move
    slow_wallet::on_new_epoch(root);

    let (compliant, n_seats) = musical_chairs::stop_the_music(root);

    let validators = proof_of_fee::end_epoch(root, &compliant, n_seats);

    let filled_seats = vector::length(&validators);
    musical_chairs::set_current_seats(filled_seats)

    stake::ol_on_new_epoch(root, validators);

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
  fun root_service_billing(vm: &signer) {
    safe::root_security_fee_billing(vm);
  }


  #[test_only]
  public fun ol_reconfigure_for_test(vm: &signer, closing_epoch: u64) {
      use diem_framework::system_addresses;

      system_addresses::assert_ol(vm);
      epoch_boundary(vm, closing_epoch);
  }

}