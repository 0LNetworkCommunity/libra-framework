
module ol_framework::vouch {
    use std::signer;
    use std::vector;
    use ol_framework::validator_universe;
    use ol_framework::ancestry;
    use ol_framework::globals;
    use aptos_framework::system_addresses;
    use aptos_framework::stake;

    // triggered once per epoch
    struct Vouch has key {
      vals: vector<address>,
    }

    // init the struct on a validators account.
    public fun init(new_account_sig: &signer ) {
      let acc = signer::address_of(new_account_sig);

      if (validator_universe::is_in_universe(acc) && !is_init(acc)) {
        move_to<Vouch>(new_account_sig, Vouch {
            vals: vector::empty(), 
          });
      }
    }

    public fun is_init(acc: address ):bool {
      exists<Vouch>(acc)
    }

    public fun vouch_for(buddy: &signer, val: address) acquires Vouch {
      let buddy_acc = signer::address_of(buddy);
      assert!(buddy_acc!=val, 12345); // TODO: Error code.

      if (!validator_universe::is_in_universe(buddy_acc)) return;
      if (!exists<Vouch>(val)) return;

      let v = borrow_global_mut<Vouch>(val);
      if (!vector::contains(&v.vals, &buddy_acc)) { // prevent duplicates
        vector::push_back<address>(&mut v.vals, buddy_acc);
      }
    }

    public fun revoke(buddy: &signer, val: address) acquires Vouch {
      let buddy_acc = signer::address_of(buddy);
      assert!(buddy_acc!=val, 12345); // TODO: Error code.

      if (!exists<Vouch>(val)) return;

      let v = borrow_global_mut<Vouch>(val);
      let (found, i) = vector::index_of(&v.vals, &buddy_acc);
      if (found) {
        vector::remove(&mut v.vals, i);
      };
    }    

    public fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires Vouch {
      system_addresses::assert_vm(vm);

      if (!validator_universe::is_in_universe(val)) return;
      if (!exists<Vouch>(val)) return;

      let v = borrow_global_mut<Vouch>(val);

      // take self out of list
      let (is_found, i) = vector::index_of(&buddy_list, &val);

      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };
      
      v.vals = buddy_list;
    }

    public fun get_buddies(val: address): vector<address> acquires Vouch{
      if (is_init(val)) {
        return *&borrow_global<Vouch>(val).vals
      };
      vector::empty<address>()
    }

    public fun buddies_in_set(val: address): vector<address> acquires Vouch {
      let current_set = stake::get_current_validators();
      if (!exists<Vouch>(val)) return vector::empty<address>();

      let v = borrow_global<Vouch>(val);

      let buddies_in_set = vector::empty<address>();
      let  i = 0;
      while (i < vector::length<address>(&v.vals)) {
        let a = vector::borrow<address>(&v.vals, i);
        if (vector::contains(&current_set, a)) {
          vector::push_back(&mut buddies_in_set, *a);
        };
        i = i + 1;
      };

      buddies_in_set
    }

    public fun unrelated_buddies(val: address): vector<address> acquires Vouch {
      // start our list empty
      let unrelated_buddies = vector::empty<address>();

      // find all our buddies in this validator set
      let buddies_in_set = buddies_in_set(val);
      let len = vector::length<address>(&buddies_in_set);
      let  i = 0;
      while (i < len) {
      
        let target_acc = vector::borrow<address>(&buddies_in_set, i);

        // now loop through all the accounts again, and check if this target
        // account is related to anyone.
        let  k = 0;
        while (k < vector::length<address>(&buddies_in_set)) {
          let comparison_acc = vector::borrow(&buddies_in_set, k);
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
      
        // if (vector::contains(&current_set, a)) {
        //   vector::push_back(&mut buddies_in_set, *a);
        // };
        i = i + 1;
      };

      unrelated_buddies
    }

    public fun unrelated_buddies_above_thresh(val: address): bool acquires Vouch{
      if (!exists<Vouch>(val)) return false;

      let len = vector::length(&unrelated_buddies(val));
      (len >= globals::get_vouch_threshold())
    }
  }