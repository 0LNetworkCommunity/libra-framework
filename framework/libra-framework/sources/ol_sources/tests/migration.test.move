#[test_only]
module ol_framework::test_migration {
  use ol_framework::genesis_migration;
  use ol_framework::infra_escrow;
  use aptos_framework::genesis;
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

    genesis_migration::migrate_legacy_user(
      &root,
      &alice,
      temp_auth_key,
      20000,
      true, // is_validator
    );

    let pledge = infra_escrow::user_infra_pledge_balance(alice_addr);
    let balance = infra_escrow::infra_escrow_balance();
    let user_balance = coin::balance<GasCoin>(alice_addr);

    assert!(pledge > 0, 73570001);
    assert!(balance == pledge, 73570001);

    assert!((user_balance + pledge) == user_balance * 5, 73570002);
  }
}