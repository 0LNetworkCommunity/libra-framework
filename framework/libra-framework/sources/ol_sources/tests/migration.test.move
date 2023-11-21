#[test_only]
module ol_framework::test_migration {
  use ol_framework::genesis_migration;
  use ol_framework::infra_escrow;
  use ol_framework::slow_wallet;
  use ol_framework::ol_account;
  use ol_framework::mock;
  use diem_framework::coin;
  use ol_framework::libra_coin::LibraCoin;
  use std::signer;
  use std::bcs;

  #[test(root = @ol_framework, user = @0xaaaaaa)]
  fun test_migration_balance_end_user(root: signer, user: signer) {
    let temp_auth_key = bcs::to_bytes(&signer::address_of(&user));

    mock::genesis_n_vals(&root, 4);

    genesis_migration::migrate_legacy_user(
      &root,
      &user,
      temp_auth_key,
      20000,
    )
  }

  // test for infra escrow contribution
  #[test(root = @ol_framework, vm = @vm_reserved, marlon_rando = @0x123456)]
  fun test_migration_balance_validators(root: signer, vm: signer, marlon_rando: signer) {

    let _vals = mock::genesis_n_vals(&root, 4);

    let addr = signer::address_of(&marlon_rando);
    let temp_auth_key =  bcs::to_bytes(&addr);

    let init_balance = 100000; // note: the scaling happens on RUST side.

    genesis_migration::migrate_legacy_user(
      &root,
      &marlon_rando,
      temp_auth_key,
      init_balance,
    );

    let user_balance = coin::balance<LibraCoin>(addr);

    assert!(user_balance == init_balance, 73570000);

    // now we migrate validators

    // alice only has 1000 unlocked, and 100 lifetime transferred out
    let legacy_unlocked = 1000;
    slow_wallet::fork_migrate_slow_wallet(&root, &marlon_rando, legacy_unlocked, 100);
    let (unlocked, total) = ol_account::balance(addr);
    assert!(unlocked == legacy_unlocked, 73570001);
    assert!(total == user_balance, 73570002);


    // after the slow wallets have been calculated we check the infra escrow / pledges
    // coins to take to escrow
    let expected_pledge = 10000;

    genesis_migration::fork_escrow_init(&vm, &marlon_rando, expected_pledge);

    let user_pledge = infra_escrow::user_infra_pledge_balance(addr);
    let all_pledge_balance = infra_escrow::infra_escrow_balance();

    assert!(user_pledge == expected_pledge, 73570003);


    assert!(all_pledge_balance > 0, 73570004);

    assert!(all_pledge_balance == user_pledge, 73570005);

    let updated_balance = coin::balance<LibraCoin>(addr);

    assert!((updated_balance + user_pledge) == init_balance, 73570006);
  }
}
