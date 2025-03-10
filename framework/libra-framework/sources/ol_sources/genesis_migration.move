///////////////////////////////////////////////////////////////////
// 0L Module
// Genesis Migration
///////////////////////////////////////////////////////////////////
// This module is used in hard upgrade where a new genesis takes place, and which requires migrations.
// on the rust side, vm_geneses/lib.rs is used to call migrate_user function here below.

module ol_framework::genesis_migration {
  use std::signer;
  use std::error;
  use diem_framework::coin;
  use ol_framework::ol_account;
  use ol_framework::validator_universe;
  use ol_framework::libra_coin;
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::pledge_accounts;
  use diem_framework::system_addresses;


  /// the unexpected value of a converted balance
  const EBALANCE_MISMATCH: u64 = 0;
  /// the balance of a genesis validator is higher than expected
  const EGENESIS_BALANCE_TOO_HIGH: u64 = 1;

  #[test_only]
  friend ol_framework::test_migration;

  /// Called by root in genesis to initialize the GAS coin
  public(friend) fun migrate_legacy_user(
      framework_sig: &signer,
      user_sig: &signer,
      auth_key: vector<u8>,
      expected_initial_balance: u64,
  ) {
    system_addresses::assert_diem_framework(framework_sig);

    let user_addr = signer::address_of(user_sig);
    // if not a validator OR operator of a validator, create a new account
    // previously during genesis validator and oper accounts were already created
    if (!is_genesis_val(user_addr)) {
      ol_account::vm_create_account_migration(
        framework_sig,
        user_addr,
        auth_key,
      );
    };

    // mint coins again to migrate balance, and all
    // system tracking of balances
    if (expected_initial_balance == 0) {
      return
    };

    // Genesis validators should not receive ANY coins from MINT during testing, testnet, nor mainnet up to this point.
    // they will receive some from the infra escrow.
    let genesis_balance = libra_coin::balance(user_addr);

    assert!(expected_initial_balance >= genesis_balance, error::invalid_state(EGENESIS_BALANCE_TOO_HIGH));

    let coins_to_mint = expected_initial_balance - genesis_balance;
    let c = coin::vm_mint<LibraCoin>(framework_sig, coins_to_mint);
    ol_account::deposit_coins(user_addr, c);

    let new_balance = libra_coin::balance(user_addr);

    assert!(new_balance == expected_initial_balance, error::invalid_state(EBALANCE_MISMATCH));

  }

  fun is_genesis_val(addr: address): bool {
    // TODO: other checks?
    validator_universe::is_in_universe(addr)
  }

  /// for an uprade using an escrow percent. Only to be called at genesis
  // escrow percent has 6 decimal precision (1m);
  public(friend) fun fork_escrow_init(framework_sig: &signer, user_sig: &signer,
  to_escrow: u64, lifetime_pledged: u64, lifetime_withdrawn: u64) {
    system_addresses::assert_diem_framework(framework_sig);
    let c = coin::vm_mint<LibraCoin>(framework_sig, to_escrow);
    pledge_accounts::migrate_pledge_account(framework_sig, user_sig, @ol_framework, c, lifetime_pledged,
    lifetime_withdrawn);
  }
}
