//////// SLOW WALLETS ////////
// Slow wallets have a limited amount available to transfer between accounts.
// Using Coins for network operations has no limit. Sending funds to DonorDirected wallets is also unlimited. Coins are free and clear user's property.
// Every epoch a new amount is made available (unlocked)
// slow wallets can use the normal payment and transfer mechanisms to move
// the unlocked amount.

module ol_framework::slow_wallet {
  use std::error;
  use std::event;
  use std::signer;
  use std::vector;
  use diem_framework::account;
  use diem_framework::system_addresses;
  use ol_framework::libra_coin;
  use ol_framework::reauthorization;
  use ol_framework::sacred_cows;
  use ol_framework::testnet;

  friend diem_framework::genesis;
  friend ol_framework::ol_account;
  friend ol_framework::transaction_fee;
  friend ol_framework::epoch_boundary;
  friend ol_framework::filo_migration;

  #[test_only]
  friend ol_framework::test_slow_wallet;
  #[test_only]
  friend ol_framework::test_pof;
  #[test_only]
  friend ol_framework::mock;
  #[test_only]
  friend ol_framework::test_boundary;



  /// genesis failed to initialized the slow wallet registry
  const EGENESIS_ERROR: u64 = 1;
  /// supply calculations exceed total supply
  const EINVALID_SUPPLY: u64 = 2;

  /// Maximum possible aggregatable coin value.
  const MAX_U64: u128 = 18446744073709551615;

    struct SlowWallet has key, drop {
        unlocked: u64,
        transferred: u64,
    }

    // the drip event at end of epoch
    struct DripEvent has drop, store {
      value: u64,
      users: u64,
    }

    struct SlowWalletList has key {
        list: vector<address>,
        drip_events: event::EventHandle<DripEvent>,
    }

    struct SlowSupply has key {
        total: u64,
        unlocked: u64,
        transferred: u64,
        daily_unlocked: u64,
    }

    public(friend) fun initialize(framework: &signer){
      system_addresses::assert_ol(framework);
      if (!exists<SlowWalletList>(@ol_framework)) {
        move_to<SlowWalletList>(framework, SlowWalletList {
          list: vector::empty<address>(),
          drip_events: account::new_event_handle<DripEvent>(framework)
        });
      }
    }

    /// one time function for all founder accounts to migrate
    /// on close of Level 7 FILO upgrade
    public(friend) fun filo_migration_reset(sig: &signer) acquires SlowWallet, SlowWalletList {
        // I feel as naked as the hour that I was born
        // Running wild with you right through
        // This coming storm that's gonna form
        // And tear our world apart

        set_slow(sig);

        // Love is a long game we can play
        // You turn to me and say
        // Let's break these rules
        // No use in playing it cool
        // No separation, you and I

        // for already existing accounts
        let state = borrow_global_mut<SlowWallet>(signer::address_of(sig));
        state.unlocked = 0;
        state.transferred = 0;
    }

    /// Users can change their account to slow, by calling the entry function
    /// Warning: this is permanent for the account. There's no way to
    /// reverse a "slow wallet".
    public entry fun user_set_slow(sig: &signer) acquires SlowWalletList {
      set_slow(sig);
    }

    /// implementation of setting slow wallet
    fun set_slow(sig: &signer) acquires SlowWalletList {
      assert!(exists<SlowWalletList>(@ol_framework), error::invalid_argument(EGENESIS_ERROR));

        let addr = signer::address_of(sig);
        let list = get_slow_list();
        if (!vector::contains<address>(&list, &addr)) {
            let s = borrow_global_mut<SlowWalletList>(@ol_framework);
            vector::push_back(&mut s.list, addr);
        };

        if (!exists<SlowWallet>(signer::address_of(sig))) {
          move_to<SlowWallet>(sig, SlowWallet {
            unlocked: libra_coin::balance(addr),
            transferred: 0,
          });
        }
    }

    /// helper to get the unlocked and total balance. (unlocked, total)
    public(friend) fun unlocked_and_total(addr: address): (u64, u64) acquires SlowWallet {
      // this is a normal account, so return the normal balance
      let total = libra_coin::balance(addr);
      let unlocked = unlocked_amount(addr);
      (unlocked, total)
    }

    /// VM causes the slow wallet to unlock by X amount
    /// @return tuple of 2
    /// 0: bool, was this successful
    /// 1: u64, how much was dripped
    public(friend) fun slow_wallet_epoch_drip(vm: &signer, amount: u64): (bool, u64) acquires SlowSupply, SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);
      assert!(exists<SlowWalletList>(@ol_framework), error::invalid_argument(EGENESIS_ERROR));

      let list = slow_wallets_to_unlock();

      let len = vector::length<address>(&list);
      if (len == 0) return (false, 0);
      let accounts_updated = 0;
      let global_unlocked = 0;

      let i = 0;
      while (i < len) {
        let addr = vector::borrow<address>(&list, i);
        let user_balance = libra_coin::balance(*addr);
        if (!exists<SlowWallet>(*addr)) continue; // NOTE: formal verification caught this, not sure how it's possible

        let state = borrow_global_mut<SlowWallet>(*addr);

        // TODO implement this as a `spec`
        if ((state.unlocked as u128) + (amount as u128) >= MAX_U64) continue;

        let next_unlock = state.unlocked + amount;

        let amount_to_unlock = if (next_unlock > user_balance) {
          // the user might have reached the end of the unlock period, and all
          // is unlocked
          user_balance
        } else {
          next_unlock
        };

        global_unlocked = global_unlocked + amount_to_unlock;

        // set the new unlocked amount
        state.unlocked = amount_to_unlock;

        // it may be that some accounts were not updated, so we can't report
        // success unless that was the case.
        spec {
          assume accounts_updated + 1 < MAX_U64;
        };

        accounts_updated = accounts_updated + 1;

        i = i + 1;
      };

      if (exists<SlowSupply>(@ol_framework)) {
        let state = borrow_global_mut<SlowSupply>(@ol_framework);
        state.daily_unlocked = global_unlocked;
      }; // else we're not quite ready to track

      emit_drip_event(vm, amount, accounts_updated);
      (accounts_updated==len, amount)
    }

    /// function which runs also at the epoch boundary, to loop through
    /// all of the users in slow wallet and update the SlowSupply data
    /// structure.
    // TODO: there's duplication in running through the list multiple times
    // some are view functions, others are only called on epoch boundary.
    fun update_slow_supply(framework: &signer) acquires SlowSupply, SlowWallet, SlowWalletList {
      system_addresses::assert_diem_framework(framework);

      let (unlocked, total, transferred) = get_slow_supply();

      // migrate on the fly if it does not exist
      if (!exists<SlowSupply>(@ol_framework)) {
        move_to<SlowSupply>(framework, SlowSupply {
          total,
          unlocked,
          transferred,
          daily_unlocked: 0,
        });
      } else {
        let state = borrow_global_mut<SlowSupply>(@ol_framework);
        state.total = total;
        state.unlocked = unlocked;
        state.transferred = transferred;
      }
    }


    /// send a drip event notification with the totals of epoch
    fun emit_drip_event(root: &signer, value: u64, users: u64) acquires SlowWalletList {
        system_addresses::assert_ol(root);
        let state = borrow_global_mut<SlowWalletList>(@ol_framework);
        event::emit_event(
          &mut state.drip_events,
          DripEvent {
              value,
              users,
          },
      );
    }

    /// wrapper to both attempt to adjust the slow wallet tracker
    /// on the sender and recipient.
    /// if either account is not a slow wallet no tracking
    /// will happen on that account.
    /// Should never abort.
    public(friend) fun maybe_track_slow_transfer(payer: address, recipient: address, amount: u64) acquires SlowWallet {
      maybe_track_unlocked_withdraw(payer, amount);
      maybe_track_unlocked_deposit(recipient, amount);
    }
    /// if a user spends/transfers unlocked coins we need to track that spend
    public(friend) fun maybe_track_unlocked_withdraw(payer: address, amount:
    u64) acquires SlowWallet {

      if (!exists<SlowWallet>(payer)) return;
      let s = borrow_global_mut<SlowWallet>(payer);

      spec {
        assume s.transferred + amount < MAX_U64;
      };

      s.transferred = s.transferred + amount;

      // THE VM is able to overdraw an account's unlocked amount.
      // in that case we need to check for zero.
      if (s.unlocked > amount) {
        s.unlocked = s.unlocked - amount;
      } else {
        s.unlocked = 0;
      }

    }

    /// when a user receives unlocked coins from another user, those coins
    /// always remain unlocked.
    public(friend) fun maybe_track_unlocked_deposit(recipient: address, amount: u64) acquires SlowWallet {
      if (!exists<SlowWallet>(recipient)) return;
      let state = borrow_global_mut<SlowWallet>(recipient);

      // TODO:
      // unlocked amount cannot be greater than total
      // this will not halt, since it's the VM that may call this.
      // but downstream code needs to check this
      state.unlocked = state.unlocked + amount;
    }

    /// Every epoch the system will drip a fixed amount
    /// @return tuple of 2
    /// 0: bool, was this successful
    /// 1: u64, how much was dripped
    public(friend) fun on_new_epoch(vm: &signer): (bool, u64) acquires SlowSupply, SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);
      let (ok, accts) = slow_wallet_epoch_drip(vm, sacred_cows::get_slow_drip_const());
      update_slow_supply(vm);
      (ok, accts)
    }

    ///////// GETTERS ////////

    #[view]
    public fun is_slow(addr: address): bool {
      exists<SlowWallet>(addr)
    }

    #[view]
    /// Returns the amount of unlocked funds for a slow wallet.
    public fun unlocked_amount(addr: address): u64 acquires SlowWallet {
      // if the account has never been activated, the unlocked amount is
      // zero despite the state (which is stale, until there is a migration).

      if (!reauthorization::is_v8_authorized(addr)) {
        return 0
      };

      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return s.unlocked
      };

      // this is a normal account, so return the normal balance

      libra_coin::balance(addr)
    }

    #[view]
    /// Returns the amount of slow wallet transfers tracked
    public fun transferred_amount(addr: address): u64 acquires SlowWallet{

      // this is a normal account, so return the normal balance
      if (exists<SlowWallet>(addr)) {
        if (!reauthorization::is_v8_authorized(addr)) {
          return 0
        };
        let s = borrow_global<SlowWallet>(addr);
        return s.transferred
      };
      0
    }

    #[view]
    // Getter for retrieving the list of slow wallets.
    // NOTE: this includes all slow wallets, active or not.
    public fun get_slow_list(): vector<address> acquires SlowWalletList{
      get_slow_list_internal()
    }

    // Internal implementation to avoid circular references that confuse the formal verifier
    fun get_slow_list_internal(): vector<address> acquires SlowWalletList {
      if (exists<SlowWalletList>(@ol_framework)) {
        let s = borrow_global<SlowWalletList>(@ol_framework);
        return *&s.list
      } else {
        return vector::empty<address>()
      }
    }

    #[view]
    /// filter the slow wallet list based on whether the
    /// account has the Activity struct from activity.move
    /// this will be the indication that they have already migrated
    /// until then they will not be unlocking.
    public fun slow_wallets_to_unlock(): vector<address> acquires SlowWalletList{
      let slow_list = get_slow_list();
      let slow_list_len = vector::length<address>(&slow_list);
      let active_slow_wallets = vector::empty<address>();
      let i = 0;
      while (i < slow_list_len) {
        let addr = vector::borrow<address>(&slow_list, i);
        if (reauthorization::is_v8_authorized(*addr)) {
          vector::push_back(&mut active_slow_wallets, *addr);
        };
        i = i + 1;
      };
      active_slow_wallets
    }

    #[view]
    // Getter for retrieving the list of slow wallets.
    public fun get_locked_supply(): u64 acquires SlowWalletList, SlowWallet{
      let (unlocked, total, _transferred) = get_slow_supply();
      if (total > unlocked) {
        return total - unlocked
      } else {
        return 0
      }
    }

    #[view]
    /// Getting the unlocked global supply means seeing what
    /// has been transferred out, and what is unlocked but not yet
    /// transferred
    public fun get_lifetime_unlocked_supply(): u64 acquires SlowWalletList, SlowWallet{
      let (unlocked, _total, transferred) = get_slow_supply();
      unlocked + transferred
    }

    #[view]
    /// Returns the aggregate statistics for all slow wallets in the system
    /// @return a tuple with three values:
    /// - unlocked: the total amount of unlocked coins across all slow wallets
    /// - total: the total balance of all slow wallet accounts
    /// - transferred: the total amount that has been transferred from all slow wallets
    public fun get_slow_supply(): (u64, u64, u64) acquires SlowWallet, SlowWalletList {
      let list = get_slow_list();
      let len = vector::length<address>(&list);

      let total = 0;
      let unlocked = 0;
      let transferred = 0;

      let i = 0;
      while (i < len) {
        let addr = vector::borrow<address>(&list, i);
        let (u, t) = unlocked_and_total(*addr);
        spec {
          assume total + t < MAX_U64;
          assume unlocked + u < MAX_U64;
        };

        total = total + t;
        unlocked = unlocked + u;
        transferred = transferred + transferred_amount(*addr);

        // TODO:
        // assert!(total < system_supply, error::invalid_argument(EINVALID_SUPPLY));
        // assert!(unlocked < total, error::invalid_argument(EINVALID_SUPPLY));
        // assert!(transferred < total, error::invalid_argument(EINVALID_SUPPLY));

        i = i + 1;
      };

      (unlocked, total, transferred)
    }

    //////// MIGRATIONS ////////

    /// private function which can only be called at genesis
    /// must apply the coin split factor.
    /// TODO: make this private with a public test helper
    fun set_slow_wallet_state(
      framework: &signer,
      user: &signer,
      unlocked: u64,
      transferred: u64,
    ) acquires SlowWallet, SlowWalletList {
      system_addresses::assert_diem_framework(framework);

      let user_addr = signer::address_of(user);
      if (!exists<SlowWallet>(user_addr)) {
        move_to<SlowWallet>(user, SlowWallet {
          unlocked,
          transferred,
        });

        update_slow_list(framework, user);
      } else {
        let state = borrow_global_mut<SlowWallet>(user_addr);
        state.unlocked = unlocked;
        state.transferred = transferred;
      }
    }

    /// private function which can only be called at genesis
    /// sets the list of accounts that are slow wallets.
    fun update_slow_list(
      framework: &signer,
      user: &signer,
    ) acquires SlowWalletList{
      system_addresses::assert_diem_framework(framework);
      if (!exists<SlowWalletList>(@ol_framework)) {
        initialize(framework); //don't abort
      };
      let state = borrow_global_mut<SlowWalletList>(@ol_framework);
      let addr = signer::address_of(user);
      if (!vector::contains(&state.list, &addr)) {
        vector::push_back(&mut state.list, addr);
      }
    }


    // Commit note: deprecated function from v6->v7 migration

    //////// TEST HELPERS /////////

    #[test_only]
    public fun test_set_slow_wallet(
      vm: &signer,
      user: &signer,
      unlocked: u64,
      transferred: u64,
    ) acquires SlowWallet, SlowWalletList {
      testnet::assert_testnet(vm);

      set_slow_wallet_state(
        vm,
        user,
        unlocked,
        transferred
      )
    }

    #[test_only]
    public fun test_epoch_drip(
      vm: &signer,
      amount: u64,
    ) acquires SlowSupply, SlowWallet, SlowWalletList {
      testnet::assert_testnet(vm);
      slow_wallet_epoch_drip(vm, amount);
    }

    ////////// SMOKE TEST HELPERS //////////
    // cannot use the #[test_only] attribute in smoke tests
    public entry fun smoke_test_vm_unlock(
      smoke_test_core_resource: &signer,
      user_addr: address,
      unlocked: u64,
      transferred: u64,
    ) acquires SlowWallet {

      system_addresses::assert_core_resource(smoke_test_core_resource);
      testnet::assert_testnet(smoke_test_core_resource);
      let state = borrow_global_mut<SlowWallet>(user_addr);
      state.unlocked = unlocked;
      state.transferred = transferred;
    }
}
