//////// SLOW WALLETS ////////
// Slow wallets have a limited amount available to transfer between accounts.
// Using Coins for network operations has no limit. Sending funds to DonorDirected wallets is also unlimited. Coins are free and clear user's property.
// Every epoch a new amount is made available (unlocked)
// slow wallets can use the normal payment and transfer mechanisms to move
// the unlocked amount.

module ol_framework::slow_wallet {
  use std::error;
  use std::event;
  use std::vector;
  use std::signer;
  use diem_framework::system_addresses;
  use diem_framework::coin;
  use diem_framework::account;
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::testnet;
  use ol_framework::sacred_cows;

  // use diem_std::debug::print;

  friend ol_framework::ol_account;
  friend ol_framework::transaction_fee;

  /// genesis failed to initialized the slow wallet registry
  const EGENESIS_ERROR: u64 = 1;

  /// Maximum possible aggregatable coin value.
  const MAX_U64: u128 = 18446744073709551615;

    struct SlowWallet has key {
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

    public fun initialize(framework: &signer){
      system_addresses::assert_ol(framework);
      if (!exists<SlowWalletList>(@ol_framework)) {
        move_to<SlowWalletList>(framework, SlowWalletList {
          list: vector::empty<address>(),
          drip_events: account::new_event_handle<DripEvent>(framework)
        });
      }
    }

    /// private function which can only be called at genesis
    /// must apply the coin split factor.
    // TODO: make this private with a public test helper
    public fun fork_migrate_slow_wallet(
      vm: &signer,
      user: &signer,
      unlocked: u64,
      transferred: u64,
      // split_factor: u64,
    ) acquires SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);

      let user_addr = signer::address_of(user);
      if (!exists<SlowWallet>(user_addr)) {
        move_to<SlowWallet>(user, SlowWallet {
          unlocked,
          transferred,
        });

        update_slow_list(vm, user);
      } else {
        let state = borrow_global_mut<SlowWallet>(user_addr);
        state.unlocked = unlocked;
        state.transferred = transferred;
      }
    }

    /// private function which can only be called at genesis
    /// sets the list of accounts that are slow wallets.
    fun update_slow_list(
      vm: &signer,
      user: &signer,
    ) acquires SlowWalletList{
      system_addresses::assert_ol(vm);
      if (!exists<SlowWalletList>(@ol_framework)) {
        initialize(vm); //don't abort
      };
      let state = borrow_global_mut<SlowWalletList>(@ol_framework);
      let addr = signer::address_of(user);
      if (!vector::contains(&state.list, &addr)) {
        vector::push_back(&mut state.list, addr);
      }
    }

    public fun set_slow(sig: &signer) acquires SlowWalletList {
      assert!(exists<SlowWalletList>(@ol_framework), error::invalid_argument(EGENESIS_ERROR));

        let addr = signer::address_of(sig);
        let list = get_slow_list();
        if (!vector::contains<address>(&list, &addr)) {
            let s = borrow_global_mut<SlowWalletList>(@ol_framework);
            vector::push_back(&mut s.list, addr);
        };

        if (!exists<SlowWallet>(signer::address_of(sig))) {
          move_to<SlowWallet>(sig, SlowWallet {
            unlocked: coin::balance<LibraCoin>(addr),
            transferred: 0,
          });
        }
    }

    /// VM causes the slow wallet to unlock by X amount
    /// @return tuple of 2
    /// 0: bool, was this successful
    // 1: u64, how much was dripped
    public fun slow_wallet_epoch_drip(vm: &signer, amount: u64): (bool, u64) acquires
    SlowWallet, SlowWalletList{

      system_addresses::assert_ol(vm);
      let list = get_slow_list();
      let len = vector::length<address>(&list);
      if (len == 0) return (false, 0);
      let accounts_updated = 0;
      let i = 0;
      while (i < len) {
        let addr = vector::borrow<address>(&list, i);
        let user_balance = coin::balance<LibraCoin>(*addr);
        if (!exists<SlowWallet>(*addr)) continue; // NOTE: formal verifiction caught
        // this, not sure how it's possible

        let state = borrow_global_mut<SlowWallet>(*addr);

        // TODO implement this as a `spec`
        if ((state.unlocked as u128) + (amount as u128) >= MAX_U64) continue;

        let next_unlock = state.unlocked + amount;
        state.unlocked = if (next_unlock > user_balance) {
          // the user might have reached the end of the unlock period, and all
          // is unlocked
          user_balance
        } else {
          next_unlock
        };

        // it may be that some accounts were not updated, so we can't report
        // success unless that was the case.
        accounts_updated = accounts_updated + 1;

        i = i + 1;
      };

      emit_drip_event(vm, amount, accounts_updated);
      (accounts_updated==len, amount)
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
    /// Sould never abort.
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
    // 1: u64, how much was dripped
    public fun on_new_epoch(vm: &signer): (bool, u64) acquires SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);
      slow_wallet_epoch_drip(vm, sacred_cows::get_slow_drip_const())
    }

    ///////// SLOW GETTERS ////////

    #[view]
    public fun is_slow(addr: address): bool {
      exists<SlowWallet>(addr)
    }

    /// helper to get the unlocked and total balance. (unlocked, total)
    public(friend) fun balance(addr: address): (u64, u64) acquires SlowWallet{
      // this is a normal account, so return the normal balance
      let total = coin::balance<LibraCoin>(addr);
      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return (s.unlocked, total)
      };

      // if the account has no SlowWallet tracker, then everything is unlocked.
      (total, total)
    }

    /// Returns the amount of unlocked funds for a slow wallet.
    public fun unlocked_amount(addr: address): u64 acquires SlowWallet{
      // this is a normal account, so return the normal balance
      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return s.unlocked
      };

      coin::balance<LibraCoin>(addr)
    }

    #[view]
    /// Returns the amount of slow wallet transfers tracked
    public fun transferred_amount(addr: address): u64 acquires SlowWallet{
      // this is a normal account, so return the normal balance
      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return s.transferred
      };
      0
    }

    #[view]
    // Getter for retrieving the list of slow wallets.
    public fun get_slow_list(): vector<address> acquires SlowWalletList{
      if (exists<SlowWalletList>(@ol_framework)) {
        let s = borrow_global<SlowWalletList>(@ol_framework);
        return *&s.list
      } else {
        return vector::empty<address>()
      }
    }

    ////////// SMOKE TEST HELPERS //////////
    // cannot use the #[test_only] attribute
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
