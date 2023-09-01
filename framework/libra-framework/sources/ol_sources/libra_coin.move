// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0
// Original source:
// https://github.com/diem/diem/blob/782b31cb08eeb717ea2b6f3edbf616b13fd4cae8/language/move-lang/stdlib/modules/libra_coin.move
// Original Commit:
// https://github.com/diem/diem/commit/782b31cb08eeb717ea2b6f3edbf616b13fd4cae8

module 0x0::LibraCoin {
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
        value(coin)
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