/// This module provides an interface to burn or collect and redistribute transaction fees.
module diem_framework::transaction_fee {
    use diem_framework::coin::{Self, AggregatableCoin, BurnCapability, Coin};
    use diem_framework::system_addresses;
    use std::error;
    use std::vector;
    use std::option::{Self, Option};
    use ol_framework::libra_coin::LibraCoin;
    use ol_framework::fee_maker;
    use ol_framework::slow_wallet;
    use ol_framework::ol_account;

    // use diem_std::debug::print;


    friend diem_framework::block;
    friend diem_framework::genesis;
    friend diem_framework::reconfiguration;
    friend diem_framework::transaction_validation;

    friend ol_framework::epoch_boundary;
    friend ol_framework::burn;

    /// Gas fees are already being collected and the struct holding
    /// information about collected amounts is already published.
    const EALREADY_COLLECTING_FEES: u64 = 1;

    /// The burn percentage is out of range [0, 100].
    const EINVALID_BURN_PERCENTAGE: u64 = 3;

    /// Stores burn capability to burn the gas fees.
    struct GasCoinCapabilities has key {
        burn_cap: BurnCapability<LibraCoin>,
    }

    /// Stores information about the block proposer and the amount of fees
    /// collected when executing the block.
    struct CollectedFeesPerBlock has key {
        amount: AggregatableCoin<LibraCoin>,
        proposer: Option<address>,
        burn_percentage: u8,
    }

    /// Initializes the resource storing information about gas fees collection and
    /// distribution. Should be called by on-chain governance.
    public fun initialize_fee_collection_and_distribution(diem_framework: &signer, burn_percentage: u8) {
        system_addresses::assert_diem_framework(diem_framework);
        assert!(
            !exists<CollectedFeesPerBlock>(@diem_framework),
            error::already_exists(EALREADY_COLLECTING_FEES)
        );
        assert!(burn_percentage <= 100, error::out_of_range(EINVALID_BURN_PERCENTAGE));

        // Make sure stakng module is aware of transaction fees collection.
        // stake::initialize_validator_fees(diem_framework);

        // Initially, no fees are collected and the block proposer is not set.
        let collected_fees = CollectedFeesPerBlock {
            amount: coin::initialize_aggregatable_coin(diem_framework),
            proposer: option::none(),
            burn_percentage,
        };
        move_to(diem_framework, collected_fees);
    }

    public fun is_fees_collection_enabled(): bool {
        exists<CollectedFeesPerBlock>(@diem_framework)
    }

    // /// Sets the burn percentage for collected fees to a new value. Should be called by on-chain governance.
    // public fun upgrade_burn_percentage(
    //     diem_framework: &signer,
    //     new_burn_percentage: u8
    // ) acquires GasCoinCapabilities, CollectedFeesPerBlock {
    //     system_addresses::assert_diem_framework(diem_framework);
    //     assert!(new_burn_percentage <= 100, error::out_of_range(EINVALID_BURN_PERCENTAGE));

    //     // Prior to upgrading the burn percentage, make sure to process collected
    //     // fees. Otherwise we would use the new (incorrect) burn_percentage when
    //     // processing fees later!
    //     process_collected_fees();

    //     if (is_fees_collection_enabled()) {
    //         // Upgrade has no effect unless fees are being collected.
    //         let burn_percentage = &mut borrow_global_mut<CollectedFeesPerBlock>(@diem_framework).burn_percentage;
    //         *burn_percentage = new_burn_percentage
    //     }
    // }

    /// Registers the proposer of the block for gas fees collection. This function
    /// can only be called at the beginning of the block.
    public(friend) fun register_proposer_for_fee_collection(proposer_addr: address) acquires CollectedFeesPerBlock {
        if (is_fees_collection_enabled()) {
            let collected_fees = borrow_global_mut<CollectedFeesPerBlock>(@diem_framework);
            let _ = option::swap_or_fill(&mut collected_fees.proposer, proposer_addr);
        }
    }

    // /// Burns a specified fraction of the coin.
    // fun burn_coin_fraction(coin: &mut Coin<LibraCoin>, burn_percentage: u8) acquires GasCoinCapabilities {
    //     assert!(burn_percentage <= 100, error::out_of_range(EINVALID_BURN_PERCENTAGE));

    //     let collected_amount = coin::value(coin);
    //     spec {
    //         // We assume that `burn_percentage * collected_amount` does not overflow.
    //         assume burn_percentage * collected_amount <= MAX_U64;
    //     };
    //     let amount_to_burn = (burn_percentage as u64) * collected_amount / 100;
    //     if (amount_to_burn > 0) {
    //         let coin_to_burn = coin::extract(coin, amount_to_burn);
    //         coin::burn(
    //             coin_to_burn,
    //             &borrow_global<GasCoinCapabilities>(@diem_framework).burn_cap,
    //         );
    //     }
    // }

    // /// Calculates the fee which should be distributed to the block proposer at the
    // /// end of an epoch, and records it in the system. This function can only be called
    // /// at the beginning of the block or during reconfiguration.
    // public(friend) fun process_collected_fees() acquires GasCoinCapabilities, CollectedFeesPerBlock {
    //     if (!is_fees_collection_enabled()) {
    //         return
    //     };
    //     let collected_fees = borrow_global_mut<CollectedFeesPerBlock>(@diem_framework);

    //     // If there are no collected fees, only unset the proposer. See the rationale for
    //     // setting proposer to option::none() below.
    //     if (coin::is_aggregatable_coin_zero(&collected_fees.amount)) {
    //         if (option::is_some(&collected_fees.proposer)) {
    //             let _ = option::extract(&mut collected_fees.proposer);
    //         };
    //         return
    //     };

    //     // Otherwise get the collected fee, and check if it can distributed later.
    //     let coin = coin::drain_aggregatable_coin(&mut collected_fees.amount);
    //     // if (option::is_some(&collected_fees.proposer)) {
    //     //     // Extract the address of proposer here and reset it to option::none(). This
    //     //     // is particularly useful to avoid any undesired side-effects where coins are
    //     //     // collected but never distributed or distributed to the wrong account.
    //     //     // With this design, processing collected fees enforces that all fees will be burnt
    //     //     // unless the proposer is specified in the block prologue. When we have a governance
    //     //     // proposal that triggers reconfiguration, we distribute pending fees and burn the
    //     //     // fee for the proposal. Otherwise, that fee would be leaked to the next block.
    //     //     let proposer = option::extract(&mut collected_fees.proposer);

    //     //     // Since the block can be produced by the VM itself, we have to make sure we catch
    //     //     // this case.
    //     //     if (proposer == @vm_reserved) {
    //     //         burn_coin_fraction(&mut coin, 100);
    //     //         coin::destroy_zero(coin);
    //     //         return
    //     //     };

    //     //     burn_coin_fraction(&mut coin, collected_fees.burn_percentage);
    //     //     // coin::burn()
    //     //     // stake::add_transaction_fee(proposer, coin);
    //     //     return
    //     // };

    //     // If checks did not pass, simply burn all collected coins and return none.
    //     burn_coin_fraction(&mut coin, 100);
    //     coin::destroy_zero(coin)
    // }

    // /// Burn transaction fees in epilogue.
    // public(friend) fun burn_fee(account: address, fee: u64) acquires GasCoinCapabilities {
    //     coin::burn_from<LibraCoin>(
    //         account,
    //         fee,
    //         &borrow_global<GasCoinCapabilities>(@diem_framework).burn_cap,
    //     );
    // }

    /// Collect transaction fees in epilogue.
    public(friend) fun collect_fee(account: address, fee: u64) acquires CollectedFeesPerBlock {
        let collected_fees = borrow_global_mut<CollectedFeesPerBlock>(@diem_framework);

        // Here, we are always optimistic and always collect fees. If the proposer is not set,
        // or we cannot redistribute fees later for some reason (e.g. account cannot receive AptoCoin)
        // we burn them all at once. This way we avoid having a check for every transaction epilogue.
        let collected_amount = &mut collected_fees.amount;
        coin::collect_into_aggregatable_coin<LibraCoin>(account, fee, collected_amount);
        // must adjust slow wallet tracker for all transactions
        slow_wallet::maybe_track_unlocked_withdraw(account, fee);
    }

    //////// 0L ////////
    /// a user can pay a fee directly
    public fun user_pay_fee(sender: &signer, fee: Coin<LibraCoin>) acquires CollectedFeesPerBlock {
        // Need to track who is making payments to Fee Maker
        fee_maker::track_user_fee(sender, coin::value(&fee));
        pay_fee_impl(fee);
    }

    /// root account will pay a fee on behalf of a user account.
    // if VM is not going to track the tx it will just add a system address here
    public fun vm_pay_fee(sender: &signer, account: address, fee: Coin<LibraCoin>) acquires CollectedFeesPerBlock {
      // Need to track who is making payments to Fee Maker
      // don't track system transfers into transaction fee
      if (!system_addresses::is_framework_reserved_address(account)) {
        fee_maker::vm_track_user_fee(sender, account, coin::value(&fee));
      };
      pay_fee_impl(fee);
    }

    /// implementation
    fun pay_fee_impl(fee: Coin<LibraCoin>) acquires CollectedFeesPerBlock {

        let collected_fees = borrow_global_mut<CollectedFeesPerBlock>(@diem_framework);

        // Here, we are always optimistic and always collect fees. If the proposer is not set,
        // or we cannot redistribute fees later for some reason (e.g. account cannot receive AptoCoin)
        // we burn them all at once. This way we avoid having a check for every transaction epilogue.
        let collected_amount = &mut collected_fees.amount;
        coin::merge_aggregatable_coin<LibraCoin>(collected_amount, fee);
    }

    #[view]
    /// get the total system fees available now.
    public fun system_fees_collected(): u64 acquires CollectedFeesPerBlock {
      let collected_fees = borrow_global<CollectedFeesPerBlock>(@diem_framework);
      (coin::aggregatable_value(&collected_fees.amount) as u64)
    }

    /// root account can use system fees to pay multiple accounts, e.g. for Proof of Fee reward.
    /// returns the actual amount that was transferred
    public fun vm_multi_collect(vm: &signer, list: &vector<address>, amount: u64): u64  acquires CollectedFeesPerBlock {
      system_addresses::assert_ol(vm);
      if (amount == 0) return 0;
      let actual_transferred = 0;

      let i = 0;

      while (i < vector::length(list)) {
        let from = vector::borrow(list, i);
        let coin_option = ol_account::vm_withdraw_unlimited(vm, *from, amount);
        if (option::is_some(&coin_option)) {
          let c = option::extract(&mut coin_option);
          actual_transferred = actual_transferred + coin::value(&c);
          vm_pay_fee(vm, *from, c)
        };
        option::destroy_none(coin_option);

        i = i + 1;
      };
      actual_transferred
    }

    /////// 0L ////////
    /// withdraw from system transaction account.
    // belt and suspenders
    public(friend) fun root_withdraw_all(root: &signer): Coin<LibraCoin> acquires CollectedFeesPerBlock {
      system_addresses::assert_ol(root);
      withdraw_all_impl(root)
    }

    // belt and suspenders implementation
    fun withdraw_all_impl(root: &signer): Coin<LibraCoin> acquires CollectedFeesPerBlock {
      system_addresses::assert_ol(root);

      let collected_fees = borrow_global_mut<CollectedFeesPerBlock>(@diem_framework);

      coin::drain_aggregatable_coin<LibraCoin>(&mut collected_fees.amount)
    }


    /// Only called during genesis.
    public(friend) fun store_diem_coin_burn_cap(diem_framework: &signer, burn_cap: BurnCapability<LibraCoin>) {
        system_addresses::assert_diem_framework(diem_framework);
        move_to(diem_framework, GasCoinCapabilities { burn_cap })
    }

    #[test_only]
    public fun test_root_withdraw_all(root: &signer): Coin<LibraCoin> acquires CollectedFeesPerBlock {
      system_addresses::assert_ol(root);
      withdraw_all_impl(root)
    }

    #[test_only]
    use diem_framework::aggregator_factory;

    #[test(diem_framework = @diem_framework)]
    fun test_initialize_fee_collection_and_distribution(diem_framework: signer) acquires CollectedFeesPerBlock {
        aggregator_factory::initialize_aggregator_factory_for_test(&diem_framework);
        initialize_fee_collection_and_distribution(&diem_framework, 25);

        // Check struct has been published.
        assert!(exists<CollectedFeesPerBlock>(@diem_framework), 0);

        // Check that initial balance is 0 and there is no proposer set.
        let collected_fees = borrow_global<CollectedFeesPerBlock>(@diem_framework);
        assert!(coin::is_aggregatable_coin_zero(&collected_fees.amount), 0);
        assert!(option::is_none(&collected_fees.proposer), 0);
        assert!(collected_fees.burn_percentage == 25, 0);
    }

    // #[test(diem_framework = @diem_framework)]
    // fun test_burn_fraction_calculation(diem_framework: signer) acquires GasCoinCapabilities {
    //     use ol_framework::libra_coin;
    //     let (burn_cap, mint_cap) = libra_coin::initialize_for_test(&diem_framework);
    //     store_diem_coin_burn_cap(&diem_framework, burn_cap);

    //     let c1 = coin::mint<LibraCoin>(100, &mint_cap);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 100, 0);

    //     // Burning 25%.
    //     burn_coin_fraction(&mut c1, 25);
    //     assert!(coin::value(&c1) == 75, 0);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 75, 0);

    //     // Burning 0%.
    //     burn_coin_fraction(&mut c1, 0);
    //     assert!(coin::value(&c1) == 75, 0);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 75, 0);

    //     // Burning remaining 100%.
    //     burn_coin_fraction(&mut c1, 100);
    //     assert!(coin::value(&c1) == 0, 0);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 0, 0);

    //     coin::destroy_zero(c1);
    //     let c2 = coin::mint<LibraCoin>(10, &mint_cap);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 10, 0);

    //     burn_coin_fraction(&mut c2, 5);
    //     assert!(coin::value(&c2) == 10, 0);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 10, 0);

    //     burn_coin_fraction(&mut c2, 100);
    //     coin::destroy_zero(c2);
    //     coin::destroy_burn_cap(burn_cap);
    //     coin::destroy_mint_cap(mint_cap);
    // }

    // #[test(root = @ol_framework, alice = @0xa11ce, bob = @0xb0b, carol = @0xca101)]
    // fun test_fees_distribution(
    //     root: signer,
    //     alice: signer,
    //     bob: signer,
    //     carol: signer,
    // ) acquires GasCoinCapabilities, CollectedFeesPerBlock {
    //     //////// 0L ////////
    //     // changed to use LibraCoin for fees
    //     use std::signer;
    //     use ol_framework::ol_account;
    //     use ol_framework::libra_coin;

    //     // Initialization.
    //     let (burn_cap, mint_cap) = libra_coin::initialize_for_test(&root);
    //     libra_coin::test_set_final_supply(&root, 100);
    //     store_diem_coin_burn_cap(&root, burn_cap);
    //     initialize_fee_collection_and_distribution(&root, 10);

    //     // Create dummy accounts.
    //     let alice_addr = signer::address_of(&alice);
    //     let bob_addr = signer::address_of(&bob);
    //     let carol_addr = signer::address_of(&carol);
    //     ol_account::create_account(&root, alice_addr);
    //     ol_account::create_account(&root, bob_addr);
    //     ol_account::create_account(&root, carol_addr);
    //     coin::deposit(alice_addr, coin::mint(10000, &mint_cap));
    //     coin::deposit(bob_addr, coin::mint(10000, &mint_cap));
    //     coin::deposit(carol_addr, coin::mint(10000, &mint_cap));
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 30000, 0);

    //     // Block 1 starts.
    //     process_collected_fees();
    //     register_proposer_for_fee_collection(alice_addr);

    //     // Check that there was no fees distribution in the first block.
    //     let collected_fees = borrow_global<CollectedFeesPerBlock>(@ol_framework);
    //     assert!(coin::is_aggregatable_coin_zero(&collected_fees.amount), 0);
    //     assert!(*option::borrow(&collected_fees.proposer) == alice_addr, 0);
    //     assert!(*option::borrow(&coin::supply<LibraCoin>()) == 30000, 0);

    //     // Simulate transaction fee collection - here we simply collect some fees from Bob.
    //     collect_fee(bob_addr, 100);
    //     collect_fee(bob_addr, 500);
    //     collect_fee(bob_addr, 400);

    //     // Now Bob must have 1000 less in his account. Alice and Carol have the same amounts.
    //     assert!(coin::balance<LibraCoin>(alice_addr) == 10000, 0);
    //     assert!(coin::balance<LibraCoin>(bob_addr) == 9000, 0);
    //     assert!(coin::balance<LibraCoin>(carol_addr) == 10000, 0);

    //     // Block 2 starts.
    //     process_collected_fees();
    //     register_proposer_for_fee_collection(bob_addr);

    //     // Collected fees from Bob must have been assigned to Alice.
    //     // assert!(stake::get_validator_fee(alice_addr) == 900, 0);
    //     assert!(coin::balance<LibraCoin>(alice_addr) == 10000, 0);
    //     assert!(coin::balance<LibraCoin>(bob_addr) == 9000, 0);
    //     assert!(coin::balance<LibraCoin>(carol_addr) == 10000, 0);

    //     // Also, aggregator coin is drained and total supply is slightly changed (10% of 1000 is burnt).
    //     let collected_fees = borrow_global<CollectedFeesPerBlock>(@ol_framework);
    //     assert!(coin::is_aggregatable_coin_zero(&collected_fees.amount), 0);
    //     assert!(*option::borrow(&collected_fees.proposer) == bob_addr, 0);
    //     // assert!(*option::borrow(&coin::supply<LibraCoin>()) == 29900, 0);

    //     // Simulate transaction fee collection one more time.
    //     collect_fee(bob_addr, 5000);
    //     collect_fee(bob_addr, 4000);

    //     assert!(coin::balance<LibraCoin>(alice_addr) == 10000, 0);
    //     assert!(coin::balance<LibraCoin>(bob_addr) == 0, 0);
    //     assert!(coin::balance<LibraCoin>(carol_addr) == 10000, 0);

    //     // Block 3 starts.
    //     process_collected_fees();
    //     register_proposer_for_fee_collection(carol_addr);

    //     // Collected fees should have been assigned to Bob because he was the peoposer.
    //     // assert!(stake::get_validator_fee(alice_addr) == 900, 0);
    //     assert!(coin::balance<LibraCoin>(alice_addr) == 10000, 0);
    //     // assert!(stake::get_validator_fee(bob_addr) == 8100, 0);
    //     assert!(coin::balance<LibraCoin>(bob_addr) == 0, 0);
    //     assert!(coin::balance<LibraCoin>(carol_addr) == 10000, 0);

    //     // Again, aggregator coin is drained and total supply is changed by 10% of 9000.
    //     let collected_fees = borrow_global<CollectedFeesPerBlock>(@diem_framework);
    //     assert!(coin::is_aggregatable_coin_zero(&collected_fees.amount), 0);
    //     assert!(*option::borrow(&collected_fees.proposer) == carol_addr, 0);
    //     // assert!(*option::borrow(&coin::supply<LibraCoin>()) == 29000, 0);

    //     // Simulate transaction fee collection one last time.
    //     collect_fee(alice_addr, 1000);
    //     collect_fee(alice_addr, 1000);

    //     // Block 4 starts.
    //     process_collected_fees();
    //     register_proposer_for_fee_collection(alice_addr);

    //     // Check that 2000 was collected from Alice.
    //     assert!(coin::balance<LibraCoin>(alice_addr) == 8000, 0);
    //     assert!(coin::balance<LibraCoin>(bob_addr) == 0, 0);

    //     // Carol must have some fees assigned now.
    //     let collected_fees = borrow_global<CollectedFeesPerBlock>(@diem_framework);
    //     // assert!(stake::get_validator_fee(carol_addr) == 1800, 0);
    //     assert!(coin::is_aggregatable_coin_zero(&collected_fees.amount), 0);
    //     assert!(*option::borrow(&collected_fees.proposer) == alice_addr, 0);
    //     // assert!(*option::borrow(&coin::supply<LibraCoin>()) == 28800, 0);

    //     coin::destroy_burn_cap(burn_cap);
    //     coin::destroy_mint_cap(mint_cap);
    // }
}
