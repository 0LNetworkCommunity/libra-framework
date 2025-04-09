module ol_framework::vouch {
    use std::error;
    use std::signer;
    use std::vector;
    use std::string;
    use ol_framework::ancestry;
    use ol_framework::epoch_helper;
    use diem_framework::system_addresses;
    use ol_framework::vouch_metrics;

    use diem_std::debug::print;

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

    /// Maximum number of vouches allowed to be given per epoch
    const MAX_VOUCHES_PER_EPOCH: u64 = 1;

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

    // TODO: someday this should be renamed to ReceivedVouches
    struct MyVouches has key, drop {
      my_buddies: vector<address>, // incoming vouches
      epoch_vouched: vector<u64>,
    }

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
      received_revoked: u64,
      last_revocation_epoch: u64,      // Track when last revocation happened
      revocations_this_epoch: u64,     // Count revocations in current epoch
      vouches_this_epoch: u64,         // Count vouches in current epoch
      current_tracked_epoch: u64,      // Current epoch being tracked
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

      if (!exists<VouchesLifetime>(acc)) {
        move_to<VouchesLifetime>(new_account_sig, VouchesLifetime {
          given: 0,
          given_revoked: 0,
          received: 0,
          received_revoked: 0,
          last_revocation_epoch: 0,
          revocations_this_epoch: 0,
          vouches_this_epoch: 0,
          current_tracked_epoch: epoch_helper::get_current_epoch(),
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

    /// REVOCATION CHECKS
    /// within a period a user might try to add and revoke
    /// many users. As such there are some checks to make on
    /// revocation.
    /// 1. Over a lifetime of the account you cannot revoke more
    /// than you have vouched for.
    /// 2. You cannot revoke more times than the current
    /// amount of vouches you currently have received.

    fun assert_revoke_limit(grantor_acc: address) acquires VouchesLifetime {
      let current_epoch = epoch_helper::get_current_epoch();
      let state = borrow_global_mut<VouchesLifetime>(grantor_acc);

      // Reset counter if we're in a new epoch
      if (state.current_tracked_epoch != current_epoch) {
        state.revocations_this_epoch = 0;
        state.current_tracked_epoch = current_epoch;
      };

      // Check if user has exceeded revocations in this epoch
      assert!(
        state.revocations_this_epoch < MAX_REVOCATIONS_PER_EPOCH,
        error::invalid_state(EREVOCATION_LIMIT_REACHED)
      );
    }

    // Check if enough time has passed since last revocation before giving a new vouch
    fun assert_cooldown_period(grantor_acc: address) acquires VouchesLifetime {
      let current_epoch = epoch_helper::get_current_epoch();
      let state = borrow_global<VouchesLifetime>(grantor_acc);

      // Skip check if no revocations have been made yet
      if (state.last_revocation_epoch == 0) {
        return
      };

      assert!(
        current_epoch >= state.last_revocation_epoch + REVOCATION_COOLDOWN_EPOCHS,
        error::invalid_state(ECOOLDOWN_PERIOD_ACTIVE)
      );
    }

    /// GIVEN VOUCHES CHECK
    /// The maximum number of vouches which can be given
    /// is the lowest of three numbers:
    /// a. below the max safety threshold of the system
    /// b. below the count of active vouches received + 1
    /// c. below the max vouches as calculated by the users' vouch score quality
    /// d. no more than 1 vouch per epoch

    fun assert_max_vouches(grantor_acc: address) acquires GivenVouches, ReceivedVouches, VouchesLifetime {
      assert_safety_ceiling_vouches(grantor_acc);
      // assert_max_vouches_by_score(grantor_acc);
      assert_received_limit_vouches(grantor_acc);
      assert_epoch_vouches_limit(grantor_acc);
    }

    fun assert_safety_ceiling_vouches(account: address) acquires GivenVouches {
        // Get the received vouches that aren't expired
        let given_vouches = vector::length(&get_given_vouches_not_expired(account));

        assert!(given_vouches <= BASE_MAX_VOUCHES, error::invalid_state(EMAX_LIMIT_GIVEN_CEILING));
    }

      fun assert_received_limit_vouches(account: address) acquires GivenVouches, ReceivedVouches{
        // Get the received vouches that aren't expired
        let received_vouches = true_friends(account);
        let received_count = vector::length(&received_vouches);
        let given_vouches = vector::length(&get_given_vouches_not_expired(account));
        // Base case: Always allow at least vouches received + 1
        let max_allowed = received_count + 1;
        assert!(given_vouches <= max_allowed, error::invalid_state(EMAX_LIMIT_GIVEN_BY_RECEIPT));
    }
    // Calculate the maximum number of vouches a user should be able to give based on their vouches received
    public fun calculate_score_limit(account: address): u64 acquires ReceivedVouches {

        // Get true friends (not expired, not related by ancestry)
        let true_friends_list = true_friends(account);

        // Calculate the quality from the list of true friends
        let total_quality = vouch_metrics::calculate_quality_from_list(account, &true_friends_list);

        // Advanced case: For accounts with higher quality vouches,
        // we can provide more outgoing vouches
        let max_allowed = 0;

        if (total_quality >= 2 && total_quality < 200) {
            max_allowed = 1;
        } else if (total_quality >= 200 && total_quality < 400) {
            max_allowed = 5;
        } else if (total_quality >= 400) {
            max_allowed = 10;
        };

        max_allowed
    }

    // a user should not be able to give more vouches than their quality score allows
    fun assert_max_vouches_by_score(grantor_acc: address) acquires GivenVouches, ReceivedVouches {
      // check if the grantor has already reached the limit of vouches
      let given_vouches = get_given_vouches_not_expired(grantor_acc);
      let max_allowed = calculate_score_limit(grantor_acc);

      assert!(
        vector::length(&given_vouches) < max_allowed,
        error::invalid_state(EMAX_LIMIT_GIVEN_BY_SCORE)
      );
    }

    // Check if user has already given the maximum number of vouches allowed per epoch
    fun assert_epoch_vouches_limit(grantor_acc: address) acquires VouchesLifetime {
      let current_epoch = epoch_helper::get_current_epoch();
      let state = borrow_global_mut<VouchesLifetime>(grantor_acc);

      // Reset counter if we're in a new epoch
      if (state.current_tracked_epoch != current_epoch) {
        state.vouches_this_epoch = 0;
        state.current_tracked_epoch = current_epoch;
      };

      // Check if user has exceeded vouches in this epoch
      assert!(
        state.vouches_this_epoch < MAX_VOUCHES_PER_EPOCH,
        error::invalid_state(EMAX_VOUCHES_PER_EPOCH)
      );
    }

    // lazily remove expired vouches prior to setting any new ones
    // ... by popular request
    public (friend) fun garbage_collect_expired(user: address) acquires GivenVouches, ReceivedVouches {
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

    fun vouch_impl(grantor: &signer, friend_account: address, check_unrelated: bool) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
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
    public fun add_given_vouches(grantor_acc: address, vouched_account: address, epoch: u64) acquires GivenVouches, VouchesLifetime, ReceivedVouches {
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
        // abort if we pass the limits of vouching and revoking
        // Check if cooldown period has passed since last revocation
        assert_cooldown_period(grantor_acc);
        // are we hitting the limit of max vouches
        assert_max_vouches(grantor_acc);

        // Update epoch tracking for vouching
        let lifetime = borrow_global_mut<VouchesLifetime>(grantor_acc);
        lifetime.vouches_this_epoch = lifetime.vouches_this_epoch + 1;
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
    public(friend) fun vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
      vouch_impl(grantor, friend_account, true);
    }

    public(friend) fun revoke(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
      let grantor_acc = signer::address_of(grantor);
      assert!(grantor_acc != friend_account, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      assert!(is_init(grantor_acc), error::invalid_state(EGRANTOR_NOT_INIT));

      // Check revocation limits
      assert_revoke_limit(grantor_acc);

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
      lifetime.last_revocation_epoch = current_epoch;
      lifetime.revocations_this_epoch = lifetime.revocations_this_epoch + 1;
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
    /// show the received vouches but filter expired vouches, and do ancestry check
    public fun true_friends(addr: address): vector<address> acquires ReceivedVouches {
        if (!exists<ReceivedVouches>(addr)) return vector::empty<address>();
        // Get non-expired vouches
        let not_expired = all_not_expired(addr);
        // Filter for ancestry relationships
        vouch_metrics::filter_unrelated(not_expired)
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

    #[view]
    /// Calculate the total vouch quality score for a user
    /// This is used by the founder module to evaluate if an account is verified
    public fun calculate_total_vouch_quality(user: address): u64 acquires ReceivedVouches {
        let valid_vouchers = true_friends(user);
        vouch_metrics::calculate_quality_from_list(user, &valid_vouchers)
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
    public(friend) fun insist_vouch_for(grantor: &signer, friend_account: address) acquires ReceivedVouches, GivenVouches, VouchesLifetime {
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
