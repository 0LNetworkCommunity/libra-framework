module ol_framework::ol_account {
    use diem_framework::account::{Self, new_event_handle, WithdrawCapability};
    use diem_framework::coin::{Self, Coin};
    use diem_framework::event::{EventHandle, emit_event};
    use diem_framework::system_addresses;
    // use diem_framework::chain_status;
    use std::error;
    use std::signer;
    use std::option::{Self, Option};
    use diem_std::from_bcs;

    use ol_framework::gas_coin::{Self, LibraCoin as GasCoin};
    use ol_framework::slow_wallet;
    use ol_framework::receipts;
    use ol_framework::cumulative_deposits;

    // use diem_std::debug::print;

    #[test_only]
    use std::vector;

    friend ol_framework::donor_directed;
    friend ol_framework::burn;
    friend ol_framework::safe;
    friend diem_framework::genesis;
    friend diem_framework::resource_account;

    /// Account does not exist.
    const EACCOUNT_NOT_FOUND: u64 = 1;
    /// Account is not registered to receive GAS.
    const EACCOUNT_NOT_REGISTERED_FOR_GAS: u64 = 2;
    /// Account opted out of receiving coins that they did not register to receive.
    const EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS: u64 = 3;
    /// Account opted out of directly receiving NFT tokens.
    const EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS: u64 = 4;
    /// The lengths of the recipients and amounts lists don't match.
    const EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH: u64 = 5;

    /// not enough unlocked coins to transfer
    const EINSUFFICIENT_BALANCE: u64 = 6;

    /// On legacy account migration we need to check if we rotated auth keys correctly and can find the user address.
    const ECANT_MATCH_ADDRESS_IN_LOOKUP: u64 = 7;


    struct BurnTracker has key {
      prev_supply: u64,
      prev_balance: u64,
      burn_at_last_calc: u64,
      cumu_burn: u64,
      // percent: u64,
      // percent_increase: u64,
    }


    /// Configuration for whether an account can receive direct transfers of coins that they have not registered.
    ///
    /// By default, this is enabled. Users can opt-out by disabling at any time.
    struct DirectTransferConfig has key {
        allow_arbitrary_coin_transfers: bool,
        update_coin_transfer_events: EventHandle<DirectCoinTransferConfigUpdatedEvent>,
    }

    /// Event emitted when an account's direct coins transfer config is updated.
    struct DirectCoinTransferConfigUpdatedEvent has drop, store {
        new_allow_direct_transfers: bool,
    }


    /// A wrapper to create a resource account and register it to receive GAS.
    public fun ol_create_resource_account(user: &signer, seed: vector<u8>): (signer, account::SignerCapability) {
      let (resource_account_sig, cap) = account::create_resource_account(user, seed);
      coin::register<GasCoin>(&resource_account_sig);
      (resource_account_sig, cap)
    }

    // /// Creates an account by sending an initial amount of GAS to it.
    // public entry fun create_user_account_by_coin(sender: &signer, auth_key: address, amount: u64) {
    //     // warn early before attempting to creat the account.
    //     let limit = slow_wallet::unlocked_amount(signer::address_of(sender));
    //     assert!(amount < limit, error::invalid_state(EINSUFFICIENT_BALANCE));

    //     create_impl(auth_key);
    //     // use the proper tracking
    //     transfer(sender, auth_key, amount);
    // }

    fun create_impl(auth_key: address) {
        let new_signer = account::create_account(auth_key);
        coin::register<GasCoin>(&new_signer);
        receipts::user_init(&new_signer);
        init_burn_tracker(&new_signer);
    }

    // #[test_only]
    /// Helper for tests to create acounts
    /// Belt and suspenders
    public entry fun create_account(root: &signer, auth_key: address) {
        system_addresses::assert_ol(root);
        create_impl(auth_key);
    }

    /// For migrating accounts from a legacy system
    /// NOTE: the legacy accounts (prefixed with 32 zeros) from 0L v5 will not be found by searching via authkey. Since the legacy authkey does not derive to the legcy account any longer, it is as if the account has rotated the authkey.
    /// The remedy is to run the authkey rotation
    /// even if it hasn't changed, such that the lookup table (OriginatingAddress) is created and populated with legacy accounts.
    public fun vm_create_account_migration(
        root: &signer,
        new_account: address,
        auth_key: vector<u8>,
        // value: u64,
    ): signer {
        system_addresses::assert_ol(root);
        // chain_status::assert_genesis(); TODO
        let new_signer = account::vm_create_account(root, new_account, auth_key);
        // fake "rotate" legacy auth key  to itself so that the lookup is populated
        account::vm_migrate_rotate_authentication_key_internal(root, &new_signer, auth_key);
        // check we can in fact look up the account
        let auth_key_as_address = from_bcs::to_address(auth_key);
        let lookup_addr = account::get_originating_address(auth_key_as_address);
        assert!(
          lookup_addr == signer::address_of(&new_signer),
          error::invalid_state(ECANT_MATCH_ADDRESS_IN_LOOKUP)
        );

        coin::register<GasCoin>(&new_signer);
        init_burn_tracker(&new_signer);
        new_signer
    }




    #[test_only]
    /// Batch version of GAS transfer.
    public entry fun batch_transfer(source: &signer, recipients:
    vector<address>, amounts: vector<u64>) acquires BurnTracker {
        let recipients_len = vector::length(&recipients);
        assert!(
            recipients_len == vector::length(&amounts),
            error::invalid_argument(EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH),
        );

        let i = 0;
        while (i < recipients_len) {
            let to = *vector::borrow(&recipients, i);
            let amount = *vector::borrow(&amounts, i);
            transfer(source, to, amount);
            i = i + 1;
        };
    }

    /// Convenient function to transfer GAS to a recipient account that might not exist.
    /// This would create the recipient account first, which also registers it to receive GAS, before transferring.
    public entry fun transfer(sender: &signer, to: address, amount: u64)
    acquires BurnTracker {
      let payer = signer::address_of(sender);
      transfer_checks(payer, to, amount);
      // both update burn tracker
      let c = withdraw(sender, amount);
      deposit_coins(to, c);
      slow_wallet::maybe_track_slow_transfer(payer, to, amount);
    }

    // transfer with capability, and do appropriate checks on both sides, and track the slow wallet
    public fun transfer_with_capability(cap: &WithdrawCapability, recipient:
    address, amount: u64) acquires BurnTracker {
      let payer = account::get_withdraw_cap_address(cap);
      transfer_checks(payer, recipient, amount);
      // NOTE: these shoud update BurnTracker
      let c = withdraw_with_capability(cap, amount);
      deposit_coins(recipient, c);
      slow_wallet::maybe_track_slow_transfer(payer, recipient, amount);
    }

    /// Withdraw a coin while tracking the unlocked withdraw
    public fun withdraw_with_capability(cap: &WithdrawCapability, amount: u64):
    Coin<GasCoin> acquires BurnTracker {
      let payer = account::get_withdraw_cap_address(cap);
      let limit = slow_wallet::unlocked_amount(payer);
      assert!(amount < limit, error::invalid_state(EINSUFFICIENT_BALANCE));

      slow_wallet::maybe_track_unlocked_withdraw(payer, amount);
      // the outgoing coins should trigger an update on this account
      maybe_update_burn_tracker_impl(payer);
      coin::withdraw_with_capability(cap, amount)
    }

    /// Withdraw funds while respecting the transfer limits
    public fun withdraw(sender: &signer, amount: u64): Coin<GasCoin> acquires
    BurnTracker {
        let addr = signer::address_of(sender);
        let limit = slow_wallet::unlocked_amount(addr);
        assert!(amount < limit, error::invalid_state(EINSUFFICIENT_BALANCE));
        slow_wallet::maybe_track_unlocked_withdraw(addr, amount);
        // the outgoing coins should trigger an update on this account
        maybe_update_burn_tracker_impl(addr);
        coin::withdraw<GasCoin>(sender, amount)
    }

    // actual implementation to allow for capability
    fun transfer_checks(payer: address, recipient: address, amount: u64) {
        let limit = slow_wallet::unlocked_amount(payer);
        assert!(amount < limit, error::invalid_state(EINSUFFICIENT_BALANCE));

        if (!account::exists_at(recipient)) {
            // creates the account address (with the same bytes as the authentication key).
            create_impl(recipient);
            // return
        };


        // TODO: Check if Resource Accounts can register here, since they
        // may be created without any coin registration.
        assert!(coin::is_account_registered<GasCoin>(recipient), error::invalid_argument(EACCOUNT_NOT_REGISTERED_FOR_GAS));

        // must track the slow wallet on both sides of the transfer
        slow_wallet::maybe_track_slow_transfer(payer, recipient, amount);

        // maybe track cumulative deposits if this is a donor directed wallet
        // or other wallet which tracks cumulative payments.
        cumulative_deposits::maybe_update_deposit(payer, recipient, amount);
    }



    /// vm can transfer between account to settle.
    /// THIS FUNCTION CAN BYPASS SLOW WALLET WITHDRAW RESTRICTIONS
    /// this is used at epoch boundary operations when the vm signer is available.
    /// returns the actual amount transferred, and whether that amount was the
    /// whole amount expected to transfer.
    /// (amount_transferred, success)
    public(friend) fun vm_transfer(vm: &signer, from: address, to: address, amount: u64): (u64, bool) acquires
    BurnTracker {
      system_addresses::assert_ol(vm);
      let amount_transferred = 0;
      // should not halt
      if (!coin::is_account_registered<GasCoin>(from)) return (0, false);
      if (!coin::is_account_registered<GasCoin>(to)) return (0, false);

      if(amount > coin::balance<GasCoin>(from)) return (0, false);

      let coin_option = coin::vm_withdraw<GasCoin>(vm, from, amount);

      if (option::is_some(&coin_option)) {
        let c = option::extract(&mut coin_option);
        amount_transferred = coin::value(&c);
        coin::deposit(to, c); // TODO: this should use internal functions to
        // deduplicate what follows
        // update both accounts
        maybe_update_burn_tracker_impl(from);
        maybe_update_burn_tracker_impl(to);

      };

      option::destroy_none(coin_option);

      // transfers which use VM authority (e.g. donor directed accounts)
      // should also track the recipient's slow wallet unlock counter.
      slow_wallet::maybe_track_slow_transfer(from, to, amount_transferred);

      // how much was actually extracted, and was that equal to the amount expected
      (amount_transferred, amount_transferred == amount)
    }

    #[test_only]
    public fun test_vm_withdraw(vm: &signer, from: address, amount: u64):
    Option<Coin<GasCoin>> acquires BurnTracker {
      system_addresses::assert_ol(vm);
      // should not halt
      if (!coin::is_account_registered<GasCoin>(from)) return option::none();
      if(amount > coin::balance<GasCoin>(from)) return option::none();

      maybe_update_burn_tracker_impl(from);
      coin::vm_withdraw<GasCoin>(vm, from, amount)

    }
    /// vm can transfer between account to settle.
    /// THIS FUNCTION CAN BYPASS SLOW WALLET WITHDRAW RESTRICTIONS
    /// used to withdraw and track the withdrawal
    public(friend) fun vm_withdraw_unlimited(vm: &signer, from: address, amount:
    u64): Option<Coin<GasCoin>> acquires
    BurnTracker {
      system_addresses::assert_ol(vm);
      // should not halt
      if(amount > coin::balance<GasCoin>(from)) return option::none();

      // since the VM can withdraw more than what is unlocked
      // it needs to adjust the unlocked amount, which may end up zero
      // if it goes over the limit
      let c_opt = coin::vm_withdraw<GasCoin>(vm, from, amount);

      // we're not always sure what's in the option
      if (option::is_some(&c_opt)) {
        let coin = option::borrow(&c_opt);
        let value = coin::value<GasCoin>(coin);
        if (value > 0) {
          maybe_update_burn_tracker_impl(from);
          slow_wallet::maybe_track_unlocked_withdraw(from, value);
        }
      };

      return c_opt

    }

    // public fun withdraw_with_capability(cap: &WithdrawCapability, amount: u64): Coin<GasCoin> {
    //   coin::withdraw_with_capability(cap, amount)
    // }

    //////// 0L ////////

    #[view]
    /// return the GasCoin balance as tuple (unlocked, total)
    // TODO v7: consolidate balance checks here, not in account, slow_wallet, or coin
    public fun balance(addr: address): (u64, u64) {
      slow_wallet::balance(addr)
    }

    // on new account creation we need the burn tracker created
    // note return quietly if it's already initialized, so we can use it
    // in the creation and tx flow
    public fun init_burn_tracker(sig: &signer) {
      let addr = signer::address_of(sig);
      if (exists<BurnTracker>(addr)) return;

      let (_, total_balance) = balance(addr);
      move_to(sig, BurnTracker {
        prev_supply: gas_coin::supply(),
        prev_balance: total_balance,
        burn_at_last_calc: 0,
        cumu_burn: 0,
      })
    }


  /// TODO: the user may update the tracker outside of transactions
  public fun user_update_burn_tracker() {}

  // NOTE: this must be called before immediately after any coins are deposited or withrdrawn.
  fun maybe_update_burn_tracker_impl(addr: address) acquires BurnTracker {
    if (!exists<BurnTracker>(addr)) return;// return quietly as the VM may call this

    let state = borrow_global_mut<BurnTracker>(addr);
    // 1. how much burn happened in between
    // this must be true but we
    // don't abort since the VM may be calling this
    let current_supply = gas_coin::supply();
    let original_supply = gas_coin::get_final_supply();
    if (original_supply > current_supply) {
      let burn_in_period = original_supply - current_supply;

      if (burn_in_period> 0 && burn_in_period > state.prev_balance ) {
        let attributed_burn = burn_in_period / state.prev_balance;
        // attributed burn may be zero because of rounding effects
        // in that case we should skip the updating altogether and
        // only track when the attributable is > 1. Otherwise the
        // whole chain of updates will be incorrect
        if (attributed_burn > 0) {
          state.cumu_burn = state.burn_at_last_calc + attributed_burn;
          // now change last calc
          state.burn_at_last_calc = attributed_burn;
          // reset trackers for next tx
          state.prev_supply = current_supply;

          let (_, total_balance) = balance(addr);
          state.prev_balance = total_balance;
        }
      }
    }
  }

    // TODO:
    // #[test_only]
    // /// Batch version of transfer_coins.
    // public entry fun batch_transfer<CoinType>(
    //     from: &signer, recipients: vector<address>, amounts: vector<u64>) {
    //     let recipients_len = vector::length(&recipients);
    //     assert!(
    //         recipients_len == vector::length(&amounts),
    //         error::invalid_argument(EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH),
    //     );


    /// A coin which is split or extracted can be sent to an account without a sender signing.
    /// TODO: cumulative tracker will not work here.
    public fun deposit_coins(to: address, coins: Coin<GasCoin>) acquires
    BurnTracker {
        assert!(coin::is_account_registered<GasCoin>(to), error::invalid_state(EACCOUNT_NOT_REGISTERED_FOR_GAS));
        slow_wallet::maybe_track_unlocked_deposit(to, coin::value(&coins));
        coin::deposit<GasCoin>(to, coins);
        // the incoming coins should trigger an update in tracker
        maybe_update_burn_tracker_impl(to);
    }

    // pass through function to guard the use of Coin
    public fun merge_coins(dst_coin: &mut Coin<GasCoin>, source_coin: Coin<GasCoin>) {
        // TODO: check it this is true: no tracking on merged coins since they are always withdrawn, and are a hot potato that might deposit later.
        // slow_wallet::maybe_track_unlocked_deposit(to, coin::value(&coins));
        coin::merge<GasCoin>(dst_coin, source_coin);
    }


    public fun assert_account_exists(addr: address) {
        assert!(account::exists_at(addr), error::not_found(EACCOUNT_NOT_FOUND));
    }

    public fun assert_account_is_registered_for_gas(addr: address) {
        assert_account_exists(addr);
        assert!(coin::is_account_registered<GasCoin>(addr), error::not_found(EACCOUNT_NOT_REGISTERED_FOR_GAS));
    }

    /// Set whether `account` can receive direct transfers of coins that they have not explicitly registered to receive.
    public entry fun set_allow_direct_coin_transfers(account: &signer, allow: bool) acquires DirectTransferConfig {
        let addr = signer::address_of(account);
        if (exists<DirectTransferConfig>(addr)) {
            let direct_transfer_config = borrow_global_mut<DirectTransferConfig>(addr);
            // Short-circuit to avoid emitting an event if direct transfer config is not changing.
            if (direct_transfer_config.allow_arbitrary_coin_transfers == allow) {
                return
            };

            direct_transfer_config.allow_arbitrary_coin_transfers = allow;
            emit_event(
                &mut direct_transfer_config.update_coin_transfer_events,
                DirectCoinTransferConfigUpdatedEvent { new_allow_direct_transfers: allow });
        } else {
            let direct_transfer_config = DirectTransferConfig {
                allow_arbitrary_coin_transfers: allow,
                update_coin_transfer_events: new_event_handle<DirectCoinTransferConfigUpdatedEvent>(account),
            };
            emit_event(
                &mut direct_transfer_config.update_coin_transfer_events,
                DirectCoinTransferConfigUpdatedEvent { new_allow_direct_transfers: allow });
            move_to(account, direct_transfer_config);
        };
    }


    #[view]
    /// Return true if `account` can receive direct transfers of coins that they have not explicitly registered to
    /// receive.
    ///
    /// By default, this returns true if an account has not explicitly set whether the can receive direct transfers.
    public fun can_receive_direct_coin_transfers(account: address): bool acquires DirectTransferConfig {
        !exists<DirectTransferConfig>(account) ||
            borrow_global<DirectTransferConfig>(account).allow_arbitrary_coin_transfers
    }

    // #[test_only]
    // use diem_std::from_bcs;
    // // #[test_only]
    // // use std::string::utf8;

    #[test_only]
    struct FakeCoin {}

    #[test(root = @ol_framework, alice = @0xa11ce, core = @0x1)]
    public fun test_transfer(root: &signer, alice: &signer, core: &signer)
    acquires BurnTracker {
        let bob = from_bcs::to_address(x"0000000000000000000000000000000000000000000000000000000000000b0b");
        let carol = from_bcs::to_address(x"00000000000000000000000000000000000000000000000000000000000ca501");

        let (burn_cap, mint_cap) = ol_framework::gas_coin::initialize_for_test(core);
        create_account(root, signer::address_of(alice));
        create_account(root, bob);
        create_account(root, carol);
        coin::deposit(signer::address_of(alice), coin::mint(10000, &mint_cap));
        transfer(alice, bob, 500);
        assert!(coin::balance<GasCoin>(bob) == 500, 0);
        transfer(alice, carol, 500);
        assert!(coin::balance<GasCoin>(carol) == 500, 1);
        transfer(alice, carol, 1500);
        assert!(coin::balance<GasCoin>(carol) == 2000, 2);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    #[test(root = @ol_framework, alice = @0xa11ce, core = @0x1)]
    public fun test_transfer_to_resource_account(root: &signer, alice: &signer,
    core: &signer) acquires BurnTracker{
        let (resource_account, _) = ol_create_resource_account(alice, vector[]);
        let resource_acc_addr = signer::address_of(&resource_account);
        // assert!(!coin::is_account_registered<GasCoin>(resource_acc_addr), 0);

        let (burn_cap, mint_cap) = ol_framework::gas_coin::initialize_for_test(core);
        create_account(root, signer::address_of(alice));
        coin::deposit(signer::address_of(alice), coin::mint(10000, &mint_cap));
        transfer(alice, resource_acc_addr, 500);
        assert!(coin::balance<GasCoin>(resource_acc_addr) == 500, 1);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    #[test(root = @ol_framework, from = @0x123, core = @0x1, recipient_1 = @0x124, recipient_2 = @0x125)]
    public fun test_batch_transfer(root: &signer, from: &signer, core: &signer,
    recipient_1: &signer, recipient_2: &signer) acquires BurnTracker{
        let (burn_cap, mint_cap) = diem_framework::gas_coin::initialize_for_test(core);
        create_account(root, signer::address_of(from));
        let recipient_1_addr = signer::address_of(recipient_1);
        let recipient_2_addr = signer::address_of(recipient_2);
        create_account(root, recipient_1_addr);
        create_account(root, recipient_2_addr);
        coin::deposit(signer::address_of(from), coin::mint(10000, &mint_cap));
        batch_transfer(
            from,
            vector[recipient_1_addr, recipient_2_addr],
            vector[100, 500],
        );
        assert!(coin::balance<GasCoin>(recipient_1_addr) == 100, 0);
        assert!(coin::balance<GasCoin>(recipient_2_addr) == 500, 1);
        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    // #[test(root = @ol_framework, from = @0x1, to = @0x12)]
    // public fun test_direct_coin_transfers(root: &signer, from: &signer, to: &signer) {
    //     let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
    //         from,
    //         utf8(b"FC"),
    //         utf8(b"FC"),
    //         10,
    //         true,
    //     );
    //     create_account(root, signer::address_of(from));
    //     create_account(root, signer::address_of(to));
    //     deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
    //     // Recipient account did not explicit register for the coin.
    //     let to_addr = signer::address_of(to);
    //     transfer_coins<FakeCoin>(from, to_addr, 500);
    //     assert!(coin::balance<FakeCoin>(to_addr) == 500, 0);

    //     coin::destroy_burn_cap(burn_cap);
    //     coin::destroy_mint_cap(mint_cap);
    //     coin::destroy_freeze_cap(freeze_cap);
    // }

    // #[test(root = @ol_framework, from = @0x1, recipient_1 = @0x124, recipient_2 = @0x125)]
    // public fun test_batch_transfer_fake_coin(root: signer,
    //     from: &signer, recipient_1: &signer, recipient_2: &signer) {
    //     let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
    //         from,
    //         utf8(b"FC"),
    //         utf8(b"FC"),
    //         10,
    //         true,
    //     );
    //     create_account(&root, signer::address_of(from));
    //     let recipient_1_addr = signer::address_of(recipient_1);
    //     let recipient_2_addr = signer::address_of(recipient_2);
    //     create_account(&root, recipient_1_addr);
    //     create_account(&root, recipient_2_addr);
    //     deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
    //     batch_transfer<FakeCoin>(
    //         from,
    //         vector[recipient_1_addr, recipient_2_addr],
    //         vector[100, 500],
    //     );
    //     assert!(coin::balance<FakeCoin>(recipient_1_addr) == 100, 0);
    //     assert!(coin::balance<FakeCoin>(recipient_2_addr) == 500, 1);

    //     coin::destroy_burn_cap(burn_cap);
    //     coin::destroy_mint_cap(mint_cap);
    //     coin::destroy_freeze_cap(freeze_cap);
    // }

    #[test(root = @ol_framework, user = @0x123)]
    public fun test_set_allow_direct_coin_transfers(root: &signer, user:
    &signer) acquires DirectTransferConfig {
        let addr = signer::address_of(user);
        create_account(root, addr);
        set_allow_direct_coin_transfers(user, true);
        assert!(can_receive_direct_coin_transfers(addr), 0);
        set_allow_direct_coin_transfers(user, false);
        assert!(!can_receive_direct_coin_transfers(addr), 1);
        set_allow_direct_coin_transfers(user, true);
        assert!(can_receive_direct_coin_transfers(addr), 2);
    }

    // #[test(root = @ol_framework, from = @0x1, to = @0x12)]
    // public fun test_direct_coin_transfers_with_explicit_direct_coin_transfer_config(
    //     root: &signer, from: &signer, to: &signer) acquires DirectTransferConfig {
    //     let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
    //         from,
    //         utf8(b"FC"),
    //         utf8(b"FC"),
    //         10,
    //         true,
    //     );
    //     create_account(root, signer::address_of(from));
    //     create_account(root, signer::address_of(to));
    //     set_allow_direct_coin_transfers(from, true);
    //     deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
    //     // Recipient account did not explicit register for the coin.
    //     let to_addr = signer::address_of(to);
    //     transfer_coins<FakeCoin>(from, to_addr, 500);
    //     assert!(coin::balance<FakeCoin>(to_addr) == 500, 0);

    //     coin::destroy_burn_cap(burn_cap);
    //     coin::destroy_mint_cap(mint_cap);
    //     coin::destroy_freeze_cap(freeze_cap);
    // }

    // #[test(root = @ol_framework, from = @0x1, to = @0x12)]
    // #[expected_failure(abort_code = 0x50003, location = Self)]
    // public fun test_direct_coin_transfers_fail_if_recipient_opted_out(
    //     root: &signer, from: &signer, to: &signer) acquires DirectTransferConfig {
    //     let (burn_cap, freeze_cap, mint_cap) = coin::initialize<FakeCoin>(
    //         from,
    //         utf8(b"FC"),
    //         utf8(b"FC"),
    //         10,
    //         true,
    //     );
    //     create_account(root, signer::address_of(from));
    //     create_account(root, signer::address_of(to));
    //     set_allow_direct_coin_transfers(from, false);
    //     deposit_coins(signer::address_of(from), coin::mint(1000, &mint_cap));
    //     // This should fail as the to account has explicitly opted out of receiving arbitrary coins.
    //     transfer_coins<FakeCoin>(from, signer::address_of(to), 500);

    //     coin::destroy_burn_cap(burn_cap);
    //     coin::destroy_mint_cap(mint_cap);
    //     coin::destroy_freeze_cap(freeze_cap);
    // }
}
