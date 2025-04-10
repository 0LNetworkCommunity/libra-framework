#[test_only]
module ol_framework::test_migration {
  use ol_framework::genesis_migration;
  use ol_framework::mock;
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
}
