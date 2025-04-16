#[test_only]
module ol_framework::test_filo_migration {
  use std::signer;
  use diem_framework::account;
  use ol_framework::activity;
  use ol_framework::founder;
  use ol_framework::ol_account;
  use ol_framework::mock;
  use ol_framework::slow_wallet;
  use ol_framework::reauthorization;
  use ol_framework::vouch;

  fun setup_one_v7_account(framework: &signer, bob: &signer) {
    mock::ol_test_genesis(framework);
    // emulate the state of a pre-migration v7 account
    // Founder account in V7 should not have
    // post-migration structs: Founder, Vouch, SlowWallet
    let b_addr = signer::address_of(bob);
    ol_account::test_emulate_v7_account(framework, b_addr);
    // check correct setup
    assert!(account::exists_at(b_addr), 735701);
    assert!(!vouch::is_init(b_addr), 735701);
    assert!(!founder::is_founder(b_addr), 735702);
    assert!(!activity::has_ever_been_touched(b_addr), 735703);
    // setup a v7 account without slow wallet.
    assert!(!slow_wallet::is_slow(b_addr), 735704);
  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// V7 user accounts should not have structs initialized
  fun meta_correct_v7_emulation(framework: &signer, bob: &signer) {
    setup_one_v7_account(framework, bob);
  }

  #[test(framework = @0x1, bob = @0x1000b)]

  /// simulating a transaction validation should not
  /// error out
  fun v7_accept_tx_validation(framework: &signer, bob: &signer) {
    setup_one_v7_account(framework, bob);
    mock::simulate_transaction_validation(bob);

  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// V7 accounts get migrated to Founder on first tx
  fun v7_migrates_lazily_on_tx(framework: &signer, bob: &signer) {
    setup_one_v7_account(framework, bob);

    //////// user sends migration tx ////////
    mock::simulate_transaction_validation(bob);
    // safety check: should not error if called again, lazy init
    mock::simulate_transaction_validation(bob);
    //////// end migration tx ////////

    let b_addr = signer::address_of(bob);
    // now the post-migration state should exist
    assert!(vouch::is_init(b_addr), 735706);
    assert!(founder::is_founder(b_addr), 735707);
    assert!(activity::has_ever_been_touched(b_addr), 735708);
    assert!(slow_wallet::is_slow(b_addr), 735709);
    // however the vouch is not sufficient
    assert!(!founder::has_friends(b_addr), 7357010);

  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// All "founder" accounts should be on equal footing,
  /// restarting the unlocking rate.
  fun v7_test_equal_footing_balance(framework: &signer, bob: &signer) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);


    // emulate the state of a pre-migration v7 account
    // with 1000 coins total, which are all unlocked
    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735701);
    assert!(total == 1000, 735702);

    //////// user sends migration tx ////////
    // The first time the user touches the account with a transaction
    // the migration should happen
    mock::simulate_transaction_validation(bob);
    //////// end migration tx ////////

    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735708);
    assert!(total == 1000, 735709);
  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// V7 accounts (Founder) which WERE previously slow wallets
  /// will not continue unlocking until migration happens.
  fun v7_slow_wallets_should_not_unlock(framework: &signer, bob: &signer) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);

    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735705);
    assert!(total == 1000, 735706);

    let mocked_unlock_amount = 50;
    // Emulate a V7 slow wallet
    slow_wallet::test_set_slow_wallet(framework, bob,
    mocked_unlock_amount, mocked_unlock_amount);
    // check setup is correct
    // now it should be a slow wallet
    assert!(slow_wallet::is_slow(b_addr), 735707);
    let (unlocked, _total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735708);

    // CHECK THAT THE BALANCE DOES NOT SHOW PREVIOUS UNLOCK
    let (unlocked, _total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735709);
    assert!(unlocked != mocked_unlock_amount, 735709);

    let locked_supply_pre = slow_wallet::get_locked_supply();

    // the test:
    // Post V8, yet prior to user's migration,
    // there should be no drip
    slow_wallet::test_epoch_drip(framework, 10);
    let (unlocked_post_epoch, total_post_epoch) = ol_account::balance(b_addr);
    assert!(unlocked_post_epoch == 0, 7357010);
    assert!(unlocked_post_epoch == unlocked, 7357011);
    assert!(total_post_epoch == total, 7357012);

    // finally check that the global locked supply hasn't changed
    let locked_supply_post = slow_wallet::get_locked_supply();
    assert!(locked_supply_pre == locked_supply_post, 7357013);
  }

  #[test(framework = @0x1, marlon = @0x1234, bob = @0x1000b)]
  #[expected_failure(abort_code = 196609, location = 0x1::reauthorization)]
  /// V7 accounts (Founder) should not be able to transfer
  /// unless the migration structs are initialized
  /// NOTE: not sure how it would be possible since that
  /// on the first transaction verification, the migration
  /// should happen lazily.
  fun v7_should_not_transfer_until_migrated(framework: &signer, bob: &signer, marlon: address) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);

    // give bob some coins, unlocked leftover from V7.
    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735705);
    assert!(total == 1000, 735706);

    assert!(!activity::has_ever_been_touched(b_addr), 735707);
    // uses transfer entry function
    ol_account::transfer(bob, marlon, 33);

    // //////// user sends migration tx ////////
    // // The first time the user touches the account with a transaction
    // // the migration should happen
    // simulate_transaction_validation(bob);
    // //////// end migration tx ////////
  }

  #[test(framework = @0x1, marlon = @0x1234, bob = @0x1000b)]
  #[expected_failure(abort_code = 196614, location = 0x1::ol_account)]
  /// can transfer after migration but not if balance is zero (no drips occurred).
  /// This test will have a different error code
  fun v7_would_have_no_unlocked_immediately(framework: &signer, bob: &signer, marlon: address) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);

    // give bob some coins, unlocked leftover from V7.
    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735705);
    assert!(total == 1000, 735706);

    assert!(!activity::has_ever_been_touched(b_addr), 735707);

    //////// user sends migration tx ////////
    // The first time the user touches the account with a transaction
    // the migration should happen
    mock::simulate_transaction_validation(bob);
    founder::test_mock_friendly(framework, bob);
    //////// end migration tx ////////

    // uses transfer entry function
    ol_account::transfer(bob, marlon, 33);
  }

  #[test(framework = @0x1, marlon = @0x1234, bob = @0x1000b)]
  #[expected_failure(abort_code = 196614, location = 0x1::ol_account)]

  /// There will be no epoch drip until a founder account is reauthorized
  fun v7_drip_fails_without_vouches(framework: &signer, bob: &signer, marlon: address) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);

    // give bob some coins, unlocked leftover from V7.
    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735705);
    assert!(total == 1000, 735706);

    assert!(!activity::has_ever_been_touched(b_addr), 735707);
    assert!(!reauthorization::is_v8_authorized(b_addr), 735708);

    //////// user sends migration tx ////////
    // The first time the user touches the account with a transaction
    // the migration should happen
    mock::simulate_transaction_validation(bob);
    //////// end migration tx ////////

    assert!(vouch::is_init(b_addr), 735706);
    assert!(founder::is_founder(b_addr), 735707);
    assert!(activity::has_ever_been_touched(b_addr), 735708);
    assert!(slow_wallet::is_slow(b_addr), 735709);
    // however the vouch is not sufficient
    assert!(!founder::has_friends(b_addr), 7357010);

    slow_wallet::test_epoch_drip(framework, 100);

    founder::test_mock_friendly(framework, bob);
    let (unlocked_post, _total_post) = ol_account::balance(b_addr);
    assert!(unlocked_post == 0, 735706);

    // uses transfer entry function
    ol_account::transfer(bob, marlon, 33);
  }

  /// Once there is an epoch drip and the user was migrated
  /// transfers should work normally
  fun v7_fails_without_vouches(framework: &signer, bob: &signer, marlon: address) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);

    // give bob some coins, unlocked leftover from V7.
    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735705);
    assert!(total == 1000, 735706);

    assert!(!activity::has_ever_been_touched(b_addr), 735707);
    assert!(!reauthorization::is_v8_authorized(b_addr), 735708);

    //////// user sends migration tx ////////
    // The first time the user touches the account with a transaction
    // the migration should happen
    mock::simulate_transaction_validation(bob);
    //////// end migration tx ////////
    slow_wallet::test_epoch_drip(framework, 100);

    // uses transfer entry function
    ol_account::transfer(bob, marlon, 33);
  }

  #[test(framework = @0x1, marlon = @0x1234, bob = @0x1000b)]
  /// Once there is an epoch drip and the user was migrated
  /// transfers should work normally
  fun v7_activate_and_transfer_happy(framework: &signer, bob: &signer, marlon: address) {
    setup_one_v7_account(framework, bob);
    let b_addr = signer::address_of(bob);

    // give bob some coins, unlocked leftover from V7.
    mock::ol_mint_to(framework, b_addr, 1000);
    let (unlocked, total) = ol_account::balance(b_addr);
    assert!(unlocked == 0, 735705);
    assert!(total == 1000, 735706);

    assert!(!activity::has_ever_been_touched(b_addr), 735707);
    assert!(!reauthorization::is_v8_authorized(b_addr), 735708);

    //////// user sends migration tx ////////
    // The first time the user touches the account with a transaction
    // the migration should happen
    mock::simulate_transaction_validation(bob);
    //////// end migration tx ////////

    founder::test_mock_friendly(framework, bob);
    // should now be authorized
    assert!(reauthorization::is_v8_authorized(b_addr), 735708);

    // on the next epoch should resume dripping
    slow_wallet::test_epoch_drip(framework, 100);

    // uses transfer entry function
    ol_account::transfer(bob, marlon, 33);
  }
}
