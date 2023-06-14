
module ol_framework::vouch {
    use std::signer;
    use std::vector;
    // use aptos_framework::validator_universe;
    use ol_framework::ancestry;
    use ol_framework::globals;
    use ol_framework::testnet;
    use aptos_framework::system_addresses;
    use aptos_framework::stake;

    /// Trying to vouch for yourself?
    const ETRY_SELF_VOUCH_REALLY: u64 = 12345;

    // triggered once per epoch
    struct Vouch has key {
      my_buddies: vector<address>,
    }

    // init the struct on a validators account.
    public fun init(new_account_sig: &signer ) {
      let acc = signer::address_of(new_account_sig);

      if (!is_init(acc)) {
        move_to<Vouch>(new_account_sig, Vouch {
            my_buddies: vector::empty(), 
          });
      }
    }

    public fun is_init(acc: address ):bool {
      exists<Vouch>(acc)
    }

    public fun vouch_for(buddy: &signer, val: address) acquires Vouch {
      let buddy_acc = signer::address_of(buddy);
      assert!(buddy_acc!=val, ETRY_SELF_VOUCH_REALLY);

      // if (!validator_universe::is_in_universe(buddy_acc)) return;
      if (!exists<Vouch>(val)) return;

      let v = borrow_global_mut<Vouch>(val);
      if (!vector::contains(&v.my_buddies, &buddy_acc)) { // prevent duplicates
        vector::push_back<address>(&mut v.my_buddies, buddy_acc);
      }
    }

    public fun revoke(buddy: &signer, val: address) acquires Vouch {
      let buddy_acc = signer::address_of(buddy);
      assert!(buddy_acc!=val, ETRY_SELF_VOUCH_REALLY);

      if (!exists<Vouch>(val)) return;

      let v = borrow_global_mut<Vouch>(val);
      let (found, i) = vector::index_of(&v.my_buddies, &buddy_acc);
      if (found) {
        vector::remove(&mut v.my_buddies, i);
      };
    }    

    public fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires Vouch {
      system_addresses::assert_vm(vm);
      bulk_set(val, buddy_list);

    }

    fun bulk_set(val: address, buddy_list: vector<address>) acquires Vouch {

      // if (!validator_universe::is_in_universe(val)) return;
      if (!exists<Vouch>(val)) return;

      let v = borrow_global_mut<Vouch>(val);

      // take self out of list
      let (is_found, i) = vector::index_of(&buddy_list, &val);

      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };
      
      v.my_buddies = buddy_list;
    }

    public fun get_buddies(val: address): vector<address> acquires Vouch{
      if (is_init(val)) {
        return *&borrow_global<Vouch>(val).my_buddies
      };
      vector::empty<address>()
    }

    public fun buddies_in_set(val: address): vector<address> acquires Vouch {
      let current_set = stake::get_current_validators();
      let (list, _) = buddies_in_list(val, current_set);
      list
    }

    public fun buddies_in_list(addr: address, list: vector<address>): (vector<address>, u64) acquires Vouch {

      if (!exists<Vouch>(addr)) return (vector::empty<address>(), 0);

      let v = borrow_global<Vouch>(addr);

      let buddies_in_list = vector::empty<address>();
      let  i = 0;
      while (i < vector::length<address>(&v.my_buddies)) {
        let a = vector::borrow<address>(&v.my_buddies, i);
        if (vector::contains(&list, a)) {
          vector::push_back(&mut buddies_in_list, *a);
        };
        i = i + 1;
      };

      (buddies_in_list, vector::length<address>(&buddies_in_list))
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

    public fun unrelated_buddies_above_thresh(val: address): bool acquires Vouch{
      if (!exists<Vouch>(val)) return false;
      
      if (testnet::is_testnet()) return true;
      let vouches = borrow_global<Vouch>(val);

      let len = vector::length(&unrelated_buddies(vouches.my_buddies));
      (len >= globals::get_vouch_threshold())
    }

    #[test_only]
    public fun test_set_buddies(val: address, buddy_list: vector<address>) acquires Vouch {
      bulk_set(val, buddy_list);
    }
  }