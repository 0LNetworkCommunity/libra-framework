
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_slow_wallet {
  use aptos_framework::stake;
  use aptos_framework::account;
  use ol_framework::slow_wallet;
  use ol_framework::mock;
  use aptos_std::debug::print;
  use std::vector;

  #[test]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_init () {
      let _set = mock::genesis_n_vals(4);
      let list = slow_wallet::get_slow_list();
      print(&list);
      // alice, the validator, is already a slow wallet.
      assert!(vector::length<address>(&list) == 4, 735701);

      // frank was not created above
      let sig = account::create_signer_for_test(@0x1000f);
      let (_sk, pk, pop) = stake::generate_identity();
      stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);

      let list = slow_wallet::get_slow_list();
      assert!(vector::length<address>(&list) == 5, 735701);
    
  }

  #[test(vm=@ol_framework)]
  fun test_epoch_drip(vm: signer) {
    let set = mock::genesis_n_vals(4);
    let a = vector::borrow(&set, 0);
    let a_sig = account::create_signer_for_test(*a);

    slow_wallet::set_slow(&a_sig);
    assert!(slow_wallet::unlocked_amount(*a) == 0, 735701);

    slow_wallet::slow_wallet_epoch_drip(&vm, 100);
    assert!(slow_wallet::unlocked_amount(*a) == 100, 735702);
  }


}