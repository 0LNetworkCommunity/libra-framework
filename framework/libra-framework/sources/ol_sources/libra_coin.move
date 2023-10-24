// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0
// Original source:
// https://github.com/diem/diem/blob/782b31cb08eeb717ea2b6f3edbf616b13fd4cae8/language/move-lang/stdlib/modules/libra_coin.move
// Original Commit:
// https://github.com/diem/diem/commit/782b31cb08eeb717ea2b6f3edbf616b13fd4cae8

module 0x1::LibraCoin {
    use std::signer;

    // A representing the Libra coin
    // The value of the coin. May be zero
    struct LibraCoin has store { value: u64 }

    // A singleton that grants access to `LibraCoin.mint`. Only the Association has one.
    struct MintCapability has store, key {}

    // The sum of the values of all LibraCoin resources in the system
    struct MarketCap has store, key { total_value: u64 }

    // Return a reference to the MintCapability published under the sender's account. Fails if the
    // sender does not have a MintCapability.
    // Since only the Association account has a mint capability, this will only succeed if it is
    // invoked by a transaction sent by that account.
    public fun mint_with_default_capability(sender: &signer, amount: u64): LibraCoin acquires MintCapability, MarketCap {
        mint(amount, borrow_global<MintCapability>(signer::address_of(sender)))
    }

    // Mint a new LibraCoin worth `value`. The caller must have a reference to a MintCapability.
    // Only the Association account can acquire such a reference, and it can do so only via
    // `borrow_sender_mint_capability`
    public fun mint(value: u64, _capability: &MintCapability): LibraCoin acquires MarketCap {
        // TODO: temporary measure for testnet only: limit minting to 1B Libra at a time.
        // this is to prevent the market cap's total value from hitting u64_max due to excessive
        // minting. This will not be a problem in the production Libra system because coins will
        // be backed with real-world assets, and thus minting will be correspondingly rarer.
        // * 1000000 because the unit is microlibra
        assert!(value <= 1000000000 * 1000000, 11);

        // update market cap to reflect minting
        let market_cap = borrow_global_mut<MarketCap>(@0xA550C18);
        market_cap.total_value = market_cap.total_value + value;

        LibraCoin { value }
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize(sender: &signer) {
        // Only callable by the Association address
        assert!(signer::address_of(sender) == @0xA550C18, 1);

        move_to(sender, MintCapability{});
        move_to(sender, MarketCap { total_value: 0 });
    }

    // Return the total value of all Libra in the system
    public fun market_cap(): u64 acquires MarketCap {
        borrow_global<MarketCap>(@0xA550C18).total_value
    }

    // Create a new LibraCoin with a value of 0
    public fun zero(): LibraCoin {
        LibraCoin { value: 0 }
    }

    // Public accessor for the value of a coin
    public fun value(coin: &LibraCoin): u64 {
        coin.value
    }

    // Splits the given coin into two and returns them both
    // It leverages `Self.withdraw` for any verifications of the values
    public fun split(coin: &mut LibraCoin, amount: u64): LibraCoin {
        let other = withdraw(coin, amount);
        other
    }

    // "Divides" the given coin into two, where original coin is modified in place
    // The original coin will have value = original value - `amount`
    // The new coin will have a value = `amount`
    // Fails if the coins value is less than `amount`
    public fun withdraw(coin: &mut LibraCoin, amount: u64): LibraCoin {
        // Check that `amount` is less than the coin's value
        assert!(coin.value >= amount, 10);
        // Split the coin
        coin.value = value(coin) - amount;
        LibraCoin { value: amount }
    }

    // Merges two coins and returns a new coin whose value is equal to the sum of the two inputs
    public fun join(coin1: &mut LibraCoin, coin2: LibraCoin)  {
        deposit(coin1, coin2);
    }

    // "Merges" the two coins
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit(coin: &mut LibraCoin, check: LibraCoin) {
        let LibraCoin { value } = check;
        coin.value = value(coin) + value
    }

    // Destroy a coin
    // Fails if the value is non-zero
    // The amount of LibraCoin in the system is a tightly controlled property,
    // so you cannot "burn" any non-zero amount of LibraCoin
    public fun destroy_zero(coin: LibraCoin) {
        let LibraCoin { value } = coin;
        assert!(value == 0, 11)
    }

}


module ol_framework::gas_coin {
    use std::string;
    use std::error;
    use std::signer;
    use std::vector;
    use std::option::{Self, Option};

    use diem_framework::coin::{Self, MintCapability, BurnCapability};
    use diem_framework::system_addresses;

    use ol_framework::globals;

    friend diem_framework::genesis;
    friend ol_framework::genesis_migration;

    /// Account does not have mint capability
    const ENO_CAPABILITIES: u64 = 1;
    /// Mint capability has already been delegated to this specified address
    const EALREADY_DELEGATED: u64 = 2;
    /// Cannot find delegation of mint capability to this account
    const EDELEGATION_NOT_FOUND: u64 = 3;

    struct LibraCoin has key {}

    struct FinalSupply has key {
      value: u64,
    }

    struct MintCapStore has key {
        mint_cap: MintCapability<LibraCoin>,
    }

    /// Delegation token created by delegator and can be claimed by the delegatee as MintCapability.
    struct DelegatedMintCapability has store {
        to: address
    }

    /// The container stores the current pending delegations.
    struct Delegations has key {
        inner: vector<DelegatedMintCapability>,
    }

    /// Can only called during genesis to initialize the Diem coin.
    public(friend) fun initialize(diem_framework: &signer) {
        system_addresses::assert_diem_framework(diem_framework);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<LibraCoin>(
            diem_framework,
            string::utf8(b"LibraCoin"),
            string::utf8(b"LIBRA"),
            globals::get_coin_decimal_places(), /* decimals  MATCHES LEGACY 0L */
            true, /* monitor_supply */
        );

        // Diem framework needs mint cap to mint coins to initial validators. This will be revoked once the validators
        // have been initialized.
        move_to(diem_framework, MintCapStore { mint_cap });

        coin::destroy_freeze_cap(freeze_cap);
        coin::destroy_burn_cap(burn_cap);
        // (burn_cap, mint_cap)
    }

    /// FOR TESTS ONLY
    /// Can only called during genesis to initialize the Diem coin.
    public(friend) fun initialize_for_core(diem_framework: &signer): (BurnCapability<LibraCoin>, MintCapability<LibraCoin>)  {
        system_addresses::assert_diem_framework(diem_framework);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<LibraCoin>(
            diem_framework,
            string::utf8(b"LibraCoin"),
            string::utf8(b"LIBRA"),
            globals::get_coin_decimal_places(), /* decimals  MATCHES LEGACY 0L */
            true, /* monitor_supply */
        );

        // Diem framework needs mint cap to mint coins to initial validators. This will be revoked once the validators
        // have been initialized.
        move_to(diem_framework, MintCapStore { mint_cap });

        coin::destroy_freeze_cap(freeze_cap);

        (burn_cap, mint_cap)
    }

    public fun has_mint_capability(account: &signer): bool {
        exists<MintCapStore>(signer::address_of(account))
    }

    /// Only called during genesis to destroy the diem framework account's mint capability once all initial validators
    /// and accounts have been initialized during genesis.
    public(friend) fun destroy_mint_cap(diem_framework: &signer) acquires MintCapStore {
        system_addresses::assert_diem_framework(diem_framework);
        let MintCapStore { mint_cap } = move_from<MintCapStore>(@diem_framework);
        coin::destroy_mint_cap(mint_cap);
    }

    // at genesis we need to init the final supply
    // done at genesis_migration
    public(friend) fun genesis_set_final_supply(diem_framework: &signer,
    final_supply: u64) {
      if (!exists<FinalSupply>(@ol_framework)) {
        move_to(diem_framework, FinalSupply {
          value: final_supply
        });
      }
    }

    #[view]
    /// get the original final supply from genesis
    public fun get_final_supply(): u64 acquires FinalSupply{
      borrow_global<FinalSupply>(@ol_framework).value
    }


    #[view]
    /// get the gas coin supply. Helper which wraps coin::supply and extracts option type
    // NOTE: there is casting between u128 and u64, but 0L has final supply below the u64.
    public fun supply(): u64 {
      let supply_opt = coin::supply<LibraCoin>();
      if (option::is_some(&supply_opt)) {
        return (*option::borrow(&supply_opt) as u64)
      };
      0
    }


    #[test_only]
    public fun restore_mint_cap(diem_framework: &signer, mint_cap: MintCapability<LibraCoin>) {
        system_addresses::assert_diem_framework(diem_framework);
        move_to(diem_framework, MintCapStore { mint_cap });
    }

    /// FOR TESTS ONLY
    /// The `core addresses` sudo account is used to execute system transactions for testing
    /// Can only be called during genesis for tests to grant mint capability to diem framework and core resources
    /// accounts.
    public(friend) fun configure_accounts_for_test(
        diem_framework: &signer,
        core_resources: &signer,
        mint_cap: MintCapability<LibraCoin>,
    ){
        system_addresses::assert_diem_framework(diem_framework);

        // Mint the core resource account LibraCoin for gas so it can execute system transactions.
        coin::register<LibraCoin>(core_resources);

        let coins = coin::mint<LibraCoin>(
            18446744073709551615,
            &mint_cap,
        );
        coin::deposit<LibraCoin>(signer::address_of(core_resources), coins);

        move_to(core_resources, MintCapStore { mint_cap });
        move_to(core_resources, Delegations { inner: vector::empty() });
    }

    // /// Only callable in tests and testnets where the core resources account exists.
    // /// Create new coins and deposit them into dst_addr's account.
    // mint_impl(
    //     root: &signer,
    //     amount: u64,
    // ): Coin<LibraCoin> acquires MintCapStore {
    //     system_addresses::assert_ol(root);

    //     let mint_cap = &borrow_global<MintCapStore>(signer::address_of(root)).mint_cap;
    //     coin::mint<LibraCoin>(amount, mint_cap)
    // }

    // NOTE: needed for smoke tests
    // TODO: guard some other way besides using the testing root account.
    /// Root account can mint to an address. Only used for genesis and tests.
    /// The "root" account in smoke tests has some privileges.
    public entry fun mint_to_impl(
        root: &signer,
        dst_addr: address,
        amount: u64,
    ) acquires MintCapStore {

        let account_addr = signer::address_of(root);

        assert!(
            exists<MintCapStore>(account_addr),
            error::not_found(ENO_CAPABILITIES),
        );

        let mint_cap = &borrow_global<MintCapStore>(account_addr).mint_cap;
        let coins_minted = coin::mint<LibraCoin>(amount, mint_cap);
        coin::deposit<LibraCoin>(dst_addr, coins_minted);
    }

    #[test_only]
    public entry fun test_mint_to(
        root: &signer,
        dst_addr: address,
        amount: u64,
    ) acquires MintCapStore {
      system_addresses::assert_ol(root);
      mint_to_impl(root, dst_addr, amount);
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Create delegated token for the address so the account could claim MintCapability later.
    public entry fun delegate_mint_capability(account: signer, to: address) acquires Delegations {
        system_addresses::assert_core_resource(&account);
        let delegations = &mut borrow_global_mut<Delegations>(@core_resources).inner;
        let i = 0;
        while (i < vector::length(delegations)) {
            let element = vector::borrow(delegations, i);
            assert!(element.to != to, error::invalid_argument(EALREADY_DELEGATED));
            i = i + 1;
        };
        vector::push_back(delegations, DelegatedMintCapability { to });
    }

    /// Only callable in tests and testnets where the core resources account exists.
    /// Claim the delegated mint capability and destroy the delegated token.
    public entry fun claim_mint_capability(account: &signer) acquires Delegations, MintCapStore {
        let maybe_index = find_delegation(signer::address_of(account));
        assert!(option::is_some(&maybe_index), EDELEGATION_NOT_FOUND);
        let idx = *option::borrow(&maybe_index);
        let delegations = &mut borrow_global_mut<Delegations>(@core_resources).inner;
        let DelegatedMintCapability { to: _ } = vector::swap_remove(delegations, idx);

        // Make a copy of mint cap and give it to the specified account.
        let mint_cap = borrow_global<MintCapStore>(@core_resources).mint_cap;
        move_to(account, MintCapStore { mint_cap });
    }

    fun find_delegation(addr: address): Option<u64> acquires Delegations {
        let delegations = &borrow_global<Delegations>(@core_resources).inner;
        let i = 0;
        let len = vector::length(delegations);
        let index = option::none();
        while (i < len) {
            let element = vector::borrow(delegations, i);
            if (element.to == addr) {
                index = option::some(i);
                break
            };
            i = i + 1;
        };
        index
    }

    #[view]
    /// helper to get balance in gas coin
    public fun get_balance(account: address): u64 {
        coin::balance<LibraCoin>(account)
    }


    #[test_only]
    use diem_framework::aggregator_factory;

    #[test_only]
    public fun initialize_for_test(diem_framework: &signer): (BurnCapability<LibraCoin>, MintCapability<LibraCoin>) {
        aggregator_factory::initialize_aggregator_factory_for_test(diem_framework);
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<LibraCoin>(
            diem_framework,
            string::utf8(b"LibraCoin"),
            string::utf8(b"LIBRA"),
            8, /* decimals */
            true, /* monitor_supply */
        );
        move_to(diem_framework, MintCapStore { mint_cap });

        coin::destroy_freeze_cap(freeze_cap);
        (burn_cap, mint_cap)
    }

    // This is particularly useful if the aggregator_factory is already initialized via another call path.
    #[test_only]
    public fun initialize_for_test_without_aggregator_factory(diem_framework: &signer): (BurnCapability<LibraCoin>, MintCapability<LibraCoin>) {
                let (burn_cap, freeze_cap, mint_cap) = coin::initialize_with_parallelizable_supply<LibraCoin>(
            diem_framework,
            string::utf8(b"LibraCoin"),
            string::utf8(b"LIBRA"),
            8, /* decimals */
            true, /* monitor_supply */
        );
        coin::destroy_freeze_cap(freeze_cap);
        (burn_cap, mint_cap)
    }
}
