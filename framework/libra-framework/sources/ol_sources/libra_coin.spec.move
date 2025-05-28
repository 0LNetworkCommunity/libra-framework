spec ol_framework::libra_coin {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec initialize {
        pragma aborts_if_is_partial;
        let addr = signer::address_of(diem_framework);
        ensures exists<MintCapStore>(addr);
        ensures exists<coin::CoinInfo<LibraCoin>>(addr);
    }

    spec initialize_for_core {
        pragma aborts_if_is_partial;
        let addr = signer::address_of(diem_framework);
        aborts_if addr != @diem_framework;
        aborts_if exists<MintCapStore>(addr);
        aborts_if exists<coin::CoinInfo<LibraCoin>>(addr);
        aborts_if !exists<FinalMint>(@diem_framework) && addr != @diem_framework;
        aborts_if exists<FinalMint>(@diem_framework) && addr != @diem_framework;
    }

    spec genesis_set_final_supply {
        pragma aborts_if_is_partial;
        let addr = signer::address_of(diem_framework);
    }

    spec get_final_supply {
        aborts_if !exists<FinalMint>(@ol_framework);
    }

    spec extract {
        aborts_if coin.value < amount;
    }

    spec supply {
        pragma aborts_if_is_partial;
    }

    spec supply_128 {
        aborts_if !exists<coin::CoinInfo<LibraCoin>>(@diem_framework);
    }

    spec maybe_register {
        pragma aborts_if_is_partial;
    }

    spec mint_to_impl {
        pragma aborts_if_is_partial;
    }

    // spec destroy_mint_cap {
    //     let addr = signer::address_of(diem_framework);
    //     aborts_if addr != @diem_framework;
    //     aborts_if !exists<MintCapStore>(@diem_framework);
    // }

    // Test function,not needed verify.
    spec configure_accounts_for_test {
        pragma verify = false;
    }

    // commit not: deprecated function


    // Only callable in tests and testnets.not needed verify.
    spec delegate_mint_capability {
        pragma verify = false;
    }

    // Only callable in tests and testnets.not needed verify.
    spec claim_mint_capability(account: &signer) {
        pragma aborts_if_is_partial;
        aborts_if !exists<diem_framework::chain_id::ChainId>(@diem_framework);
        aborts_if testnet::is_not_mainnet() == false;
        aborts_if !exists<Delegations>(@core_resources);
    }

    spec find_delegation(addr: address): Option<u64> {
        aborts_if !exists<Delegations>(@core_resources);
    }
}
