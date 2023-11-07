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
  friend ol_framework::cumulative_deposits;

  use std::vector;
  use std::signer;
  use diem_framework::system_addresses;
  use diem_framework::timestamp;

  // use diem_std::debug::print;

  struct UserReceipts has key {
    destination: vector<address>,
    cumulative: vector<u64>,
    last_payment_timestamp: vector<u64>,
    last_payment_value: vector<u64>,
  }

  // Utility used at genesis (and on upgrade) to initialize the system state.
  public fun user_init(account: &signer) {
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
  fun genesis_migrate_user(
    vm: &signer,
    account: &signer,
    destination: vector<address>,
    cumulative: vector<u64>,
    last_payment_timestamp: vector<u64>,
    last_payment_value: vector<u64>,
    // assume split factor done in rust
  ) acquires UserReceipts {
    system_addresses::assert_ol(vm);
    let addr = signer::address_of(account);
    if (!exists<UserReceipts>(addr)) {
      move_to<UserReceipts>(
        account,
        UserReceipts {
          destination,
          last_payment_timestamp,
          last_payment_value,
          cumulative,
        }
      )
    } else {
    let state = borrow_global_mut<UserReceipts>(addr);
    state.destination = destination;
    state.cumulative = cumulative;
    state.last_payment_timestamp = last_payment_timestamp;
    state.last_payment_value = last_payment_value;
    }
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


  //     /// Separate struct to track cumulative deposits
  //   struct CumulativeDeposits has key {
  //       /// Store the cumulative deposits made to this account.
  //       /// not all accounts will have this enabled.
  //       value: u64, // the cumulative deposits with no adjustments.
  //       index: u64, // The index is a time-weighted cumulative sum of the deposits made to this account. This favors most recent donations.
  //       depositors: vector<address>, // donor directed wallets need a place to reference all the donors in the case of liquidation.
  //   }


  //   //////// 0L ////////
  //   // init struct for storing cumulative deposits, for community wallets
  //   // TODO: set true or false, that the account gets current balance or
  //   // starts at zero.
  //   public(friend) fun init_cumulative_deposits(sender: &signer) {
  //     let addr = signer::address_of(sender);
  //     if (!exists<CumulativeDeposits>(addr)) {
  //       move_to<CumulativeDeposits>(sender, CumulativeDeposits {
  //         value: 0,
  //         index: 0,
  //         depositors: vector::empty<address>(),
  //       })
  //     };
  //   }

  //   public fun vm_migrate_cumulative_deposits(vm: &signer, sender: &signer, starting_balance: u64) {
  //     system_addresses::assert_ol(vm);
  //     let addr = signer::address_of(sender);
  //     if (!exists<CumulativeDeposits>(addr)) {
  //       move_to<CumulativeDeposits>(sender, CumulativeDeposits {
  //         value: starting_balance,
  //         index: starting_balance,
  //         depositors: vector::empty<address>(),
  //       })
  //     };
  //   }

  //   /// private function for the genesis fork migration
  //   /// adjust for the coin split factor.
  //   // TODO! split factor wil likely have fractions
  //   fun fork_migrate_cumulative_deposits(vm: &signer, sender: &signer, value: u64, index: u64, coin_split_factor: u64) {
  //     system_addresses::assert_ol(vm);
  //     if (!exists<CumulativeDeposits>(signer::address_of(sender))) {
  //       move_to<CumulativeDeposits>(sender, CumulativeDeposits {
  //         value: value * coin_split_factor,
  //         index: index * coin_split_factor,
  //         depositors: vector::empty<address>(),
  //       })
  //     };
  //   }

  //   fun maybe_update_deposit(payer: address, payee: address, deposit_value: u64) acquires CumulativeDeposits, UserReceipts {
  //       // update cumulative deposits if the account has the struct.
  //       if (exists<CumulativeDeposits>(payee)) {
  //         let epoch = epoch_helper::get_current_epoch();
  //         // adjusted for the time-weighted index.
  //         let index = deposit_index_curve(epoch, deposit_value);
  //         let cumu = borrow_global_mut<CumulativeDeposits>(payee);
  //         cumu.value = cumu.value + deposit_value;
  //         cumu.index = cumu.index + index;

  //         // add the payer to the list of depositors.
  //         if (!vector::contains(&cumu.depositors, &payer)) {
  //           vector::push_back(&mut cumu.depositors, payer);
  //         };

  //         // also write the receipt to the payee's account.
  //         write_receipt(payer, payee, deposit_value);
  //       };
  //   }

  //     /// get the proportion of donoations of all donors to account.
  //  public fun get_pro_rata_cumu_deposits(multisig_address: address): (vector<address>, vector<FixedPoint32>, vector<u64>) acquires CumulativeDeposits, UserReceipts{
  //   // get total fees
  //   let balance = get_cumulative_deposits(multisig_address);
  //   let donors = get_depositors(multisig_address);
  //   let pro_rata_addresses = vector::empty<address>();
  //   let pro_rata = vector::empty<FixedPoint32>();
  //   let pro_rata_amounts = vector::empty<u64>();

  //   let i = 0;
  //   let len = vector::length(&donors);
  //   while (i < len) {
  //     let donor = vector::borrow(&donors, i);
  //     let (_, _, cumu)  = read_receipt(*donor, multisig_address);

  //     let ratio = fixed_point32::create_from_rational(cumu, balance);
  //     let value = fixed_point32::multiply_u64(balance, copy ratio);

  //     vector::push_back(&mut pro_rata_addresses, *donor);
  //     vector::push_back(&mut pro_rata, ratio);
  //     vector::push_back(&mut pro_rata_amounts, value);
  //     i = i + 1;
  //   };

  //     (pro_rata_addresses, pro_rata, pro_rata_amounts)
  //  }


  //   /// adjust the points of the deposits favoring more recent deposits.
  //   /// inflation by x% per day from the start of network.
  //   fun deposit_index_curve(
  //     epoch: u64,
  //     value: u64,
  //   ): u64 {

  //     // increment 1/2 percent per day, not compounded.
  //     (value * (1000 + (epoch * 5))) / 1000
  //   }

  //   //////// GETTERS ////////
  //   public fun is_init_cumu_tracking(addr: address): bool {
  //     exists<CumulativeDeposits>(addr)
  //   }

  //   #[view]
  //   public fun get_depositors(payee: address): vector<address> acquires CumulativeDeposits {
  //     let cumu = borrow_global<CumulativeDeposits>(payee);
  //     *&cumu.depositors
  //   }

  //   #[view]
  //   public fun get_cumulative_deposits(addr: address): u64 acquires CumulativeDeposits {
  //     if (!exists<CumulativeDeposits>(addr)) return 0;

  //     borrow_global<CumulativeDeposits>(addr).value
  //   }

  //   #[view]
  //   public fun get_index_cumu_deposits(addr: address): u64 acquires CumulativeDeposits {
  //     if (!exists<CumulativeDeposits>(addr)) return 0;

  //     borrow_global<CumulativeDeposits>(addr).index
  //   }



}