#[test_only]
module ol_framework::test_migration {
  use ol_framework::genesis_migration;
  use ol_framework::infra_escrow;
  use ol_framework::slow_wallet;
  use diem_framework::genesis;
  use std::fixed_point32;
  use ol_framework::mock;
  use diem_framework::coin;
  use ol_framework::gas_coin::GasCoin;
  use std::signer;
  use std::bcs;

  // use diem_std::debug::print;

  #[test(root = @ol_framework, user = @0xaaaaaa)]
  fun test_migration_balance_end_user(root: signer, user: signer) {
    let temp_auth_key = bcs::to_bytes(&signer::address_of(&user));

    mock::genesis_n_vals(&root, 4);
    genesis::test_initalize_coins(&root);

    genesis_migration::migrate_legacy_user(
      &root,
      &user,
      temp_auth_key,
      20000,
      // 5,
    )
  }

  // test for infra escrow contribution
  #[test(root = @ol_framework, vm = @vm_reserved, marlon_rando = @0x123456)]
  fun test_migration_balance_validators(root: signer, vm: signer, marlon_rando: signer) {

    let _vals = mock::genesis_n_vals(&root, 4);
    // mock::ol_initialize_coin_and_fund_vals(&root, 1000000);
    genesis::test_initalize_coins(&root);

    // let alice = vector::borrow(&vals, 0);
    let addr = signer::address_of(&marlon_rando);
    let temp_auth_key =  bcs::to_bytes(&addr);

    let init_balance = 100000; // note: the scaling happens on RUST side.
    let escrow_pct = 80;

    genesis_migration::migrate_legacy_user(
      &root,
      &marlon_rando,
      temp_auth_key,
      init_balance,
      // true, // is_validator
      // scale_integer * 1000000,// of 1m
      // escrow_pct * 10000, // of 1m
    );

    let user_balance = coin::balance<GasCoin>(addr);

    assert!(user_balance == init_balance, 73570000);

    // now we migrate validators

    // alice only has 1000 unlocked, and 100 lifetime transferred out
    let legacy_unlocked = 1000;
    slow_wallet::fork_migrate_slow_wallet(&root, &marlon_rando, legacy_unlocked, 100);
    let (unlocked, total) = slow_wallet::balance(addr);
    assert!(unlocked == legacy_unlocked, 73570001);
    assert!(total == user_balance, 73570002);


    // after the slow wallets have been calculated we check the infra escrow pledges
    genesis_migration::fork_escrow_init(&vm, &marlon_rando, escrow_pct * 10000);

    let user_pledge = infra_escrow::user_infra_pledge_balance(addr);
    let all_pledge_balance = infra_escrow::infra_escrow_balance();

    let pct = fixed_point32::create_from_rational(escrow_pct, 100);
    let locked = total-unlocked;
    let expected_pledge = fixed_point32::multiply_u64(locked, pct);
    assert!(user_pledge == expected_pledge, 73570003);


    assert!(all_pledge_balance > 0, 73570004);

    assert!(all_pledge_balance == user_pledge, 73570005);

    let updated_balance = coin::balance<GasCoin>(addr);

    assert!((updated_balance + user_pledge) == init_balance, 73570006);
  }
}