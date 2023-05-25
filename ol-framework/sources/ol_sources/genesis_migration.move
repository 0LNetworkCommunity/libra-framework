///////////////////////////////////////////////////////////////////
// 0L Module
// Genesis Migration
///////////////////////////////////////////////////////////////////
// This module is used in hard upgrade where a new genesis takes place, and which requires migrations.
// on the rust side, vm_geneses/lib.rs is used to call migrate_user function here below.

module ol_framework::genesis_migration {
  use std::signer;
  use std::error;
  use aptos_framework::coin;
  use ol_framework::ol_account;
  use ol_framework::globals;
  use ol_framework::validator_universe;
  use ol_framework::gas_coin;
  use ol_framework::gas_coin::GasCoin;

  const EBALANCE_MISMATCH: u64 = 0;

  const VAL_ESCROW_PCT: u64 = 80;
  /// Called by root in genesis to initialize the GAS coin 
  public fun fork_migrate_account(
      vm: &signer,
      user_sig: &signer,
      // user_addr: address,
      auth_key: vector<u8>,
      balance: u64,
      is_validator: bool,
  ) {
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
    if (balance == 0) {
      return
    };
    // scale up by the coin split factor
    let new_balance = globals::get_coin_split_factor() * balance;

    gas_coin::mint(vm, user_addr, new_balance);

    let balance = coin::balance<GasCoin>(user_addr);
    assert!(balance == new_balance, error::invalid_state(EBALANCE_MISMATCH));

    // establish the infrastructure escrow pledge
    if (is_validator) {
      let _share = (balance * VAL_ESCROW_PCT) / 100;
      // TODO: deposit
      // InfraEscrow::user_pledge_infra(user_sig, share)
    };
  }

  fun is_genesis_val(addr: address): bool {
    // TODO: other checks?
    validator_universe::is_in_universe(addr)
  }
}