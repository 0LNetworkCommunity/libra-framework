
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_slow_wallet {
  use diem_framework::stake;
  use diem_framework::account;
  use ol_framework::slow_wallet;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::gas_coin;
  use ol_framework::epoch_boundary;
  use diem_framework::reconfiguration;
  use diem_framework::coin;
  use std::vector;
  // use diem_std::debug::print;

  #[test(root = @ol_framework)]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_init (root: signer) {
      let _set = mock::genesis_n_vals(&root, 4);
      let list = slow_wallet::get_slow_list();

      // alice, the validator, is already a slow wallet.
      assert!(vector::length<address>(&list) == 4, 735701);

      // frank was not created above
      let sig = account::create_signer_for_test(@0x1000f);
      let (_sk, pk, pop) = stake::generate_identity();
      stake::initialize_test_validator(&root, &pk, &pop, &sig, 100, true, true);

      let list = slow_wallet::get_slow_list();
      assert!(vector::length<address>(&list) == 5, 735701);
  }

  #[test(root = @ol_framework)]
  fun test_epoch_drip(root: signer) {
    let set = mock::genesis_n_vals(&root, 4);
    mock::ol_initialize_coin_and_fund_vals(&root, 100, false);
    let a = vector::borrow(&set, 0);

    assert!(slow_wallet::is_slow(*a), 7357000);
    assert!(slow_wallet::unlocked_amount(*a) == 0, 735701);


    slow_wallet::slow_wallet_epoch_drip(&root, 100);
    assert!(slow_wallet::unlocked_amount(*a) == 100, 735702);
  }

  #[test(root = @ol_framework, alice = @0x123, bob = @0x456)]
  fun test_transfer_happy(root: signer, alice: signer) {
    slow_wallet::initialize(&root);
    ol_account::create_account(&root, @0x123);


    // let _vm = vm;
    // let set = mock::genesis_n_vals(&root, 4);

    // let a_sig = account::create_signer_for_test(*a);

    slow_wallet::set_slow(&alice);
    assert!(slow_wallet::is_slow(@0x123), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735701);
    let (burn_cap, mint_cap) = gas_coin::initialize_for_test(&root);
    ol_account::deposit_coins(@0x123, coin::mint(10000, &mint_cap));

    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);

    ol_account::create_account(&root, @0x456);
    slow_wallet::slow_wallet_epoch_drip(&root, 100);

    assert!(slow_wallet::unlocked_amount(@0x123) == 100, 735703);
    ol_account::transfer(&alice, @0x456, 10);
    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 10, 735704);
  }

  #[test(root = @ol_framework, alice = @0x123, bob = @0x456)]
  #[expected_failure(abort_code = 196614, location = 0x1::ol_account)]

  fun test_transfer_sad(root: signer, alice: signer) {
    slow_wallet::initialize(&root);
    ol_account::create_account(&root, @0x123);

    slow_wallet::set_slow(&alice);
    assert!(slow_wallet::is_slow(@0x123), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735701);
    let (burn_cap, mint_cap) = gas_coin::initialize_for_test(&root);
    ol_account::deposit_coins(@0x123, coin::mint(10000, &mint_cap));

    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);
    ol_account::create_account(&root, @0x456);
    ol_account::transfer(&alice, @0x456, 99);

    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 0, 735702);
    slow_wallet::slow_wallet_epoch_drip(&root, 100);

    assert!(slow_wallet::unlocked_amount(@0x123) == 100, 735703);
    ol_account::transfer(&alice, @0x456, 200); // TOO MUCH
    let b_balance = coin::balance<gas_coin::GasCoin>(@0x456);
    assert!(b_balance == 10, 735704);
  }


  #[test(root = @ol_framework)]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_reconfigure (root: signer) {
    let set = mock::genesis_n_vals(&root, 4);
    mock::ol_initialize_coin(&root);
    let a = vector::borrow(&set, 0);
    assert!(slow_wallet::unlocked_amount(*a) == 0, 735701);
    epoch_boundary::ol_reconfigure_for_test(&root, reconfiguration::get_current_epoch())

  }


}