///////////////////////////////////////////////////////////////////
// 0L Module
// Genesis Migration
///////////////////////////////////////////////////////////////////
// This module is used in hard upgrade where a new genesis takes place, and which requires migrations.
// on the rust side, vm_geneses/lib.rs is used to call migrate_user function here below.

module ol_framework::genesis_migration {
  use std::signer;
  use std::error;
  use std::option;
  use diem_framework::coin;
  use ol_framework::ol_account;
  use ol_framework::validator_universe;
  use ol_framework::libra_coin;
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::transaction_fee;
  use ol_framework::pledge_accounts;
  use ol_framework::make_whole;
  use diem_framework::system_addresses;
  // use diem_std::debug::print;

  /// the unexpected value of a converted balance
  const EBALANCE_MISMATCH: u64 = 0;
  /// the balance of a genesis validator is higher than expected
  const EGENESIS_BALANCE_TOO_HIGH: u64 = 1;
  /// balances converted incorrectly, more coins created than the target
  const EMINTED_OVER_TARGET: u64 = 2;
  /// not sufficient infra escrow balance
  const ENO_INFRA_BALANCE: u64 = 3;
  /// makewhole is not initialized
  const EMAKEWHOLE_NOT_INIT: u64 = 4;

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
    let genesis_balance = coin::balance<LibraCoin>(user_addr);

    assert!(expected_initial_balance >= genesis_balance, error::invalid_state(EGENESIS_BALANCE_TOO_HIGH));

    let coins_to_mint = expected_initial_balance - genesis_balance;
    let c = coin::vm_mint<LibraCoin>(vm, coins_to_mint);
    ol_account::deposit_coins(user_addr, c);

    let new_balance = coin::balance<LibraCoin>(user_addr);

    assert!(new_balance == expected_initial_balance, error::invalid_state(EBALANCE_MISMATCH));

  }

  fun is_genesis_val(addr: address): bool {
    // TODO: other checks?
    validator_universe::is_in_universe(addr)
  }

  fun rounding_mint(root: &signer, target_supply: u64) {
    let existing_supply = libra_coin::supply();

    // we should not ever have migrated more coins than expected
    // this should abort the genesis process
    assert!(existing_supply <= target_supply,
    error::invalid_state(EMINTED_OVER_TARGET));

    if (target_supply > existing_supply) {
        let coin = coin::vm_mint<LibraCoin>(root, target_supply - existing_supply);
        transaction_fee::vm_pay_fee(root, @ol_framework, coin);
    };
  }

    /// for an uprade using an escrow percent. Only to be called at genesis
  // escrow percent has 6 decimal precision (1m);
  public fun fork_escrow_init(vm: &signer, user_sig: &signer, to_escrow: u64) {
    system_addresses::assert_vm(vm);
    let user_addr = signer::address_of(user_sig);
    // shouldn't be tracking slow wallets at this point, do a direct withdraw
    let coin_opt = coin::vm_withdraw<LibraCoin>(vm, user_addr, to_escrow);
    if (option::is_some(&coin_opt)) {
      let c = option::extract(&mut coin_opt);
      pledge_accounts::save_pledge(user_sig, @0x0, c);
    };
    option::destroy_none(coin_opt);
  }

  //////// MAKE WHOLE INIT ////////
  struct MinerMathError has key {}

  /// initializes the Miner Math Error incident make-whole
  // note: this exits silently when there's no infra_escrow, since some tests
  // don't need it
  fun init_make_whole(vm: &signer, make_whole_budget: u64) {
    system_addresses::assert_ol(vm);
    // withdraw from infraescrow
    let opt = pledge_accounts::withdraw_from_all_pledge_accounts(vm,
    make_whole_budget);
    if (option::is_none(&opt)) {
      option::destroy_none(opt);
        return // exit quietly
    } else {
      let coin = option::extract(&mut opt);
      option::destroy_none(opt);

      let burns_unclaimed = true;
      make_whole::init_incident<MinerMathError>(vm, coin, burns_unclaimed);
    }
  }

  /// creates an individual claim for a user
  // note: this exits silently when there's no infra_escrow, since some tests
  // don't need it
  fun vm_create_credit_user(vm: &signer, user: address, value: u64) {
    system_addresses::assert_ol(vm);
    if (!make_whole::is_init<MinerMathError>(signer::address_of(vm))) return;
    make_whole::create_each_user_credit<MinerMathError>(vm, user, value);

  }
}
