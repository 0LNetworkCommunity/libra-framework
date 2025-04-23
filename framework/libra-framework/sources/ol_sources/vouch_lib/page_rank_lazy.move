module ol_framework::page_rank_lazy {
    use std::error;
    use std::signer;
    use std::timestamp;
    use std::vector;
    use ol_framework::vouch;
    use ol_framework::root_of_trust;

    friend ol_framework::vouch_txs;

    //////// ERROR CODES ////////
    /// trust record not initialized
    const ENOT_INITIALIZED: u64 = 2;

    //////// CONSTANTS ////////
    // Max score on a single vouch
    const MAX_VOUCH_SCORE: u64 = 100_000;

    // Circuit breaker to prevent stack overflow
    const MAX_PROCESSED_ADDRESSES: u64 = 1000;

    // Maximum path length for full graph traversal
    const FULL_WALK_MAX_DEPTH: u64 = 6;

    // Per-user trust record - each user stores their own trust data
    struct UserTrustRecord has key, drop {
        // No need to store active_vouches - we'll get this from vouch module
        // Cached trust score
        cached_score: u64,
        // When the score was last computed (timestamp)
        score_computed_at_timestamp: u64,
        // Whether this node's trust data is stale and needs recalculation
        is_stale: bool,
        // Shortest path to root now handled in a separate module
    }

    // Initialize a user trust record if it doesn't exist
    public fun maybe_initialize_trust_record(account: &signer) {
        let addr = signer::address_of(account);
        if (!exists<UserTrustRecord>(addr)) {
            move_to(account, UserTrustRecord {
                cached_score: 0,
                score_computed_at_timestamp: 0,
                is_stale: true,
            });
        };
    }

    // Calculate or retrieve cached trust score
    public fun get_trust_score(addr: address): u64 acquires UserTrustRecord {
        let current_timestamp = timestamp::now_seconds();

        // If user has no trust record, they have no score
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));

        // If we've calculated the score recently
        // and it's not stale, return the cached score
        let user_record = borrow_global<UserTrustRecord>(addr);
        if (!user_record.is_stale) {
            return user_record.cached_score
        };

        // Cache is stale or expired - compute fresh score
        // Default roots to system account if no registry
        let roots = root_of_trust::get_current_roots_at_registry(@diem_framework);

        // Compute score using selected algorithm
        let score = traverse_graph(&roots, addr, FULL_WALK_MAX_DEPTH);

        // Update the cache
        let user_record_mut = borrow_global_mut<UserTrustRecord>(addr);
        user_record_mut.cached_score = score;
        user_record_mut.score_computed_at_timestamp = current_timestamp;
        user_record_mut.is_stale = false;

        score
    }

    // Simplified graph traversal - only uses exhaustive walk
    fun traverse_graph(
        roots: &vector<address>,
        target: address,
        max_depth: u64,
    ): u64 {
        let total_score = 0;
        let root_idx = 0;
        let roots_len = vector::length(roots);

        // For each root, calculate its contribution independently
        while (root_idx < roots_len) {
            let root = *vector::borrow(roots, root_idx);

            let visited = vector::empty<address>();
            vector::push_back(&mut visited, root);

            if (root != target) {
              // NOTE: root, you don't don't give yourself points in the walk

                // Initial trust power is 100 (full trust from root)
                total_score = total_score + walk_from_node(
                    root, target, &mut visited, 1, max_depth, 2 * MAX_VOUCH_SCORE
                );
            };

            root_idx = root_idx + 1;
        };

        total_score
    }

    // Simplified full graph traversal from a single node - returns weighted score
    fun walk_from_node(
        current: address,
        target: address,
        visited: &mut vector<address>,
        current_depth: u64,
        max_depth: u64,
        current_power: u64
    ): u64 {
        if(!vouch::is_init(current)) {
            return 0
        };

        // Great, we found the target!
        // then we get to return the power
        // otherwise it will be zero
        // when we run out of power or depth
        if (current == target) {
            return current_power
        };

        // Stop conditions
        if (current_depth >= max_depth || current_power < 2) {
            return 0
        };

        let (neighbors, _) = vouch::get_given_vouches(current);
        let neighbor_count = vector::length(&neighbors);

        // No neighbors means no path
        if (neighbor_count == 0) {
            return 0
        };

        // Track total score from all paths
        let total_score = 0;

        // Calculate power passed to neighbors (50% decay)
        let next_power = current_power / 2;

        // if the both current and target are a root of trust
        // catch the case
        // and exit early
        if(
          root_of_trust::is_root_at_registry(@diem_framework, current) &&
          root_of_trust::is_root_at_registry(@diem_framework, target)
          ) {
            return next_power
        };

        // Check ALL neighbors for paths to target
        let i = 0;
        while (i < neighbor_count) {
            let neighbor = *vector::borrow(&neighbors, i);


            // Only visit if not already in path (avoid cycles)
            if (!vector::contains(visited, &neighbor)) {
                // Mark as visited
                vector::push_back(visited, neighbor);

                // we don't re-enter the root of
                // trust list, because we don't
                // want to accumulate points from
                // roots vouching for each other.
                if(
                  root_of_trust::is_root_at_registry(@diem_framework, neighbor)
                  ) {
                    continue
                };

                // Continue search from this neighbor with reduced power
                let path_score = walk_from_node(
                    neighbor,
                    target,
                    visited,
                    current_depth + 1,
                    max_depth,
                    next_power
                );

                // Add to total score
                total_score = total_score + path_score;
            };

            i = i + 1;
        };

        total_score
    }

    // Mark a user's trust score as stale
    public(friend) fun mark_as_stale(user: address) acquires UserTrustRecord {
        walk_stale(user, &vector::empty<address>(), &mut 0);
    }
    // Internal helper function with cycle detection for marking nodes as stale
    // Uses vouch module to get outgoing vouches
    fun walk_stale(
        user: address,
        visited: &vector<address>,
        processed_count: &mut u64
    ) acquires UserTrustRecord {

        // Circuit breaker: stop processing if we've hit our limit
        if (*processed_count >= MAX_PROCESSED_ADDRESSES) {
            return
        };

        // Skip if we've already visited this node (cycle detection)
        if (vector::contains(visited, &user)) {
            return
        };

        // Increment the number of addresses we've processed
        *processed_count = *processed_count + 1;

        // Mark this user's record as stale if it exists
        if (exists<UserTrustRecord>(user)) {
            let record = borrow_global_mut<UserTrustRecord>(user);
            record.is_stale = true;
        };

        // Get outgoing vouches from vouch module
        let (outgoing_vouches, _) = vouch::get_given_vouches(user);

        // Create a new visited list that includes the current node
        let new_visited = *visited; // Clone the visited list
        vector::push_back(&mut new_visited, user);

        // Recursively process downstream addresses
        let i = 0;
        let len = vector::length(&outgoing_vouches);
        while (i < len) {
            let each_vouchee = vector::borrow(&outgoing_vouches, i);
            // Mark this user's record as stale if it exists
            if (exists<UserTrustRecord>(*each_vouchee)) {
                let record = borrow_global_mut<UserTrustRecord>(user);
                record.is_stale = true;
            };

            // Pass the updated visited list to avoid cycles
            walk_stale(*each_vouchee, &new_visited, processed_count);

            // If we've hit the circuit breaker, stop processing
            if (*processed_count >= MAX_PROCESSED_ADDRESSES) {
                break
            };

            i = i + 1;
        };
    }

    //////// GETTERS ////////
    #[view]
    /// Get the cached trust score for a user
    public fun get_cached_score(addr: address): u64 acquires UserTrustRecord {
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));
        let record = borrow_global<UserTrustRecord>(addr);
        record.cached_score
    }

    #[view]
    // check if it's stale
    public fun is_stale(addr: address): bool acquires UserTrustRecord {
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));
        let record = borrow_global<UserTrustRecord>(addr);
        record.is_stale
    }

    #[view]
    // get the const for highest vouch score
    public fun get_max_single_score(): u64 {
        MAX_VOUCH_SCORE
    }

    //////// TEST HELPERS ///////

    #[test_only]
    public fun setup_mock_trust_network(
        admin: &signer,
        root: &signer,
        user1: &signer,
        user2: &signer,
        user3: &signer
    ) {

        root_of_trust::framework_migration(admin, vector[signer::address_of(root)], 1, 1000);
        // Initialize trust records for all accounts
        maybe_initialize_trust_record(root);
        maybe_initialize_trust_record(user1);
        maybe_initialize_trust_record(user2);
        maybe_initialize_trust_record(user3);

        // Ensure full initialization of vouch structures for all accounts
        // The init function should create both ReceivedVouches and GivenVouches structures
        vouch::init(root);
        vouch::init(user1);
        vouch::init(user2);
        vouch::init(user3);

        // Verify that all resources are initialized correctly before proceeding
        assert!(vouch::is_init(signer::address_of(root)), 99);
        assert!(vouch::is_init(signer::address_of(user1)), 99);
        assert!(vouch::is_init(signer::address_of(user2)), 99);
        assert!(vouch::is_init(signer::address_of(user3)), 99);

        // Initialize ancestry for test accounts to ensure they're unrelated
        ol_framework::ancestry::test_fork_migrate(
            admin,
            root,
            vector::empty<address>()
        );

        ol_framework::ancestry::test_fork_migrate(
            admin,
            user1,
            vector::empty<address>()
        );

        ol_framework::ancestry::test_fork_migrate(
            admin,
            user2,
            vector::empty<address>()
        );

        ol_framework::ancestry::test_fork_migrate(
            admin,
            user3,
            vector::empty<address>()
        );

        // Get addresses we need
        let root_addr = signer::address_of(root);
        let user1_addr = signer::address_of(user1);
        let user2_addr = signer::address_of(user2);
        let user3_addr = signer::address_of(user3);

        // Use direct setting of vouching relationships instead of vouch_txs
        // This avoids dependencies on other modules for testing

        // 1. Setup ROOT -> USER1 and ROOT -> USER2 vouching relationships
        let root_gives = vector::empty<address>();
        vector::push_back(&mut root_gives, user1_addr);
        vector::push_back(&mut root_gives, user2_addr);

        let user1_receives = vector::empty<address>();
        vector::push_back(&mut user1_receives, root_addr);

        let user2_receives = vector::empty<address>();
        vector::push_back(&mut user2_receives, root_addr);

        vouch::test_set_both_lists(root_addr, vector::empty(), root_gives);
        vouch::test_set_both_lists(user1_addr, user1_receives, vector::empty());
        vouch::test_set_both_lists(user2_addr, user2_receives, vector::empty());

        // 2. Setup USER2 -> USER3 vouching relationship
        let user2_gives = vector::empty<address>();
        vector::push_back(&mut user2_gives, user3_addr);

        let user3_receives = vector::empty<address>();
        vector::push_back(&mut user3_receives, user2_addr);

        vouch::test_set_both_lists(user2_addr, user2_receives, user2_gives);
        vouch::test_set_both_lists(user3_addr, user3_receives, vector::empty());
    }

}
