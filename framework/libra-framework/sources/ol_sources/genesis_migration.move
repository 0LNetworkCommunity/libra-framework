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
  use ol_framework::gas_coin;
  use ol_framework::gas_coin::GasCoin;
  use ol_framework::transaction_fee;
  use diem_framework::system_addresses;
  // use diem_std::debug::print;

  /// the unexpected value of a converted balance
  const EBALANCE_MISMATCH: u64 = 0;
  /// the balance of a genesis validator is higher than expected
  const EGENESIS_BALANCE_TOO_HIGH: u64 = 1;
  /// balances converted incorrectly, more coins created than the target
  const EMINTED_OVER_TARGET: u64 = 2;

  /// Called by root in genesis to initialize the GAS coin
  public fun migrate_legacy_user(
      vm: &signer,
      user_sig: &signer,
      auth_key: vector<u8>,
      expected_initial_balance: u64,
  ) {
    system_addresses::assert_diem_framework(vm);

    let user_addr = signer::address_of(user_sig);
    // if not a validator OR operator of a validator, create a new account
    // previously during genesis validator and oper accounts were already created
    if (!is_genesis_val(user_addr)) {
      ol_account::vm_create_account_migration(
        vm,
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
    let genesis_balance = coin::balance<GasCoin>(user_addr);

    assert!(expected_initial_balance >= genesis_balance, error::invalid_state(EGENESIS_BALANCE_TOO_HIGH));

    let coins_to_mint = expected_initial_balance - genesis_balance;
    let c = coin::vm_mint<GasCoin>(vm, coins_to_mint);
    coin::deposit<GasCoin>(user_addr, c);

    let new_balance = coin::balance<GasCoin>(user_addr);

    assert!(new_balance == expected_initial_balance, error::invalid_state(EBALANCE_MISMATCH));

  }

  fun is_genesis_val(addr: address): bool {
    // TODO: other checks?
    validator_universe::is_in_universe(addr)
  }

  fun rounding_mint(root: &signer, target_supply: u64) {
    let existing_supply = gas_coin::supply();

    assert!(target_supply >= existing_supply, error::invalid_state(EMINTED_OVER_TARGET));
    if (target_supply > existing_supply) {
        let coin = coin::vm_mint<GasCoin>(root, target_supply - existing_supply);
        transaction_fee::vm_pay_fee(root, @ol_framework, coin);

    }
  }
}