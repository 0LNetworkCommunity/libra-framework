
module aptos_framework::epoch_boundary {

    use std::signer;
    use ol_framework::slow_wallet;
    use ol_framework::musical_chairs;
    use ol_framework::proof_of_fee;
    use ol_framework::stake;
    use ol_framework::gas_coin::GasCoin;
    use ol_framework::rewards;
    use ol_framework::jail;
    use ol_framework::cases;
    use aptos_framework::system_addresses;
    use aptos_framework::coin::{Self, Coin};
    use std::vector;

    // use aptos_std::debug::print;

    friend aptos_framework::block;

    // Contains all of 0L's business logic for end of epoch.
    // This removed business logic from reconfiguration.move
    // and prevents dependency cycling.
    public(friend) fun epoch_boundary(root: &signer) {
        if (signer::address_of(root) != @ol_framework) {
            return
        };

        // TODO: this needs to be a friend function, but it's in a different namespace, so we are gating it with vm signer, which is what was done previously. Which means hacking block.move
        slow_wallet::on_new_epoch(root);

        let (compliant, n_seats) = musical_chairs::stop_the_music(root);  

        let validators = proof_of_fee::end_epoch(root, &compliant, n_seats);

        stake::ol_on_new_epoch(root, validators);

    }

  /// process the payments for performant validators
  /// jail the non performant
  /// NOTE: receives from reconfiguration.move a mutable borrow of a coin to pay reward
  /// NOTE: burn remaining fees from transaction fee account happens in reconfiguration.move (it's not a validator_universe concern)
  public(friend) fun end_epoch_process_outgoing(root: &signer, reward_budget: &mut Coin<GasCoin>): vector<address> {
    system_addresses::assert_ol(root);

    // TODO: get proof of fee reward
    let reward_per = 1000;

    let vals = stake::get_current_validators();

    let compliant_vals = vector::empty<address>();
    let i = 0;
    while (i < vector::length(&vals)) {
      let addr = vector::borrow(&vals, i);
      let (performed, _, _, _) = cases::get_validator_grade(*addr);

      if (!performed) {
        jail::jail(root, *addr);
      } else {
        let user_coin = coin::extract(reward_budget, reward_per);
        rewards::process_single(root, *addr, user_coin, 1);
        vector::push_back(&mut compliant_vals, *addr);
      };
      
      i = i + 1;
    };

    return compliant_vals
  }

    #[test_only]
    public fun ol_reconfigure_for_test(vm: &signer) {
        use aptos_framework::system_addresses;

        system_addresses::assert_ol(vm);
        epoch_boundary(vm);
    }

}