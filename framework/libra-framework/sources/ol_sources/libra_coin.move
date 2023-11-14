// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0
// Original source:
// https://github.com/diem/diem/blob/782b31cb08eeb717ea2b6f3edbf616b13fd4cae8/language/move-lang/stdlib/modules/libra_coin.move
// Original Commit:
// https://github.com/diem/diem/commit/782b31cb08eeb717ea2b6f3edbf616b13fd4cae8


module ol_framework::libra_coin {
    use std::string;
    use std::error;
    use std::signer;
    use std::vector;
    use std::option::{Self, Option};
    // use diem_std::debug::print;

    use diem_framework::coin::{Self, MintCapability, BurnCapability};
    use diem_framework::system_addresses;

    use ol_framework::globals;

    friend diem_framework::genesis;
    friend ol_framework::genesis_migration;

    const MAX_U64: u128 = 18446744073709551615;

    /// Account does not have mint capability
    const ENO_CAPABILITIES: u64 = 1;
    /// Mint capability has already been delegated to this specified address
    const EALREADY_DELEGATED: u64 = 2;
    /// Cannot find delegation of mint capability to this account
    const EDELEGATION_NOT_FOUND: u64 = 3;
    /// Supply somehow above MAX_U64
    const ESUPPLY_OVERFLOW: u64 = 4;

    struct LibraCoin has key { /* new games for society */}

    struct FinalMint has key {
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
    public(friend) fun initialize(diem_framework: &signer) acquires FinalMint {
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

        genesis_set_final_supply(diem_framework, 1000)
    }

    /// FOR TESTS ONLY
    /// Can only called during genesis to initialize the Diem coin.
    public(friend) fun initialize_for_core(diem_framework: &signer):
    (BurnCapability<LibraCoin>, MintCapability<LibraCoin>) acquires FinalMint {
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

        genesis_set_final_supply(diem_framework, 100); // TODO: set this number
        // in testnets

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
    fun genesis_set_final_supply(diem_framework: &signer,
    final_supply: u64) acquires FinalMint {
      system_addresses::assert_ol(diem_framework);

      if (!exists<FinalMint>(@ol_framework)) {
        move_to(diem_framework, FinalMint {
          value: final_supply
        });
      } else {
        let state = borrow_global_mut<FinalMint>(@ol_framework);
        state.value = final_supply
      }
    }
    #[test_only]
    public fun test_set_final_supply(diem_framework: &signer,
    final_supply: u64) acquires FinalMint {
      system_addresses::assert_ol(diem_framework);

      if (!exists<FinalMint>(@ol_framework)) {
        move_to(diem_framework, FinalMint {
          value: final_supply
        });
      } else {
        let state = borrow_global_mut<FinalMint>(@ol_framework);
        state.value = final_supply
      }
    }

    #[view]
    /// get the original final supply from genesis
    public fun get_final_supply(): u64 acquires FinalMint{
      borrow_global<FinalMint>(@ol_framework).value
    }


    #[view]
    /// get the gas coin supply. Helper which wraps coin::supply and extracts option type
    // NOTE: there is casting between u128 and u64, but 0L has final supply below the u64.
    public fun supply(): u64 {

      let supply_opt = coin::supply<LibraCoin>();
      if (option::is_some(&supply_opt)) {
        let value = *option::borrow(&supply_opt);
        spec {
          assume value < MAX_U64;
        };
        return (value as u64)
      };
      0
    }
    #[view]
    /// debugging view
    public fun supply_128(): u128 {
      let supply_opt = coin::supply<LibraCoin>();
      if (option::is_some(&supply_opt)) {
        return *option::borrow(&supply_opt)
      };
      0
    }


    #[test_only]
    public fun restore_mint_cap(diem_framework: &signer, mint_cap: MintCapability<LibraCoin>) {
        system_addresses::assert_diem_framework(diem_framework);
        move_to(diem_framework, MintCapStore { mint_cap });
    }

    #[test_only]
    public fun extract_mint_cap(diem_framework: &signer):
    MintCapability<LibraCoin> acquires MintCapStore {
        system_addresses::assert_diem_framework(diem_framework);
        let MintCapStore { mint_cap } = move_from<MintCapStore>(@diem_framework);
        mint_cap
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
            1000000 * 1000000, // core resources can have 1M coins, MAX_U64 was
            // causing arthmetic errors calling supply() on downcast
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
        let _s = supply(); // check we didn't overflow supply

        let account_addr = signer::address_of(root);

        assert!(
            exists<MintCapStore>(account_addr),
            error::not_found(ENO_CAPABILITIES),
        );

        let mint_cap = &borrow_global<MintCapStore>(account_addr).mint_cap;
        let coins_minted = coin::mint<LibraCoin>(amount, mint_cap);
        coin::deposit<LibraCoin>(dst_addr, coins_minted);

        // TODO: update the final supply for tests
        // genesis_set_final_supply(root, supply());
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

    // #[view]
    // /// helper to get balance in gas coin
    // public fun get_balance(account: address): u64 {
    //     coin::balance<LibraCoin>(account)
    // }


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
