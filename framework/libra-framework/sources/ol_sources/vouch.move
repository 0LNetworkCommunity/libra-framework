
module ol_framework::vouch {
    use std::signer;
    use std::vector;
    // use aptos_framework::validator_universe;
    use ol_framework::ancestry;
    use ol_framework::globals;
    use ol_framework::testnet;
    use ol_framework::ol_account;

    use aptos_framework::system_addresses;
    use aptos_framework::stake;
    // use aptos_framework::coin;
    use aptos_framework::transaction_fee;

    /// Trying to vouch for yourself?
    const ETRY_SELF_VOUCH_REALLY: u64 = 1;

    /// how many epochs must pass before the voucher expires.
    const EXPIRATION_ELAPSED_EPOCHS: u64 = 90;
    // struct OneVouch has key, store, drop, copy {
    //   voucher: address,
    //   epoch: u64,
    // }
    // triggered once per epoch
    struct MyVouches has key {
      my_buddies: vector<address>,
      epoch_vouched: vector<u64>,
    }

    // init the struct on a validators account.
    public fun init(new_account_sig: &signer ) {
      let acc = signer::address_of(new_account_sig);

      if (!is_init(acc)) {
        move_to<MyVouches>(new_account_sig, MyVouches {
            my_buddies: vector::empty(),
            epoch_vouched: vector::empty(),
          });
      }
    }

    public fun is_init(acc: address ):bool {
      exists<MyVouches>(acc)
    }

    public entry fun vouch_for(ill_be_your_friend: &signer, wanna_be_my_friend: address) acquires MyVouches {
      let buddy_acc = signer::address_of(ill_be_your_friend);
      assert!(buddy_acc != wanna_be_my_friend, ETRY_SELF_VOUCH_REALLY);

      if (!exists<MyVouches>(wanna_be_my_friend)) return;

      // this fee is paid to the system, cannot be reclaimed
      let c = ol_account::withdraw(ill_be_your_friend, vouch_cost());
      transaction_fee::user_pay_fee(ill_be_your_friend, c);

      let v = borrow_global_mut<MyVouches>(wanna_be_my_friend);

      // let (found, i) = find_vouch(buddy_acc, wanna_be_my_friend);
      let (found, i) = vector::index_of(&v.my_buddies, &buddy_acc);
      if (found) { // prevent duplicates
        // update date
        let epoch = 0; // TODO get epoch
        let e = vector::borrow_mut(&mut v.epoch_vouched, i);
        *e = epoch;
      } else {
        vector::push_back(&mut v.my_buddies, buddy_acc);
        vector::push_back(&mut v.epoch_vouched, 0); // TODO get epoch

      }
    }

    public entry fun revoke(buddy: &signer, its_not_me_its_you: address) acquires MyVouches {
      let buddy_acc = signer::address_of(buddy);
      assert!(buddy_acc!=its_not_me_its_you, ETRY_SELF_VOUCH_REALLY);

      if (!exists<MyVouches>(its_not_me_its_you)) return;

      let v = borrow_global_mut<MyVouches>(its_not_me_its_you);
      let (found, i) = vector::index_of(&v.my_buddies, &buddy_acc);
      if (found) {
        vector::remove(&mut v.my_buddies, i);
        vector::remove(&mut v.epoch_vouched, i);
      };
    }

    public fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires MyVouches {
      system_addresses::assert_vm(vm);
      bulk_set(val, buddy_list);

    }

    fun bulk_set(val: address, buddy_list: vector<address>) acquires MyVouches {

      // if (!validator_universe::is_in_universe(val)) return;
      if (!exists<MyVouches>(val)) return;

      let v = borrow_global_mut<MyVouches>(val);

      // take self out of list
      let (is_found, i) = vector::index_of(&buddy_list, &val);

      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };

      v.my_buddies = buddy_list;
    }

    #[view]
    /// gets all buddies, including expired ones
    public fun get_buddies(val: address): vector<address> acquires MyVouches{

      if (!exists<MyVouches>(val)) return vector::empty<address>();
      let state = borrow_global<MyVouches>(val);
      *&state.my_buddies
    }

    #[view]
    /// gets the buddies and checks if they are expired
    public fun get_buddies_valid(val: address): vector<address> acquires MyVouches{
      if (!exists<MyVouches>(val)) return vector::empty<address>();

      let valid_vouches = vector::empty<address>();
      if (is_init(val)) {
        let state = borrow_global<MyVouches>(val);
        vector::for_each(state.my_buddies, |b| {
          if (is_not_expired(b, state)) {
            vector::push_back(&mut valid_vouches, b)
          }
        })
      };
      valid_vouches
    }

    fun is_not_expired(voucher: address, state: &MyVouches): bool {
      let (found, i) = vector::index_of(&state.my_buddies, &voucher);
      if (found) {
        let epoch = 0; // TODO get epoch
        let e = vector::borrow(&state.epoch_vouched, i);
        return epoch > *e + EXPIRATION_ELAPSED_EPOCHS
      };
      false
    }

    #[view]
    public fun buddies_in_set(val: address): vector<address> acquires MyVouches {
      let current_set = stake::get_current_validators();
      let (list, _) = buddies_in_list(val, current_set);
      list
    }

    public fun buddies_in_list(addr: address, list: vector<address>): (vector<address>, u64) acquires MyVouches {

      if (!exists<MyVouches>(addr)) return (vector::empty<address>(), 0);

      let v = borrow_global<MyVouches>(addr);

      let buddies_in_list = vector::empty<address>();
      let  i = 0;
      while (i < vector::length(&v.my_buddies)) {
        let addr = vector::borrow(&v.my_buddies, i);

        if (vector::contains(&list, addr)) {
          vector::push_back(&mut buddies_in_list, *addr);
        };
        i = i + 1;
      };

      (buddies_in_list, vector::length(&buddies_in_list))
    }


    public fun unrelated_buddies(list: vector<address>): vector<address> {
      // start our list empty
      let unrelated_buddies = vector::empty<address>();

      // iterare througth this list to see which accounts are created downstream of others.
      let len = vector::length<address>(&list);
      let  i = 0;
      while (i < len) {
        // for each account in list, compare to the others.
        // if they are unrelated, add them to the list.
        let target_acc = vector::borrow<address>(&list, i);

        // now loop through all the accounts again, and check if this target
        // account is related to anyone.
        let  k = 0;
        while (k < vector::length<address>(&list)) {
          let comparison_acc = vector::borrow(&list, k);
          // skip if you're the same person
          if (comparison_acc != target_acc) {
            // check ancestry algo
            let (is_fam, _) = ancestry::is_family(*comparison_acc, *target_acc);
            if (!is_fam) {
              if (!vector::contains(&unrelated_buddies, target_acc)) {
                vector::push_back<address>(&mut unrelated_buddies, *target_acc)
              }
            }
          };
          k = k + 1;
        };
        i = i + 1;
      };

      unrelated_buddies
    }

    public fun unrelated_buddies_above_thresh(val: address): bool acquires MyVouches{
      if (!exists<MyVouches>(val)) return false;

      if (testnet::is_testnet()) return true;
      let vouches = borrow_global<MyVouches>(val);

      let len = vector::length(&unrelated_buddies(vouches.my_buddies));
      (len >= globals::get_vouch_threshold())
    }

    // the cost to verify a vouch. Coins are burned.
    fun vouch_cost(): u64 {
      100
    }

    #[test_only]
    public fun test_set_buddies(val: address, buddy_list: vector<address>) acquires MyVouches {
      bulk_set(val, buddy_list);
    }
  }