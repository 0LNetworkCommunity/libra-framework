module ol_framework::vouch {
    use std::error;
    use std::signer;
    use std::vector;
    use ol_framework::ancestry;
    use ol_framework::epoch_helper;
    use diem_framework::system_addresses;

    friend diem_framework::genesis;
    friend ol_framework::proof_of_fee;
    friend ol_framework::jail;
    friend ol_framework::epoch_boundary;
    friend ol_framework::filo_migration;
    friend ol_framework::validator_universe;
    friend ol_framework::vouch_txs;
    friend ol_framework::page_rank_lazy;
    friend ol_framework::validator_vouch;

    #[test_only]
    friend ol_framework::mock;
    #[test_only]
    friend ol_framework::test_pof;
    #[test_only]
    friend ol_framework::test_user_vouch;
    #[test_only]
    friend ol_framework::test_validator_vouch;
    #[test_only]
    friend ol_framework::test_page_rank;
    #[test_only]
    friend ol_framework::test_permanent_vouches;

    //////// CONST ////////

    /// How many epochs must pass before the voucher expires.
    const EXPIRATION_ELAPSED_EPOCHS: u64 = 45;


    /*
        Anti-Sybil Protection:

        The vouch system needs protection against sybil attack vectors. A malicious actor could:
        1. Obtain vouches from honest users
        2. Use those vouches to create many sybil identities by quickly vouching for and revoking from
           accounts they control
        3. Use these identities to gain undue influence (e.g., for Founder status)

        To prevent this, we implement:
        - A limit on revocations per epoch
        - A cooldown period after revocation before new vouches can be given
        - Lifetime tracking of vouching activity

        This ensures that even if a malicious user receives multiple valid vouches, they cannot
        rapidly reuse those vouches to create many sybil identities.
    */

    /*
        Anti-Sybil Protection:

        The vouch system needs protection against sybil attack vectors. A malicious actor could:
        1. Obtain vouches from honest users
        2. Use those vouches to create many sybil identities by quickly vouching for and revoking from
           accounts they control
        3. Use these identities to gain undue influence (e.g., for Founder status)

        To prevent this, we implement:
        - A limit on revocations per epoch
        - A cooldown period after revocation before new vouches can be given
        - Lifetime tracking of vouching activity

        This ensures that even if a malicious user receives multiple valid vouches, they cannot
        rapidly reuse those vouches to create many sybil identities.
    */

    // Add these constants for revocation limits
    /// Maximum number of revocations allowed in a single epoch
    const MAX_REVOCATIONS_PER_EPOCH: u64 = 2;

    /// Cooldown period (in epochs) required after a revocation before giving a new vouch
    const REVOCATION_COOLDOWN_EPOCHS: u64 = 3;

    //////// ERROR CODES ////////

    /// Trying to vouch for yourself?
    const ETRY_SELF_VOUCH_REALLY: u64 = 1;

    /// User cannot receive vouch because their vouch state is not initialized
    const ERECEIVER_NOT_INIT: u64 = 2;

    /// Cannot query vouches because the GivenVouches state is not initialized
    const EGIVEN_VOUCHES_NOT_INIT: u64 = 3;

    /// Vouch not found
    const EVOUCH_NOT_FOUND: u64 = 5;

    /// Grantor state is not initialized
    const EGRANTOR_NOT_INIT: u64 = 6;

    /// Revocation limit reached. You cannot revoke any more vouches in this epoch.
    const EREVOCATION_LIMIT_REACHED: u64 = 7;

    /// Cooldown period after revocation not yet passed
    const ECOOLDOWN_PERIOD_ACTIVE: u64 = 8;

    /// Vouch limit reached: above max ceiling
    const EMAX_LIMIT_GIVEN_CEILING: u64 = 9;

    /// Vouch limit reached: above number of vouches received
    const EMAX_LIMIT_GIVEN_BY_RECEIPT: u64 = 10;

    /// Vouch limit reached: because of quality of your voucher
    const EMAX_LIMIT_GIVEN_BY_SCORE: u64 = 11;

    /// Vouch limit reached: too many given in current epoch
    const EMAX_VOUCHES_PER_EPOCH: u64 = 12;


    //////// STRUCTS ////////

    // // TODO: someday this should be renamed to ReceivedVouches
    // struct MyVouches has key, drop {
    //   my_buddies: vector<address>, // incoming vouches
    //   epoch_vouched: vector<u64>,
    // }

    // Make this public for vouch_metrics.move access
    struct ReceivedVouches has key {
      incoming_vouches: vector<address>,
      epoch_vouched: vector<u64>,
    }

    struct GivenVouches has key {
      outgoing_vouches: vector<address>,
      epoch_vouched: vector<u64>,
    }

    // TODO: this struct should be merged
    // with GivenVouches whenever a cleanup is possible
    /// Every period a voucher can get a limit
    /// of vouches to give.
    /// But also limit the revocations
    struct VouchesLifetime has key {
      given: u64,
      given_revoked: u64,
      received: u64,
      received_revoked: u64, // TODO: currently not tracked
      last_revocation_epoch: u64,      // Track when last revocation happened
      revocations_this_epoch: u64,     // Count revocations in current epoch
      last_given_epoch: u64,      // Current epoch being tracked
      given_this_epoch: u64,         // Count vouches in current epoch
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
      // if (exists<MyVouches>(acc)) {
      //   // let migration handle the initialization of new structs
      //   return
      // };

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

      if (!exists<VouchesLifetime>(acc)) {
        move_to<VouchesLifetime>(new_account_sig, VouchesLifetime {
          given: 0,
          given_revoked: 0,
          received: 0,
          received_revoked: 0,
          last_revocation_epoch: 0,
          revocations_this_epoch: 0,
          given_this_epoch: 0,
          last_given_epoch: epoch_helper::get_current_epoch(),
        });
      };
    }

    /// Remove a vouch between voucher and vouchee, updating both states
    public(friend) fun remove_vouch(voucher: address, vouchee: address) acquires GivenVouches, ReceivedVouches {
        // Remove vouchee from voucher's outgoing vouches
        if (exists<GivenVouches>(voucher)) {
            let gv = borrow_global_mut<GivenVouches>(voucher);
            let (found, i) = vector::index_of(&gv.outgoing_vouches, &vouchee);
            if (found) {
                vector::remove<address>(&mut gv.outgoing_vouches, i);
                vector::remove<u64>(&mut gv.epoch_vouched, i);
            }
        };
        // Remove voucher from vouchee's incoming vouches
        if (exists<ReceivedVouches>(vouchee)) {
            let rv = borrow_global_mut<ReceivedVouches>(vouchee);
            let (found, i) = vector::index_of(&rv.incoming_vouches, &voucher);
            if (found) {
                vector::remove<address>(&mut rv.incoming_vouches, i);
                vector::remove<u64>(&mut rv.epoch_vouched, i);
            }
        }
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

    /// REVOCATION CHECKS
    /// within a period a user might try to add and revoke
    /// many users. As such there are some checks to make on
    /// revocation.
    /// 1. Over a lifetime of the account you cannot revoke more
    /// than you have vouched for.
    /// 2. You cannot revoke more times than the current
    /// amount of vouches you currently have received.


    fun vouch_impl(grantor: &signer, friend_account: address, check_unrelated: bool) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_account, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      // check if structures are initialized
      assert!(is_init(grantor_acc), error::invalid_state(EGRANTOR_NOT_INIT));
      assert!(is_init(friend_account), error::invalid_state(ERECEIVER_NOT_INIT));



      // // while we are here.
      // garbage_collect_expired(grantor_acc);
      // garbage_collect_expired(friend_account);


      if (check_unrelated) {
        ancestry::assert_unrelated(grantor_acc, friend_account);
      };

      let epoch = epoch_helper::get_current_epoch();

      // add friend to grantor given vouches
      add_given_vouches(grantor_acc, friend_account, epoch);

      // add grantor to friend received vouches
      add_received_vouches(friend_account, grantor_acc, epoch);

      // Update lifetime tracking
      let lifetime = borrow_global_mut<VouchesLifetime>(grantor_acc);
      lifetime.given = lifetime.given + 1;
    }

    // Function to add given vouches
    fun add_given_vouches(grantor_acc: address, vouched_account: address, epoch: u64) acquires GivenVouches, VouchesLifetime {
      let given_vouches = borrow_global_mut<GivenVouches>(grantor_acc);

      let (found, i) = vector::index_of(&given_vouches.outgoing_vouches, &vouched_account);

      // don't check max vouches if we are just extending the expiration
      if (found) {
        // Update the epoch if the vouched account is already present
        let epoch_value = vector::borrow_mut(&mut given_vouches.epoch_vouched, i);
        *epoch_value = epoch;
      } else {

        // Add new vouched account and epoch if not already present
        vector::push_back(&mut given_vouches.outgoing_vouches, vouched_account);
        vector::push_back(&mut given_vouches.epoch_vouched, epoch);
          // Update lifetime tracking
          let lifetime = borrow_global_mut<VouchesLifetime>(grantor_acc);
          lifetime.given = lifetime.given + 1;
          if (epoch > lifetime.last_given_epoch) {
            lifetime.given_this_epoch = 1;
            lifetime.last_given_epoch = epoch;
          } else {
            lifetime.given_this_epoch = lifetime.given_this_epoch + 1;
          };
      }
    }

    // Function to add received vouches
    fun add_received_vouches(vouched_account: address, grantor_acc: address, epoch: u64) acquires ReceivedVouches, VouchesLifetime {
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

        // Update lifetime tracking
        let lifetime = borrow_global_mut<VouchesLifetime>(vouched_account);
        lifetime.received = lifetime.received + 1;
      }
    }

    /// will only successfully vouch if the two are not related by ancestry
    /// prevents spending a vouch that would not be counted.
    /// to add a vouch and ignore this check use insist_vouch
    public(friend) fun vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
      vouch_impl(grantor, friend_account, true);
    }

    public(friend) fun revoke(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
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

      // Update lifetime tracking for revocation
      let current_epoch = epoch_helper::get_current_epoch();
      let lifetime = borrow_global_mut<VouchesLifetime>(grantor_acc);
      lifetime.given_revoked = lifetime.given_revoked + 1;

      // lazy update counter
      if (current_epoch > lifetime.last_revocation_epoch) {
        // If this is a new epoch vs the last revoke, reset the per-epoch count
        lifetime.last_revocation_epoch = current_epoch;
        lifetime.revocations_this_epoch = 1;
      } else {
        // Otherwise, increment the revoke count for the current epoch
        lifetime.revocations_this_epoch = lifetime.revocations_this_epoch + 1;
      }

    }

    public(friend) fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires ReceivedVouches, GivenVouches {
      system_addresses::assert_ol(vm);
      bulk_set(val, buddy_list, buddy_list);
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
    fun bulk_set(val: address, received_list: vector<address>, given_list: vector<address>) acquires ReceivedVouches, GivenVouches {
      // Handle received vouches
      if (exists<ReceivedVouches>(val)) {
        // take self out of list
        let received_buddies = received_list;
        let (is_found, i) = vector::index_of(&received_buddies, &val);
        if (is_found) {
          vector::swap_remove<address>(&mut received_buddies, i);
        };

        let v = borrow_global_mut<ReceivedVouches>(val);
        v.incoming_vouches = received_buddies;

        let epoch_data: vector<u64> = vector::map_ref(&received_buddies, |_e| { 0u64 });
        v.epoch_vouched = epoch_data;
      };

      // Handle given vouches
      if (exists<GivenVouches>(val)) {
        // No need to check for self in given_list as you can't vouch for yourself
        let v = borrow_global_mut<GivenVouches>(val);
        v.outgoing_vouches = given_list;

        let epoch_data: vector<u64> = vector::map_ref(&given_list, |_e| { 0u64 });
        v.epoch_vouched = epoch_data;
      };
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
    /// TODO: deprecate this, left for compatibility
    /// gets the received vouches not expired
    public fun all_not_expired(addr: address): vector<address> acquires ReceivedVouches {
      all_vouchers(addr)
    }


    #[view]
    /// TODO: deprecate this, left for compatibility
    public fun get_received_vouches_not_expired(addr: address): vector<address> acquires ReceivedVouches {
      all_vouchers(addr)
    }

    #[view]
    /// TODO: deprecate this, left for compatibility
    /// gets the given vouches not expired
    public fun get_given_vouches_not_expired(addr: address): vector<address> acquires GivenVouches {
      // get the given vouches
      let (outgoing_vouches, _) = get_given_vouches(addr);
      outgoing_vouches
    }

    #[view]
    /// get count of given vouches this epoch
    public fun get_given_this_epoch(addr: address): u64 acquires VouchesLifetime {
      let state = borrow_global<VouchesLifetime>(addr);
      // if the last given epoch is in the past, reset the count
      if (state.last_given_epoch < epoch_helper::get_current_epoch()) {
        return 0
      };
      // baby reindeer:
      //  sorry, will get on it,
      //  just applying for explosives licence
      //  (sic)

      state.given_this_epoch
    }


    #[view]
    /// show the received vouches but filter expired vouches, and do ancestry check
    public fun true_friends(addr: address): vector<address> acquires ReceivedVouches {
        if (!exists<ReceivedVouches>(addr)) return vector::empty<address>();
        // Get non-expired vouches
        let not_expired = all_not_expired(addr);
        // Filter for ancestry relationships
        ancestry::list_unrelated(not_expired)
    }

    #[view]
    /// check if the user is in fact a valid voucher
    public fun is_valid_voucher_for(voucher: address, recipient: address):bool
    acquires ReceivedVouches {
      let list = true_friends(recipient);
      vector::contains(&list, &voucher)
    }

    #[view]
    /// get last revocation epoch
    public fun get_last_revocation_epoch(addr: address): u64 acquires VouchesLifetime {
      let state = borrow_global<VouchesLifetime>(addr);
      state.last_revocation_epoch
    }

    #[view]
    /// get last revocation epoch
    public fun get_revocations_this_epoch(addr: address): u64 acquires VouchesLifetime {
      let state = borrow_global<VouchesLifetime>(addr);
      state.revocations_this_epoch
    }

    //////// TESTS HELPERS ////////

    #[test_only]
    public fun test_set_received_list(val: address, buddy_list: vector<address>) acquires ReceivedVouches, GivenVouches {
      bulk_set(val, buddy_list, vector::empty<address>());
    }


    #[test_only]
    public fun test_set_given_list(val: address, vouch_list: vector<address>) acquires ReceivedVouches, GivenVouches {
      bulk_set(val, vector::empty<address>(), vouch_list);
    }

    #[test_only]
    public fun test_set_both_lists(val: address, received_list: vector<address>, given_list: vector<address>) acquires ReceivedVouches, GivenVouches {
      bulk_set(val, received_list, given_list);
    }


  }
