
module ol_framework::vouch {
    use std::error;
    use std::signer;
    use std::vector;
    use ol_framework::ancestry;
    use ol_framework::ol_account;
    use ol_framework::epoch_helper;

    use diem_framework::account;
    use diem_framework::system_addresses;
    use diem_framework::transaction_fee;

    //use diem_std::debug::print;

    friend diem_framework::genesis;
    friend ol_framework::proof_of_fee;
    friend ol_framework::jail;
    friend ol_framework::epoch_boundary;
    friend ol_framework::vouch_migration; // TODO: remove after vouch migration is completed
    friend ol_framework::validator_universe;

    #[test_only]
    friend ol_framework::mock;
    #[test_only]
    friend ol_framework::test_pof;
    #[test_only]
    friend ol_framework::test_vouch;

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
    struct MyVouches has key {
      my_buddies: vector<address>, // incoming vouches
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

      if (!exists<MyVouches>(acc)) {
        move_to<MyVouches>(new_account_sig, MyVouches {
          my_buddies: vector::empty(),
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

    // WARNING: Val signer being forgded here for migration purpose!!!
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
        if (val != account) {
          let (incoming_vouches, epoch_vouched) = get_received_vouches(val);
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

    fun vouch_impl(grantor: &signer, friend_acc: address) acquires MyVouches, GivenVouches, VouchPrice {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_acc, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      // check if structures are initialized
      assert!(is_init(grantor_acc), error::invalid_state(EGRANTOR_NOT_INIT));
      assert!(is_init(friend_acc), error::invalid_state(ERECEIVER_NOT_INIT));

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
      add_given_vouches(grantor_acc, friend_acc, epoch);

      // add grantor to friend received vouches
      add_received_vouches(friend_acc, grantor_acc, epoch);
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
    public fun add_received_vouches(vouched_account: address, grantor_acc: address, epoch: u64) acquires MyVouches {
      let received_vouches = borrow_global_mut<MyVouches>(vouched_account);
      let (found, i) = vector::index_of(&received_vouches.my_buddies, &grantor_acc);

      if (found) {
        // Update the epoch if the vouching account is already present
        let epoch_value = vector::borrow_mut(&mut received_vouches.epoch_vouched, i);
        *epoch_value = epoch;
      } else {
        // Add new vouching account and epoch if not already present
        vector::push_back(&mut received_vouches.my_buddies, grantor_acc);
        vector::push_back(&mut received_vouches.epoch_vouched, epoch);
      }
    }

    /// will only succesfully vouch if the two are not related by ancestry
    /// prevents spending a vouch that would not be counted.
    /// to add a vouch and ignore this check use insist_vouch
    public entry fun vouch_for(grantor: &signer, wanna_be_my_friend: address) acquires MyVouches, GivenVouches, VouchPrice {
      ancestry::assert_unrelated(signer::address_of(grantor), wanna_be_my_friend);
      vouch_impl(grantor, wanna_be_my_friend);
    }

    /// you may want to add people who are related to you
    /// there are no known use cases for this at the moment.
    public entry fun insist_vouch_for(grantor: &signer, wanna_be_my_friend: address) acquires MyVouches, GivenVouches, VouchPrice {
      vouch_impl(grantor, wanna_be_my_friend);
    }

    public entry fun revoke(grantor: &signer, friend_acc: address) acquires MyVouches, GivenVouches {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_acc, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      if (!exists<MyVouches>(friend_acc)) return;

      // remove friend from grantor given vouches
      let v = borrow_global_mut<GivenVouches>(grantor_acc);
      let (found, i) = vector::index_of(&v.outgoing_vouches, &friend_acc);
      assert!(found, error::invalid_argument(EVOUCH_NOT_FOUND));
      vector::remove<address>(&mut v.outgoing_vouches, i);
      vector::remove<u64>(&mut v.epoch_vouched, i);

      // remove grantor from friends received vouches
      let v = borrow_global_mut<MyVouches>(friend_acc);
      let (found, i) = vector::index_of(&v.my_buddies, &grantor_acc);
      assert!(found, error::invalid_argument(EVOUCH_NOT_FOUND));
      vector::remove(&mut v.my_buddies, i);
      vector::remove(&mut v.epoch_vouched, i);
    }

    public(friend) fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires MyVouches {
      system_addresses::assert_ol(vm);
      bulk_set(val, buddy_list);
    }

    fun bulk_set(val: address, buddy_list: vector<address>) acquires MyVouches {
      if (!exists<MyVouches>(val)) return;

      // take self out of list
      let v = borrow_global_mut<MyVouches>(val);
      let (is_found, i) = vector::index_of(&buddy_list, &val);
      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };
      v.my_buddies = buddy_list;

      let epoch_data: vector<u64> = vector::map_ref(&buddy_list, |_e| { 0u64 } );
      v.epoch_vouched = epoch_data;
    }

    ///////// GETTERS //////////

    #[view]
    public fun is_init(acc: address ):bool {
      exists<MyVouches>(acc)
    }

    #[view]
    public fun get_received_vouches(acc: address): (vector<address>, vector<u64>) acquires MyVouches {
      if (!exists<MyVouches>(acc)) {
        return (vector::empty(), vector::empty())
      };

      let state = borrow_global<MyVouches>(acc);
      (*&state.my_buddies, *&state.epoch_vouched)
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
          // account might have dropped
          if (account::exists_at(buddy_acc)){
            if (is_not_expired(buddy_acc, state)) {
              vector::push_back(&mut valid_vouches, buddy_acc)
            }
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
    public(friend) fun true_friends_in_list(addr: address, list: &vector<address>): (vector<address>, u64) acquires MyVouches {
      if (!exists<MyVouches>(addr)) return (vector::empty(), 0);

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
    public fun test_set_buddies(val: address, buddy_list: vector<address>) acquires MyVouches {
      bulk_set(val, buddy_list);
    }
  }
