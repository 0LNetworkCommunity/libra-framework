
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_ol_account {
  use aptos_framework::stake;
  use aptos_framework::account;
  use ol_framework::ol_account;
  use ol_framework::mock;
  use aptos_std::debug::print;
  use std::vector;

  #[test]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_init () {
      let _set = mock::genesis_n_vals(4);
      let list = ol_account::get_slow_list();
      print(&list);
      // alice, the validator, is already a slow wallet.
      assert!(vector::length<address>(&list) == 4, 735701);

      // frank was not created above
      let sig = account::create_signer_for_test(@0x1000f);
      let (_sk, pk, pop) = stake::generate_identity();
      stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);

      let list = ol_account::get_slow_list();
      assert!(vector::length<address>(&list) == 5, 735701);
    
  }


}