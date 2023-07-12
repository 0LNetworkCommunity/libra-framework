//////// SLOW WALLETS ////////
// Slow wallets have a limited amount available to transfer between accounts.
// Using Coins for network operations has no limit. Sending funds to DonorDirected wallets is also unlimited. Coins are free and clear user's property.
// Every epoch a new amount is made available (unlocked)
// slow wallets can use the normal payment and transfer mechanisms to move
// the unlocked amount.

module ol_framework::slow_wallet {
  use aptos_framework::system_addresses;
  use aptos_framework::coin;
  use std::vector;
  use std::signer;
  // use ol_framework::globals;
  use ol_framework::gas_coin::GasCoin;
  use std::error;
  use std::fixed_point32;

  const EGENESIS_ERROR: u64 = 10001;

  const EPOCH_DRIP_CONST: u64 = 100000;

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
      split_factor: u64,
    ) acquires SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);
      let split_factor = fixed_point32::create_from_rational(split_factor, 1000000);
      let unlocked = if (unlocked > 0) {
        fixed_point32::multiply_u64(unlocked, split_factor)
      } else { 0 };

      let transferred = if (transferred > 0) {
        fixed_point32::multiply_u64(transferred, split_factor)
      } else { 0 };

      let user_addr = signer::address_of(user);
      if (!exists<SlowWallet>(user_addr)) {
        move_to<SlowWallet>(user, SlowWallet {
          unlocked,
          transferred,
        });

        fork_migrate_slow_list(vm, user);
      } else {
        let state = borrow_global_mut<SlowWallet>(user_addr);
        state.unlocked = unlocked;
        state.transferred = transferred;
      }
    }

    /// private function which can only be called at genesis
    /// sets the list of accounts that are slow wallets.
    fun fork_migrate_slow_list(
      vm: &signer,
      user: &signer,
    ) acquires SlowWalletList{
      system_addresses::assert_ol(vm);
      if (!exists<SlowWalletList>(@ol_framework)) {
        initialize(vm); //don't abort
      };
      let list = borrow_global_mut<SlowWalletList>(@ol_framework);
      vector::push_back(&mut list.list, signer::address_of(user));
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
            unlocked: 0,
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
        let s = borrow_global_mut<SlowWallet>(*addr);
        s.unlocked = s.unlocked + amount;
        i = i + 1;
      }
    }

    /////// 0L /////////
    // NOTE: danger, this is a private function that should only be called with account capability or VM.
    fun decrease_unlocked_tracker(payer: address, amount: u64) acquires SlowWallet {
      let s = borrow_global_mut<SlowWallet>(payer);
      s.transferred = s.transferred + amount;
      s.unlocked = s.unlocked - amount;
    }

    /////// 0L /////////
    fun increase_unlocked_tracker(recipient: address, amount: u64) acquires SlowWallet {
      let s = borrow_global_mut<SlowWallet>(recipient);
      s.unlocked = s.unlocked + amount;
    }

    public fun on_new_epoch(vm: &signer) acquires SlowWallet, SlowWalletList {
      system_addresses::assert_ol(vm);
      slow_wallet_epoch_drip(vm, EPOCH_DRIP_CONST);
    }

    public fun vm_multi_pay_fee(_vm: &signer, _list: &vector<address>, _price: u64, _metadata: &vector<u8>) {
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



}
