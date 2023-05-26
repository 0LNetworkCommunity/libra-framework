/////////////////////////////////////////////////////////////////////////
// 0L Module
// Receipts Module
// Error code:
/////////////////////////////////////////////////////////////////////////

// THe module stores the last receipt of payment to an account (if the user
// chooses to document the receipt), in addition the running cumulative payments
// to an account. This can be used by smart contracts to prove payments 
// interactively, event when a payment is not part of an atomic transaction 
// involving the smart contract. E.g. when an autopay transaction happens
// a payment to a community wallet can have a receipt for later use in a smart contract.

module ol_framework::receipts {
  // friend DiemFramework::DiemAccount; todo v7

  use std::vector;
  use std::signer;
  use aptos_framework::system_addresses;
  use aptos_framework::timestamp;
  use ol_framework::globals;

    struct UserReceipts has key {
      destination: vector<address>,
      cumulative: vector<u64>,
      last_payment_timestamp: vector<u64>,
      last_payment_value: vector<u64>,
    }

    // Utility used at genesis (and on upgrade) to initialize the system state.
    public fun init(account: &signer) { 
      let addr = signer::address_of(account);     
      if (!exists<UserReceipts>(addr)) {
        move_to<UserReceipts>(
          account,
          UserReceipts {
            destination: vector::empty<address>(),
            last_payment_timestamp: vector::empty<u64>(),
            last_payment_value: vector::empty<u64>(),
            cumulative: vector::empty<u64>(),
          }
        )
      }; 
    }

    // should only be called from the genesis script.
    fun fork_migrate(
      vm: &signer,
      account: &signer,
      destination: address,
      cumulative: u64,
      last_payment_timestamp: u64,
      last_payment_value: u64,
    ) acquires UserReceipts {
      
      system_addresses::assert_vm(vm);
      let addr = signer::address_of(account);
      assert!(is_init(addr), 0);
      let state = borrow_global_mut<UserReceipts>(addr);
      vector::push_back(&mut state.destination, destination);
      vector::push_back(
        &mut state.cumulative,
        cumulative * globals::get_coin_split_factor()
      );
      vector::push_back(&mut state.last_payment_timestamp, last_payment_timestamp);
      vector::push_back(
        &mut state.last_payment_value,
        last_payment_value * globals::get_coin_split_factor()
      );
    }

    public fun is_init(addr: address):bool {
      exists<UserReceipts>(addr)
    }

  public fun write_receipt_vm(
    sender: &signer,
    payer: address,
    destination: address,
    value: u64
  ):(u64, u64, u64) acquires UserReceipts {
      // TODO: make a function for user to write own receipt.
      system_addresses::assert_vm(sender);
      write_receipt(payer, destination, value)
  }
    
  /// Restricted to DiemAccount, we need to write receipts for certain users,
  /// like to DonorDirected Accounts.
  /// Core Devs: Danger: only DiemAccount can use this.
  public(friend) fun write_receipt(
    payer: address, 
    destination: address, 
    value: u64
  ):(u64, u64, u64) acquires UserReceipts {
      // TODO: make a function for user to write own receipt.
      if (!exists<UserReceipts>(payer)) {
        return (0, 0, 0)
      };

      let r = borrow_global_mut<UserReceipts>(payer);
      let (found_it, i) = vector::index_of(&r.destination, &destination);

      let cumu = 0;
      if (found_it) {
        cumu = *vector::borrow<u64>(&r.cumulative, i);
      };
      cumu = cumu + value;
      vector::push_back(&mut r.cumulative, *&cumu);

      let timestamp = timestamp::now_seconds();
      vector::push_back(&mut r.last_payment_timestamp, *&timestamp);
      vector::push_back(&mut r.last_payment_value, *&value);

      if (found_it) { // put in same index if the account was already there.
        vector::swap_remove(&mut r.last_payment_timestamp, i);
        vector::swap_remove(&mut r.last_payment_value, i);
        vector::swap_remove(&mut r.cumulative, i);
      } else {
        vector::push_back(&mut r.destination, destination);
      };
      
      (timestamp, value, cumu)
  }

    // Reads the last receipt for a given account, returns (timestamp of last
    // payment, last value sent, cumulative)
    public fun read_receipt(
      account: address, 
      destination: address
    ):(u64, u64, u64) acquires UserReceipts {
      if (!exists<UserReceipts>(account)) {
        return (0, 0, 0)
      };

      let receipt = borrow_global<UserReceipts>(account);
      let (found_it, i) = vector::index_of(&receipt.destination, &destination);
      if (!found_it) return (0, 0, 0);

      let time = vector::borrow<u64>(&receipt.last_payment_timestamp, i);
      let value = vector::borrow<u64>(&receipt.last_payment_value, i);
      let cumu = vector::borrow<u64>(&receipt.cumulative, i);

      (*time, *value, *cumu)
    }
}