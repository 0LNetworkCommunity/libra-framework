///////////////////////////////////////////////////////////////////
// 0L Module
// Genesis Migration
///////////////////////////////////////////////////////////////////
// This module is used in hard upgrade where a new genesis takes place, and which requires migrations.
// on the rust side, vm_geneses/lib.rs is used to call migrate_user function here below.

module ol_framework::genesis_migration {
  use std::signer;
  use std::error;
  // use std::fixed_point32;
  use aptos_framework::coin;
  use ol_framework::ol_account;
  // use ol_framework::globals;
  use ol_framework::validator_universe;
  use ol_framework::gas_coin;
  use ol_framework::gas_coin::GasCoin;
  use ol_framework::infra_escrow;
  use aptos_framework::system_addresses;

  const EBALANCE_MISMATCH: u64 = 0;

  // const VAL_ESCROW_PCT: u64 = 80;


  /// Called by root in genesis to initialize the GAS coin 
  public fun migrate_legacy_user(
      vm: &signer,
      user_sig: &signer,
      auth_key: vector<u8>,
      legacy_balance: u64,
      is_validator: bool,
      split_factor: u64,
      escrow_pct: u64, // precision of 1,000,000
  ) {
    system_addresses::assert_aptos_framework(vm);

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
    if (legacy_balance == 0) {
      return
    };
    let genesis_balance = coin::balance<GasCoin>(user_addr);

    // scale up by the coin split factor
    let expected_final_balance = split_factor * legacy_balance;
    let coins_to_mint = expected_final_balance - genesis_balance;
    gas_coin::mint(vm, user_addr, coins_to_mint);

    // TODO: only mint the delta of the expected balance vs what is in the account.
    let new_balance = coin::balance<GasCoin>(user_addr);

    assert!(new_balance == expected_final_balance, error::invalid_state(EBALANCE_MISMATCH));

    // establish the infrastructure escrow pledge
    if (is_validator) {
      let to_escrow = (escrow_pct * new_balance) / 1000000;
      infra_escrow::user_pledge_infra(user_sig, to_escrow)
    };
  }

  fun is_genesis_val(addr: address): bool {
    // TODO: other checks?
    validator_universe::is_in_universe(addr)
  }
}