module ol_framework::vouch {
    use std::error;
    use std::signer;
    use std::vector;
    use ol_framework::ancestry;
    use ol_framework::epoch_helper;
    use diem_framework::system_addresses;

    // use diem_std::debug::print;

    friend diem_framework::genesis;
    friend ol_framework::proof_of_fee;
    friend ol_framework::jail;
    friend ol_framework::epoch_boundary;
    friend ol_framework::vouch_migration; // TODO: remove after vouch migration is completed
    friend ol_framework::filo_migration;
    friend ol_framework::validator_universe;
    friend ol_framework::vouch_txs;

    #[test_only]
    friend ol_framework::mock;
    #[test_only]
    friend ol_framework::test_pof;
    #[test_only]
    friend ol_framework::test_user_vouch;
    #[test_only]
    friend ol_framework::test_validator_vouch;
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


    // a user should not be able to give more vouches than valid
    // vouches received. An allowance of +1 more than the
    // current given vouches allows a vouch to be given before received
    fun assert_max_vouches(grantor_acc: address) acquires GivenVouches, ReceivedVouches {
      // check if the grantor has already reached the limit of vouches
      let given_vouches = get_given_vouches_not_expired(grantor_acc);
      let received_vouches = get_received_vouches_not_expired(grantor_acc);
      assert!(
        vector::length(&given_vouches) <= (vector::length(&received_vouches) + 1),
        error::invalid_state(EMAX_LIMIT_GIVEN)
      );
    }

    // lazily remove expired vouches prior to setting any new ones
    // ... by popular request
    fun garbage_collect_expired(user: address) acquires GivenVouches, ReceivedVouches {
      let valid_vouches = vector::empty<address>();
      let valid_epochs = vector::empty<u64>();
      let current_epoch = epoch_helper::get_current_epoch();

      let state = borrow_global_mut<GivenVouches>(user);

      let i = 0;
      while (i < vector::length(&state.outgoing_vouches)) {
        let vouch_received = vector::borrow(&state.outgoing_vouches, i);
        let when_vouched = *vector::borrow(&state.epoch_vouched, i);
        // check if the vouch is expired
        if ((when_vouched + EXPIRATION_ELAPSED_EPOCHS) > current_epoch) {
          vector::push_back(&mut valid_vouches, *vouch_received);
          vector::push_back(&mut valid_epochs, when_vouched);

        };
        i = i + 1;
      };
      state.outgoing_vouches = valid_vouches;
      state.epoch_vouched = valid_epochs;

      //////// RECEIVED ////////
      // TODO: Gross code duplication, only because field incoming_vouches is different, otherwise could be generic type.
      let valid_vouches = vector::empty<address>();
      let valid_epochs = vector::empty<u64>();
      let current_epoch = epoch_helper::get_current_epoch();

      let state = borrow_global_mut<ReceivedVouches>(user);

      let i = 0;
      while (i < vector::length(&state.incoming_vouches)) {
        let vouch_received = vector::borrow(&state.incoming_vouches, i);
        let when_vouched = *vector::borrow(&state.epoch_vouched, i);
        // check if the vouch is expired
        if ((when_vouched + EXPIRATION_ELAPSED_EPOCHS) > current_epoch) {
          vector::push_back(&mut valid_vouches, *vouch_received);
          vector::push_back(&mut valid_epochs, when_vouched);

        };
        i = i + 1;
      };
      state.incoming_vouches = valid_vouches;
      state.epoch_vouched = valid_epochs;
    }

    fun vouch_impl(grantor: &signer, friend_account: address, check_unrelated: bool) acquires ReceivedVouches, GivenVouches {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_account, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      // check if structures are initialized
      assert!(is_init(grantor_acc), error::invalid_state(EGRANTOR_NOT_INIT));
      assert!(is_init(friend_account), error::invalid_state(ERECEIVER_NOT_INIT));

      // while we are here.
      garbage_collect_expired(grantor_acc);
      garbage_collect_expired(friend_account);


      if (check_unrelated) {
        ancestry::assert_unrelated(grantor_acc, friend_account);
      };

      assert_max_vouches(grantor_acc);

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
    public(friend) fun vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches {
      vouch_impl(grantor, friend_account, true);
    }

    public(friend) fun revoke(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches {
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

      // print(&string::utf8(b">>> Migrated GivenVouches for "));
      // print(&account);
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

      // print(&string::utf8(b">>> Migrated ReceivedVouches for "));
      // print(&account);
    }

    ///////// GETTERS //////////

    #[view]
    public fun is_init(acc: address ):bool {
      exists<ReceivedVouches>(acc) && exists<GivenVouches>(acc)
    }

    #[view]
    /// @return (vector of buddies, vector of epoch when vouched)
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
    // Deprecation notice, function will renamed, pass through until then.
    /// gets the received vouches not expired
    public fun all_not_expired(addr: address): vector<address> acquires ReceivedVouches {
      get_received_vouches_not_expired(addr)
    }


    fun filter_expired(list_addr: vector<address>, epoch_vouched: vector<u64>): vector<address> {
      let valid_vouches = vector::empty<address>();

      let current_epoch = epoch_helper::get_current_epoch();

      let i = 0;
      while (i < vector::length(&list_addr)) {
        let vouch_received = vector::borrow(&list_addr, i);
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
    /// gets the received vouches not expired
    public fun get_received_vouches_not_expired(addr: address): vector<address> acquires ReceivedVouches {

      let (all_received, epoch_vouched) = get_received_vouches(addr);
      filter_expired(all_received, epoch_vouched)
    }

    #[view]
    /// gets the given vouches not expired
    public fun get_given_vouches_not_expired(addr: address): vector<address> acquires GivenVouches {
      let (all_received, epoch_vouched) = get_given_vouches(addr);
      filter_expired(all_received, epoch_vouched)
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
    /// you may want to add people who are related to you
    /// there are no known use cases for this at the moment.
    public(friend) fun insist_vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches {
      vouch_impl(grantor, friend_account, false);
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
