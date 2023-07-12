#[test_only]
module ol_framework::test_migration {
  use ol_framework::genesis_migration;
  use ol_framework::infra_escrow;
  use ol_framework::slow_wallet;
  use aptos_framework::genesis;
  use std::fixed_point32;
  // use std::vector;
  use ol_framework::mock;
  use aptos_framework::coin;
  use ol_framework::gas_coin::GasCoin;
  use std::signer;
  use std::bcs;

  // use aptos_std::debug::print;

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
      false,
      5,
      800000,
    )
  }

  // test for infra escrow contribution
  #[test(root = @ol_framework, alice = @0x1000a)]
  fun test_migration_balance_validators(root: signer, alice: signer) {

    let _vals = mock::genesis_n_vals(&root, 4);
    genesis::test_initalize_coins(&root);

    // let alice = vector::borrow(&vals, 0);
    let alice_addr = signer::address_of(&alice);
    let temp_auth_key =  bcs::to_bytes(&alice_addr);

    let legacy_balance = 20000;
    let scale_integer = 5;
    let escrow_pct = 80;
    genesis_migration::migrate_legacy_user(
      &root,
      &alice,
      temp_auth_key,
      legacy_balance,
      true, // is_validator
      scale_integer * 1000000,// of 1m
      escrow_pct * 10000, // of 1m
    );

    let user_balance = coin::balance<GasCoin>(alice_addr);

    assert!(user_balance == (legacy_balance * scale_integer), 73570000);

    // now we migrate validators

    // alice only has 1000 unlocked, and 100 lifetime transferred out
    let legacy_unlocked = 1000;
    slow_wallet::fork_migrate_slow_wallet(&root, &alice, legacy_unlocked, 100, scale_integer * 1000000);
    let (unlocked, total) = slow_wallet::balance(alice_addr);
    assert!(unlocked == (legacy_unlocked * scale_integer), 73570001);
    assert!(total == user_balance, 73570002);


    // after the slow wallets have been calculated we check the infra escrow pledges
    infra_escrow::fork_escrow_init(&root, &alice, 800000);

    let user_pledge = infra_escrow::user_infra_pledge_balance(alice_addr);
    let all_pledge_balance = infra_escrow::infra_escrow_balance();

    let pct = fixed_point32::create_from_rational(escrow_pct, 100);
    let locked = total-unlocked;
    let expected_pledge = fixed_point32::multiply_u64(locked, pct);
    assert!(user_pledge == expected_pledge, 73570003);


    assert!(all_pledge_balance > 0, 73570004);

    assert!(all_pledge_balance == user_pledge, 73570005);

    let updated_balance = coin::balance<GasCoin>(alice_addr);

    assert!((updated_balance + user_pledge) == legacy_balance * scale_integer, 73570006);
  }
}