//////// SLOW WALLETS ////////
// Slow wallets have a limited amount available to transfer between accounts.
// Using Coins for network operations has no limit. Sending funds to DonorDirected wallets is also unlimited. Coins are free and clear user's property.
// Every epoch a new amount is made available (unlocked)
// slow wallets can use the normal payment and transfer mechanisms to move
// the unlocked amount.

module ol_framework::slow_wallet {
  use diem_framework::system_addresses;
  use diem_framework::coin;
  use std::vector;
  use std::signer;
  use ol_framework::gas_coin::GasCoin;
  use ol_framework::testnet;
  use std::error;
  use ol_framework::sacred_cows;

  // use diem_std::debug::print;

  friend ol_framework::ol_account;

  /// genesis failed to initialized the slow wallet registry
  const EGENESIS_ERROR: u64 = 1;


    struct SlowWallet has key {
        unlocked: u64,
        transferred: u64,
    }

    struct SlowWalletList has key {
        list: vector<address>
    }

    public fun initialize(vm: &signer){
      system_addresses::assert_ol(vm);
      if (!exists<SlowWalletList>(@ol_framework)) {
        move_to<SlowWalletList>(vm, SlowWalletList {
          list: vector::empty<address>()
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
            unlocked: coin::balance<GasCoin>(addr),
            transferred: 0,
          });
        }
    }

    public fun slow_wallet_epoch_drip(vm: &signer, amount: u64) acquires SlowWallet, SlowWalletList{
      system_addresses::assert_ol(vm);
      let list = get_slow_list();
      let i = 0;
      while (i < vector::length<address>(&list)) {
        let addr = vector::borrow<address>(&list, i);
        let total = coin::balance<GasCoin>(*addr);
        let state = borrow_global_mut<SlowWallet>(*addr);
        let next_unlock = state.unlocked + amount;
        state.unlocked = if (next_unlock > total) { total } else { next_unlock };
        i = i + 1;
      }
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
    public(friend) fun maybe_track_unlocked_withdraw(payer: address, amount: u64) acquires SlowWallet {
      if (!exists<SlowWallet>(payer)) return;
      let s = borrow_global_mut<SlowWallet>(payer);

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

    public fun on_new_epoch(vm: &signer) acquires SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);
      slow_wallet_epoch_drip(vm, sacred_cows::get_slow_drip_const());
    }

    ///////// SLOW GETTERS ////////

    #[view]
    public fun is_slow(addr: address): bool {
      exists<SlowWallet>(addr)
    }

    #[view]
    /// helper to get the unlocked and total balance. (unlocked, total)
    public fun balance(addr: address): (u64, u64) acquires SlowWallet{
      // this is a normal account, so return the normal balance
      let total = coin::balance<GasCoin>(addr);
      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return (s.unlocked, total)
      };

      // if the account has no SlowWallet tracker, then everything is unlocked.
      (total, total)
    }

    #[view]
    // TODO: Deprecate this function in favor of `balance`
    /// Returns the amount of unlocked funds for a slow wallet.
    public fun unlocked_amount(addr: address): u64 acquires SlowWallet{
      // this is a normal account, so return the normal balance
      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return s.unlocked
      };

      coin::balance<GasCoin>(addr)
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
