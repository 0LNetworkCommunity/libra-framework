
module ol_framework::vouch {
    use std::signer;
    use std::vector;
    use ol_framework::ancestry;
    use ol_framework::ol_account;
    use ol_framework::epoch_helper;

    use diem_framework::system_addresses;
    use diem_framework::transaction_fee;

    friend diem_framework::genesis;

    /// trying to vouch for yourself?
    const ETRY_SELF_VOUCH_REALLY: u64 = 1;

    /// how many epochs must pass before the voucher expires.
    const EXPIRATION_ELAPSED_EPOCHS: u64 = 90;

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

    #[view]
    public fun is_init(acc: address ):bool {
      exists<MyVouches>(acc)
    }

    fun vouch_impl(ill_be_your_friend: &signer, wanna_be_my_friend: address) acquires MyVouches {
      let buddy_acc = signer::address_of(ill_be_your_friend);
      assert!(buddy_acc != wanna_be_my_friend, ETRY_SELF_VOUCH_REALLY);

      if (!exists<MyVouches>(wanna_be_my_friend)) return;
      let epoch = epoch_helper::get_current_epoch();
      // this fee is paid to the system, cannot be reclaimed
      let c = ol_account::withdraw(ill_be_your_friend, vouch_cost_microlibra());
      transaction_fee::user_pay_fee(ill_be_your_friend, c);

      let v = borrow_global_mut<MyVouches>(wanna_be_my_friend);

      let (found, i) = vector::index_of(&v.my_buddies, &buddy_acc);
      if (found) { // prevent duplicates
        // update date
        let e = vector::borrow_mut(&mut v.epoch_vouched, i);
        *e = epoch;
      } else {
        vector::push_back(&mut v.my_buddies, buddy_acc);
        vector::push_back(&mut v.epoch_vouched, epoch);
      }
    }

    /// will only succesfully vouch if the two are not related by ancestry
    /// prevents spending a vouch that would not be counted.
    /// to add a vouch and ignore this check use insist_vouch
    public entry fun vouch_for(grantor: &signer, wanna_be_my_friend: address) acquires MyVouches {
      ancestry::assert_unrelated(signer::address_of(grantor), wanna_be_my_friend);
      vouch_impl(grantor, wanna_be_my_friend);
    }

    /// you may want to add people who are related to you
    /// there are no known use cases for this at the moment.
    public entry fun insist_vouch_for(grantor: &signer, wanna_be_my_friend: address) acquires MyVouches {
      vouch_impl(grantor, wanna_be_my_friend);
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

    public(friend) fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires MyVouches {
      system_addresses::assert_ol(vm);
      bulk_set(val, buddy_list);
    }

    fun bulk_set(val: address, buddy_list: vector<address>) acquires MyVouches {

      if (!exists<MyVouches>(val)) return;

      let v = borrow_global_mut<MyVouches>(val);

      // take self out of list
      let (is_found, i) = vector::index_of(&buddy_list, &val);

      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };

      v.my_buddies = buddy_list;

      let epoch_data: vector<u64> = vector::map_ref(&buddy_list, |_e| { 0u64 } );
      v.epoch_vouched = epoch_data;
    }

    #[view]
    /// gets all buddies, including expired ones
    public fun all_vouchers(val: address): vector<address> acquires MyVouches{

      if (!exists<MyVouches>(val)) return vector::empty<address>();
      let state = borrow_global<MyVouches>(val);
      *&state.my_buddies
    }

    #[view]
    /// gets the buddies and checks if they are expired
    public fun all_not_expired(addr: address): vector<address> acquires MyVouches{
      let valid_vouches = vector::empty<address>();
      if (is_init(addr)) {
        let state = borrow_global<MyVouches>(addr);
        vector::for_each(state.my_buddies, |buddy_acc| {
          if (is_not_expired(buddy_acc, state)) {
            vector::push_back(&mut valid_vouches, buddy_acc)
          }
        })
      };
      valid_vouches
    }

    #[view]
    /// filter expired vouches, and do ancestry check
    public fun true_friends(addr: address): vector<address> acquires MyVouches{

      if (!exists<MyVouches>(addr)) return vector::empty<address>();
      let not_expired = all_not_expired(addr);
      let filtered_ancestry = ancestry::list_unrelated(not_expired);
      filtered_ancestry
    }

    #[view]
    /// check if the user is in fact a valid voucher
    public fun is_valid_voucher_for(voucher: address, recipient: address):bool
    acquires MyVouches {
      let list = true_friends(recipient);
      vector::contains(&list, &voucher)
    }


    fun is_not_expired(voucher: address, state: &MyVouches): bool {
      let (found, i) = vector::index_of(&state.my_buddies, &voucher);
      if (found) {
        let when_vouched = vector::borrow(&state.epoch_vouched, i);
        return  (*when_vouched + EXPIRATION_ELAPSED_EPOCHS) > epoch_helper::get_current_epoch()
      };
      false
    }

    /// for a given list find and count any of my vouchers
    public fun true_friends_in_list(addr: address, list: &vector<address>): (vector<address>, u64) acquires MyVouches {

      if (!exists<MyVouches>(addr)) return (vector::empty(), 0);

      let tf = true_friends(addr);

      let buddies_in_list = vector::empty();
      let  i = 0;
      while (i < vector::length(&tf)) {
        let addr = vector::borrow(&tf, i);

        if (vector::contains(list, addr)) {
          vector::push_back(&mut buddies_in_list, *addr);
        };
        i = i + 1;
      };

      (buddies_in_list, vector::length(&buddies_in_list))
    }


    // TODO: move to globals
    // the cost to verify a vouch. Coins are burned.
    fun vouch_cost_microlibra(): u64 {
      1000
    }

    #[test_only]
    public fun test_set_buddies(val: address, buddy_list: vector<address>) acquires MyVouches {
      bulk_set(val, buddy_list);
    }
  }