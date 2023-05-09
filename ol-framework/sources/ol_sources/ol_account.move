
module ol_framework::ol_account {
  use aptos_framework::system_addresses;
  use aptos_framework::coin;
  use std::vector;
  use std::signer;
  use ol_framework::globals;
  use ol_framework::gas_coin::GasCoin;
  // TODO: v7: just stubs

      //////// SLOW WALLETS ////////
    // Slow wallets have a limited amount available to spend at every epoch.
    // Every epoch a new amount is made available (unlocked)
    // slow wallets can use the normal payment and transfer mechanisms to move
    // the unlocked amount.
    struct SlowWallet has key {
        unlocked: u64,
        transferred: u64,
    }

    struct SlowWalletList has key {
        list: vector<address>
    }

    public fun vm_init_slow(vm: &signer){
      system_addresses::assert_ol(vm);
      if (!exists<SlowWalletList>(@ol_framework)) {
        move_to<SlowWalletList>(vm, SlowWalletList {
          list: vector::empty<address>()
        });  
      }
    }

        /// private function which can only be called at genesis
    /// must apply the coin split factor.
    fun fork_migrate_slow_wallet(
      vm: &signer,
      user: &signer,
      unlocked: u64,
      transferred: u64,
    ) acquires SlowWalletList {
      system_addresses::assert_ol(vm);
      if (!exists<SlowWallet>(signer::address_of(user))) {
        move_to<SlowWallet>(vm, SlowWallet {
          unlocked: unlocked * globals::get_coin_split_factor(),
          transferred: transferred * globals::get_coin_split_factor(),
        });

        fork_migrate_slow_list(vm, user);
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
        vm_init_slow(vm); //don't abort
      };
      let list = borrow_global_mut<SlowWalletList>(@ol_framework);
      vector::push_back(&mut list.list, signer::address_of(user));
    }

    public fun set_slow(sig: &signer) acquires SlowWalletList {
      if (exists<SlowWalletList>(@ol_framework)) {
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

    ///////// SLOW GETTERS ////////

    public fun is_slow(addr: address): bool {
      exists<SlowWallet>(addr)
    }

    public fun unlocked_amount(addr: address): u64 acquires SlowWallet{
      if (exists<SlowWallet>(addr)) {
        let s = borrow_global<SlowWallet>(addr);
        return s.unlocked
      };
      // this is a normal account, so return the normal balance
      coin::balance<GasCoin>(addr)
    }

    // Getter for retrieving the list of slow wallets.
    public fun get_slow_list(): vector<address> acquires SlowWalletList{
      if (exists<SlowWalletList>(@ol_framework)) {
        let s = borrow_global<SlowWalletList>(@ol_framework);
        return *&s.list
      } else {
        return vector::empty<address>()
      }
    }


  // public fun is_slow(_addr: address): bool {
  //   true
  // }

  // public fun unlocked_amount(_addr: address): u64 {
  //   0

  public fun vm_multi_pay_fee(_vm: &signer, _list: &vector<address>, _price: u64, _metadata: &vector<u8>) {

  }
}
