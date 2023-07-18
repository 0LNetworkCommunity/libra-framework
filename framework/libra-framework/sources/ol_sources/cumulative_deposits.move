
module ol_framework::cumulative_deposits {
  use std::signer;
  use std::vector;
  use ol_framework::epoch_helper;
  use aptos_framework::system_addresses;
  use ol_framework::receipts;
  // use ol_framework::ol_account;
  friend ol_framework::ol_account;
  friend ol_framework::donor_directed;

  use std::fixed_point32::{Self, FixedPoint32};
      /// Separate struct to track cumulative deposits
    struct CumulativeDeposits has key {
        /// Store the cumulative deposits made to this account.
        /// not all accounts will have this enabled.
        value: u64, // the cumulative deposits with no adjustments.
        index: u64, // The index is a time-weighted cumulative sum of the deposits made to this account. This favors most recent donations.
        depositors: vector<address>, // donor directed wallets need a place to reference all the donors in the case of liquidation.
    }


    //////// 0L ////////
    // init struct for storing cumulative deposits, for community wallets
    // TODO: set true or false, that the account gets current balance or
    // starts at zero.
    public(friend) fun init_cumulative_deposits(sender: &signer) {
      let addr = signer::address_of(sender);
      if (!exists<CumulativeDeposits>(addr)) {
        move_to<CumulativeDeposits>(sender, CumulativeDeposits {
          value: 0,
          index: 0,
          depositors: vector::empty<address>(),
        })
      };
    }

    public fun vm_migrate_cumulative_deposits(vm: &signer, sender: &signer, starting_balance: u64) {
      system_addresses::assert_ol(vm);
      let addr = signer::address_of(sender);
      if (!exists<CumulativeDeposits>(addr)) {
        move_to<CumulativeDeposits>(sender, CumulativeDeposits {
          value: starting_balance,
          index: starting_balance,
          depositors: vector::empty<address>(),
        })
      };
    }

    /// private function for the genesis fork migration
    /// adjust for the coin split factor.
    // TODO! split factor wil likely have fractions
    fun fork_migrate_cumulative_deposits(vm: &signer, sender: &signer, value: u64, index: u64, coin_split_factor: u64) {
      system_addresses::assert_ol(vm);
      if (!exists<CumulativeDeposits>(signer::address_of(sender))) {
        move_to<CumulativeDeposits>(sender, CumulativeDeposits {
          value: value * coin_split_factor,
          index: index * coin_split_factor,
          depositors: vector::empty<address>(),
        })
      };
    }

    public(friend) fun maybe_update_deposit(payer: address, payee: address, deposit_value: u64) acquires CumulativeDeposits {
        // update cumulative deposits if the account has the struct.
        if (exists<CumulativeDeposits>(payee)) {
          let epoch = epoch_helper::get_current_epoch();
          // adjusted for the time-weighted index.
          let index = deposit_index_curve(epoch, deposit_value);
          let cumu = borrow_global_mut<CumulativeDeposits>(payee);
          cumu.value = cumu.value + deposit_value;
          cumu.index = cumu.index + index;

          // add the payer to the list of depositors.
          if (!vector::contains(&cumu.depositors, &payer)) {
            vector::push_back(&mut cumu.depositors, payer);
          };

          // also write the receipt to the payee's account.
          receipts::write_receipt(payer, payee, deposit_value);
        };
    }

      /// get the proportion of donoations of all donors to account.
   public fun get_pro_rata_cumu_deposits(multisig_address: address): (vector<address>, vector<FixedPoint32>, vector<u64>) acquires CumulativeDeposits{
    // get total fees
    let balance = get_cumulative_deposits(multisig_address);
    let donors = get_depositors(multisig_address);
    let pro_rata_addresses = vector::empty<address>();
    let pro_rata = vector::empty<FixedPoint32>();
    let pro_rata_amounts = vector::empty<u64>();

    let i = 0;
    let len = vector::length(&donors);
    while (i < len) {
      let donor = vector::borrow(&donors, i);
      let (_, _, cumu)  = receipts::read_receipt(*donor, multisig_address);

      let ratio = fixed_point32::create_from_rational(cumu, balance);
      let value = fixed_point32::multiply_u64(balance, copy ratio);

      vector::push_back(&mut pro_rata_addresses, *donor);
      vector::push_back(&mut pro_rata, ratio);
      vector::push_back(&mut pro_rata_amounts, value);
      i = i + 1;
    };

      (pro_rata_addresses, pro_rata, pro_rata_amounts)
   }


    /// adjust the points of the deposits favoring more recent deposits.
    /// adjusting by x% per day from the start of network.
    fun deposit_index_curve(
      epoch: u64,
      value: u64,
    ): u64 {

      // increment 1/2 percent per day, not compounded.
      (value * (1000 + (epoch * 5))) / 1000
    }

    //////// GETTERS ////////
    public fun is_init_cumu_tracking(addr: address): bool {
      exists<CumulativeDeposits>(addr)
    }

    #[view]
    public fun get_depositors(payee: address): vector<address> acquires CumulativeDeposits {
      let cumu = borrow_global<CumulativeDeposits>(payee);
      *&cumu.depositors
    }

    #[view]
    public fun get_cumulative_deposits(addr: address): u64 acquires CumulativeDeposits {
      if (!exists<CumulativeDeposits>(addr)) return 0;

      borrow_global<CumulativeDeposits>(addr).value
    }

    #[view]
    public fun get_index_cumu_deposits(addr: address): u64 acquires CumulativeDeposits {
      if (!exists<CumulativeDeposits>(addr)) return 0;

      borrow_global<CumulativeDeposits>(addr).index
    }

}