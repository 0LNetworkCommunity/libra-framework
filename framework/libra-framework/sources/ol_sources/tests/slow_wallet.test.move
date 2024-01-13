
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_slow_wallet {
  use diem_framework::stake;
  use diem_framework::account;
  use ol_framework::slow_wallet;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::libra_coin::{Self, LibraCoin};
  use ol_framework::epoch_boundary;
  use diem_framework::reconfiguration;
  use diem_framework::coin;
  use diem_framework::block;
  use ol_framework::transaction_fee;
  use ol_framework::rewards;
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

    let coin = transaction_fee::test_root_withdraw_all(&root);
    rewards::test_helper_pay_reward(&root, *a, coin, 0);

    let (u, b) = ol_account::balance(*a);
    assert!(b==100000100, 735702);
    assert!(u==0, 735703);

    slow_wallet::slow_wallet_epoch_drip(&root, 233);
    let (u, b) = ol_account::balance(*a);
    assert!(b==100000100, 735704);
    assert!(u==233, 735705);
  }

  #[test(root = @ol_framework, alice = @0x123, bob = @0x456)]
  fun test_deposit_locked_happy(root: signer, alice: signer) {
    mock::ol_test_genesis(&root);
    let mint_cap = libra_coin::extract_mint_cap(&root);

    slow_wallet::initialize(&root);


    // create alice account
    ol_account::create_account(&root, @0x123);

    slow_wallet::set_slow(&alice);

    assert!(slow_wallet::is_slow(@0x123), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735701);

    // add some coins to alice
    let alice_init_balance = 10000;
    ol_account::deposit_coins(@0x123, coin::test_mint(alice_init_balance, &mint_cap));
    coin::destroy_mint_cap(mint_cap);
    // the deposit was not unlocked coins, so there should be no unlocked coins
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735703);
    assert!(slow_wallet::transferred_amount(@0x123) == 0, 735704);

  }

  // scenario: testing sending unlocked coins from slow wallet to slow wallet
  #[test(root = @ol_framework, alice = @0x123, bob = @0x456, carol = @0x789)]
  fun test_transfer_unlocked_happy(root: signer, alice: signer, bob: signer, carol: signer) {
    mock::ol_test_genesis(&root);
    let mint_cap = libra_coin::extract_mint_cap(&root);

    slow_wallet::initialize(&root);

    // create alice, bob, and carol accounts
    ol_account::create_account(&root, @0x123);
    ol_account::create_account(&root, @0x456);
    ol_account::create_account(&root, @0x789);

    // Make bob and carol accounts slow
    slow_wallet::set_slow(&bob);
    slow_wallet::set_slow(&carol);

    assert!(slow_wallet::is_slow(@0x456), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x456) == 0, 735701);
    assert!(slow_wallet::transferred_amount(@0x456) == 0, 735710);
    assert!(slow_wallet::is_slow(@0x789), 7357011);
    assert!(slow_wallet::unlocked_amount(@0x789) == 0, 735712);
    assert!(slow_wallet::transferred_amount(@0x789) == 0, 735713);

    // add some coins to alice which are unlocked since they are not a slow wallet
    let alice_init_balance = 10000;
    ol_account::deposit_coins(@0x123, coin::test_mint(alice_init_balance, &mint_cap));
    coin::destroy_mint_cap(mint_cap);
    
    // transfer unlocked coins from alice to bob
    let alice_to_bob_transfer_amount = 100;
    ol_account::transfer(&alice, @0x456, alice_to_bob_transfer_amount);

    // the transfer was of already unlocked coins, so they will post as unlocked on bob
    assert!(slow_wallet::unlocked_amount(@0x456) == alice_to_bob_transfer_amount, 735703);
    assert!(slow_wallet::transferred_amount(@0x456) == 0, 735704);

    // transfer 10 of bob's unlocked coins to carol
    let bob_to_carol_transfer_amount = 10;
    ol_account::transfer(&bob, @0x789, bob_to_carol_transfer_amount);
    let b_balance = coin::balance<LibraCoin>(@0x789);
    assert!(b_balance == bob_to_carol_transfer_amount, 735704);
    assert!(slow_wallet::unlocked_amount(@0x456) == (alice_to_bob_transfer_amount - bob_to_carol_transfer_amount), 735705);
    assert!(slow_wallet::unlocked_amount(@0x789) == bob_to_carol_transfer_amount, 735706);

    // bob should show a slow wallet transfer amount equal to sent;
    assert!(slow_wallet::transferred_amount(@0x456) == bob_to_carol_transfer_amount, 735707);
    // carol should show no change
    assert!(slow_wallet::transferred_amount(@0x789) == 0, 735708);

  }

  // scenario: testing trying send more funds than are unlocked
  #[test(root = @ol_framework, alice = @0x123, bob = @0x456)]
  #[expected_failure(abort_code = 196614, location = 0x1::ol_account)]
  fun test_transfer_sad_no_unlocked_coins(root: signer, alice: signer) {
    mock::ol_test_genesis(&root);
    let mint_cap = libra_coin::extract_mint_cap(&root);
    slow_wallet::initialize(&root);
    ol_account::create_account(&root, @0x123);
    slow_wallet::set_slow(&alice);
    assert!(slow_wallet::is_slow(@0x123), 7357000);
    assert!(slow_wallet::unlocked_amount(@0x123) == 0, 735701);

    // fund alice with locked-only coins
    ol_account::deposit_coins(@0x123, coin::test_mint(100, &mint_cap));
    coin::destroy_mint_cap(mint_cap);

    // alice will transfer and create bob's account
    ol_account::transfer(&alice, @0x456, 99);
  }

  #[test(root = @ol_framework)]
  // we are testing that genesis creates the needed struct
  // and a validator creation sets the users account to slow.
  fun slow_wallet_reconfigure (root: signer) {
    let set = mock::genesis_n_vals(&root, 4);
    // mock::ol_initialize_coin(&root);
    let a = vector::borrow(&set, 0);
    assert!(slow_wallet::unlocked_amount(*a) == 0, 735701);
    epoch_boundary::ol_reconfigure_for_test(&root, reconfiguration::get_current_epoch(), block::get_current_block_height())

  }


  #[test(root = @0x1)]
  public entry fun test_human_read(
        root: signer,
    ) {
        let _set = mock::genesis_n_vals(&root, 4);
    mock::ol_initialize_coin_and_fund_vals(&root, 1234567890, false);


        let (integer, decimal) = ol_account::balance_human(@0x1000a);

        assert!(integer == 12, 7357001);
        assert!(decimal == 34567890, 7357002);

    }

}
