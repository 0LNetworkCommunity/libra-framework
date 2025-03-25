
module ol_framework::vouch {
    use std::error;
    use std::signer;
    use std::vector;
    use std::string;
    use ol_framework::ancestry;
    use ol_framework::ol_account;
    use ol_framework::epoch_helper;

    use diem_framework::system_addresses;
    use diem_framework::transaction_fee;

    use diem_std::debug::print;

    friend diem_framework::genesis;
    friend ol_framework::proof_of_fee;
    friend ol_framework::jail;
    friend ol_framework::epoch_boundary;
    friend ol_framework::vouch_migration; // TODO: remove after vouch migration is completed
    friend ol_framework::filo_migration;
    friend ol_framework::validator_universe;

    #[test_only]
    friend ol_framework::mock;
    #[test_only]
    friend ol_framework::test_pof;
    #[test_only]
    friend ol_framework::test_vouch;
    #[test_only]
    friend ol_framework::test_vouch_migration;

    //////// CONST ////////

    /// How many epochs must pass before the voucher expires.
    const EXPIRATION_ELAPSED_EPOCHS: u64 = 45;

    /// Maximum number of vouches
    const BASE_MAX_VOUCHES: u64 = 10;


    //////// ERROR CODES ////////

    /// Trying to vouch for yourself?
    const ETRY_SELF_VOUCH_REALLY: u64 = 1;

    /// User cannot receive vouch because their vouch state is not initialized
    const ERECEIVER_NOT_INIT: u64 = 2;

    /// Cannot query vouches because the GivenVouches state is not initialized
    const EGIVEN_VOUCHES_NOT_INIT: u64 = 3;

    /// Limit reached. You cannot give any new vouches.
    const EMAX_LIMIT_GIVEN: u64 = 4;

    /// Vouch not found
    const EVOUCH_NOT_FOUND: u64 = 5;

    /// Grantor state is not initialized
    const EGRANTOR_NOT_INIT: u64 = 6;


    //////// STRUCTS ////////

    // TODO: someday this should be renamed to ReceivedVouches
    struct MyVouches has key, drop {
      my_buddies: vector<address>, // incoming vouches
      epoch_vouched: vector<u64>,
    }

    struct ReceivedVouches has key {
      incoming_vouches: vector<address>,
      epoch_vouched: vector<u64>,
    }

    struct GivenVouches has key {
      outgoing_vouches: vector<address>,
      epoch_vouched: vector<u64>,
    }

    /// struct to store the price to vouch for someone
    struct VouchPrice has key {
      amount: u64
    }

    /*
    Oh, I get by with a little help from my friends,
    Mm, I get high with a little help from my friends,
    Mm, gonna try with a little help from my friends.
    */

    // init the struct on a validators account.
    public(friend) fun init(new_account_sig: &signer) {
      let acc = signer::address_of(new_account_sig);

      if (exists<MyVouches>(acc)) {
        // let migration handle the initialization of new structs
        return
      };

      if (!exists<ReceivedVouches>(acc)) {
        move_to<ReceivedVouches>(new_account_sig, ReceivedVouches {
          incoming_vouches: vector::empty(),
          epoch_vouched: vector::empty(),
        });
      };

      if (!exists<GivenVouches>(acc)) {
        move_to<GivenVouches>(new_account_sig, GivenVouches {
          outgoing_vouches: vector::empty(),
          epoch_vouched: vector::empty(),
        });
      };
    }

    public(friend) fun set_vouch_price(framework: &signer, amount: u64) acquires VouchPrice {
      system_addresses::assert_ol(framework);
      if (!exists<VouchPrice>(@ol_framework)) {
        move_to(framework, VouchPrice { amount });
      } else {
        let price = borrow_global_mut<VouchPrice>(@ol_framework);
        price.amount = amount;
      };
    }

    fun vouch_impl(grantor: &signer, friend_account: address, check_unrelated: bool) acquires ReceivedVouches, GivenVouches, VouchPrice {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_account, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      // check if structures are initialized
      assert!(is_init(grantor_acc), error::invalid_state(EGRANTOR_NOT_INIT));
      assert!(is_init(friend_account), error::invalid_state(ERECEIVER_NOT_INIT));

      if (check_unrelated) {
        ancestry::assert_unrelated(grantor_acc, friend_account);
      };

      // check if the grantor has already reached the limit of vouches
      let (given_vouches, _) = get_given_vouches(grantor_acc);
      assert!(
        vector::length(&given_vouches) < BASE_MAX_VOUCHES,
        error::invalid_state(EMAX_LIMIT_GIVEN)
      );

      // this fee is paid to the system, cannot be reclaimed
      let price = get_vouch_price();
      if (price > 0) {
        let vouch_cost = ol_account::withdraw(grantor, price);
        transaction_fee::user_pay_fee(grantor, vouch_cost);
      };

      let epoch = epoch_helper::get_current_epoch();

      // add friend to grantor given vouches
      add_given_vouches(grantor_acc, friend_account, epoch);

      // add grantor to friend received vouches
      add_received_vouches(friend_account, grantor_acc, epoch);
    }

    // Function to add given vouches
    public fun add_given_vouches(grantor_acc: address, vouched_account: address, epoch: u64) acquires GivenVouches {
      let given_vouches = borrow_global_mut<GivenVouches>(grantor_acc);
      let (found, i) = vector::index_of(&given_vouches.outgoing_vouches, &vouched_account);

      if (found) {
        // Update the epoch if the vouched account is already present
        let epoch_value = vector::borrow_mut(&mut given_vouches.epoch_vouched, i);
        *epoch_value = epoch;
      } else {
        // Add new vouched account and epoch if not already present
        vector::push_back(&mut given_vouches.outgoing_vouches, vouched_account);
        vector::push_back(&mut given_vouches.epoch_vouched, epoch);
      }
    }

    // Function to add received vouches
    public fun add_received_vouches(vouched_account: address, grantor_acc: address, epoch: u64) acquires ReceivedVouches {
      let received_vouches = borrow_global_mut<ReceivedVouches>(vouched_account);
      let (found, i) = vector::index_of(&received_vouches.incoming_vouches, &grantor_acc);

      if (found) {
        // Update the epoch if the vouching account is already present
        let epoch_value = vector::borrow_mut(&mut received_vouches.epoch_vouched, i);
        *epoch_value = epoch;
      } else {
        // Add new vouching account and epoch if not already present
        vector::push_back(&mut received_vouches.incoming_vouches, grantor_acc);
        vector::push_back(&mut received_vouches.epoch_vouched, epoch);
      }
    }

    /// will only successfully vouch if the two are not related by ancestry
    /// prevents spending a vouch that would not be counted.
    /// to add a vouch and ignore this check use insist_vouch
    public entry fun vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchPrice {
      vouch_impl(grantor, friend_account, true);
    }

    /// you may want to add people who are related to you
    /// there are no known use cases for this at the moment.
    public entry fun insist_vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchPrice {
      vouch_impl(grantor, friend_account, false);
    }

    public entry fun revoke(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_account, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      assert!(is_init(grantor_acc), error::invalid_state(EGRANTOR_NOT_INIT));

      // remove friend from grantor given vouches
      let v = borrow_global_mut<GivenVouches>(grantor_acc);
      let (found, i) = vector::index_of(&v.outgoing_vouches, &friend_account);
      assert!(found, error::invalid_argument(EVOUCH_NOT_FOUND));
      vector::remove<address>(&mut v.outgoing_vouches, i);
      vector::remove<u64>(&mut v.epoch_vouched, i);

      // remove grantor from friends received vouches
      let v = borrow_global_mut<ReceivedVouches>(friend_account);
      let (found, i) = vector::index_of(&v.incoming_vouches, &grantor_acc);
      assert!(found, error::invalid_argument(EVOUCH_NOT_FOUND));
      vector::remove(&mut v.incoming_vouches, i);
      vector::remove(&mut v.epoch_vouched, i);
    }

    public(friend) fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires ReceivedVouches {
      system_addresses::assert_ol(vm);
      bulk_set(val, buddy_list);
    }

    fun bulk_set(val: address, buddy_list: vector<address>) acquires ReceivedVouches {
      if (!exists<ReceivedVouches>(val)) return;

      // take self out of list
      let v = borrow_global_mut<ReceivedVouches>(val);
      let (is_found, i) = vector::index_of(&buddy_list, &val);
      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };
      v.incoming_vouches = buddy_list;

      let epoch_data: vector<u64> = vector::map_ref(&buddy_list, |_e| { 0u64 } );
      v.epoch_vouched = epoch_data;
    }

    // The struct GivenVouches cannot not be lazy initialized because
    // vouch module cannot depend on validator_universe (circle dependency)
    // TODO: this migration function must be removed after all validators have migrated
    public(friend) fun migrate_given_vouches(account_sig: &signer, all_accounts: vector<address>) acquires MyVouches {
      let account = signer::address_of(account_sig);

      if (exists<GivenVouches>(account)) {
        // struct already initialized
        return
      };

      let new_outgoing_vouches = vector::empty();
      let new_epoch_vouched = vector::empty();

      vector::for_each(all_accounts, |val| {
        if (val != account && exists<MyVouches>(val)) {
          let legacy_state = borrow_global<MyVouches>(val);
          let incoming_vouches = legacy_state.my_buddies;
          let epoch_vouched = legacy_state.epoch_vouched;
          let (found, i) = vector::index_of(&incoming_vouches, &account);
          if (found) {
            vector::push_back(&mut new_outgoing_vouches, val);
            vector::push_back(&mut new_epoch_vouched, *vector::borrow(&epoch_vouched, i));
          }
        }
      });

      // add the new GivenVouches struct with the new data
      move_to<GivenVouches>(account_sig, GivenVouches {
        outgoing_vouches: new_outgoing_vouches,
        epoch_vouched: new_epoch_vouched,
      });

      print(&string::utf8(b">>> Migrated GivenVouches for "));
      print(&account);
    }

    // TODO: remove/deprecate after migration is completed
    public(friend) fun migrate_received_vouches(account_sig: &signer) acquires MyVouches {
      let account = signer::address_of(account_sig);

      if (!exists<MyVouches>(account)) {
        // struct not initialized
        return
      };

      if (exists<ReceivedVouches>(account)) {
        // struct already initialized
        return
      };

      let state = borrow_global<MyVouches>(account);
      let new_incoming_vouches = state.my_buddies;
      let new_epoch_vouched = state.epoch_vouched;

      // add the new ReceivedVouches struct with the legacy data
      move_to<ReceivedVouches>(account_sig, ReceivedVouches {
        incoming_vouches: new_incoming_vouches,
        epoch_vouched: new_epoch_vouched,
      });

      // remove deprecated MyVouches struct
      move_from<MyVouches>(account);

      print(&string::utf8(b">>> Migrated ReceivedVouches for "));
      print(&account);
    }

    ///////// GETTERS //////////

    #[view]
    public fun is_init(acc: address ):bool {
      exists<ReceivedVouches>(acc) && exists<GivenVouches>(acc)
    }

    #[view]
    public fun get_received_vouches(acc: address): (vector<address>, vector<u64>) acquires ReceivedVouches {
      if (!exists<ReceivedVouches>(acc)) {
        return (vector::empty(), vector::empty())
      };

      let state = borrow_global<ReceivedVouches>(acc);
      (*&state.incoming_vouches, *&state.epoch_vouched)
    }

    #[view]
    public fun get_given_vouches(acc: address): (vector<address>, vector<u64>) acquires GivenVouches {
      assert!(exists<GivenVouches>(acc), error::invalid_state(EGIVEN_VOUCHES_NOT_INIT));

      let state = borrow_global<GivenVouches>(acc);
      (*&state.outgoing_vouches, *&state.epoch_vouched)
    }

    #[view]
    public fun get_vouch_price(): u64 acquires VouchPrice {
      if (!exists<VouchPrice>(@ol_framework)) {
        return 1_000 // previous micro-libra cost
      };

      let price = borrow_global<VouchPrice>(@ol_framework);
      price.amount
    }

    #[view]
    /// gets all buddies, including expired ones
    public fun all_vouchers(val: address): vector<address> acquires ReceivedVouches {
      let (incoming_vouches, _) = get_received_vouches(val);
      incoming_vouches
    }

    #[view]
    /// gets the received vouches not expired
    public fun all_not_expired(addr: address): vector<address> acquires ReceivedVouches {
      let valid_vouches = vector::empty<address>();

      let (all_received, epoch_vouched) = get_received_vouches(addr);
      let current_epoch = epoch_helper::get_current_epoch();

      let i = 0;
      while (i < vector::length(&all_received)) {
        let vouch_received = vector::borrow(&all_received, i);
        let when_vouched = *vector::borrow(&epoch_vouched, i);
        // check if the vouch is expired
        if ((when_vouched + EXPIRATION_ELAPSED_EPOCHS) > current_epoch) {
          vector::push_back(&mut valid_vouches, *vouch_received)
        };
        i = i + 1;
      };

      valid_vouches
    }

    #[view]
    /// filter expired vouches, and do ancestry check
    public fun true_friends(addr: address): vector<address> acquires ReceivedVouches {
      if (!exists<ReceivedVouches>(addr)) return vector::empty<address>();
      let not_expired = all_not_expired(addr);
      let filtered_ancestry = ancestry::list_unrelated(not_expired);
      filtered_ancestry
    }

    #[view]
    /// check if the user is in fact a valid voucher
    public fun is_valid_voucher_for(voucher: address, recipient: address):bool
    acquires ReceivedVouches {
      let list = true_friends(recipient);
      vector::contains(&list, &voucher)
    }

    /// for a given list find and count any of my vouchers
    public(friend) fun true_friends_in_list(addr: address, list: &vector<address>): (vector<address>, u64) acquires ReceivedVouches {
      if (!exists<ReceivedVouches>(addr)) return (vector::empty(), 0);

      let tf = true_friends(addr);
      let buddies_in_list = vector::empty();
      let i = 0;
      while (i < vector::length(&tf)) {
        let addr = vector::borrow(&tf, i);
        if (vector::contains(list, addr)) {
          vector::push_back(&mut buddies_in_list, *addr);
        };
        i = i + 1;
      };

      (buddies_in_list, vector::length(&buddies_in_list))
    }

    #[test_only]
    public fun test_set_buddies(val: address, buddy_list: vector<address>) acquires ReceivedVouches {
      bulk_set(val, buddy_list);
    }

    #[test_only]
    public fun legacy_init(new_account_sig: &signer) {
      let acc = signer::address_of(new_account_sig);

      if (!exists<MyVouches>(acc)) {
        move_to<MyVouches>(new_account_sig, MyVouches {
          my_buddies: vector::empty(),
          epoch_vouched: vector::empty(),
        });
      };
    }

    #[test_only]
    public fun legacy_vouch_for(ill_be_your_friend: &signer, wanna_be_my_friend: address) acquires MyVouches {
      let buddy_acc = signer::address_of(ill_be_your_friend);
      assert!(buddy_acc != wanna_be_my_friend, ETRY_SELF_VOUCH_REALLY);

      if (!exists<MyVouches>(wanna_be_my_friend)) return;
      let epoch = epoch_helper::get_current_epoch();

      // this fee is paid to the system, cannot be reclaimed
      // let c = ol_account::withdraw(ill_be_your_friend, vouch_cost_microlibra());
      // transaction_fee::user_pay_fee(ill_be_your_friend, c);

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

    #[test_only]
    public fun get_legacy_vouches(acc: address): (vector<address>, vector<u64>) acquires MyVouches {
      if (!exists<MyVouches>(acc)) {
        return (vector::empty(), vector::empty())
      };

      let state = borrow_global<MyVouches>(acc);
      (*&state.my_buddies, *&state.epoch_vouched)
    }

    #[test_only]
    public fun is_legacy_init(acc: address): bool {
      exists<MyVouches>(acc)
    }
  }
