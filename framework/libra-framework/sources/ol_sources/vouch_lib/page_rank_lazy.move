module ol_framework::page_rank_lazy {
    use std::error;
    use std::signer;
    use std::timestamp;
    use std::vector;
    use ol_framework::vouch;
    use ol_framework::root_of_trust;

    friend ol_framework::vouch_txs;
    friend ol_framework::founder;
    friend ol_framework::vouch_limits;

    #[test_only]
    friend ol_framework::test_page_rank;
    #[test_only]
    friend ol_framework::mock;

    //////// ERROR CODES ////////
    /// Thrown if a trust record is missing for the user.
    const ENOT_INITIALIZED: u64 = 2;

    /// Thrown if the maximum number of nodes to process is exceeded.
    const EMAX_PROCESSED_ADDRESSES: u64 = 3;

    //////// CONSTANTS ////////
    /// Maximum score assignable for a single vouch (upper bound for direct trust).
    const MAX_VOUCH_SCORE: u64 = 100_000;

    /// Maximum number of nodes processed in a single traversal (prevents stack overflow).
    const MAX_PROCESSED_ADDRESSES: u64 = 10_000;

    /// Maximum allowed depth for trust graph traversal.
    const MAX_PATH_DEPTH: u64 = 5;

    /// Stores a user's trust score and its staleness state.
    struct UserTrustRecord has key, drop {
        /// Last computed trust score for this user.
        cached_score: u64,
        /// Timestamp (seconds) when the score was last computed.
        score_computed_at_timestamp: u64,
        /// True if the trust score is outdated and needs recalculation.
        is_stale: bool,
        // Shortest path to root is managed in a separate module.
    }

    /// Initializes a trust record for the account if it does not already exist.
    /// This creates the basic structure needed to track a user's trust score.
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


    /// Returns the cached trust score for an address, or recalculates it if stale.
    /// Uses a reverse PageRank-like algorithm to aggregate trust from roots.
    ///
    /// This function uses an optimized page rank algorithm that:
    /// 1. Finds all possible paths from roots of trust to the target
    /// 2. Accumulates scores from all valid paths, including diamond patterns
    /// 3. Applies trust decay proportional to distance from roots
    ///
    /// The calculation considers the entire trust graph and properly handles:
    /// - Multiple paths to the same target
    /// - Branching and merging paths
    /// - Root-of-trust special cases
    public(friend) fun get_trust_score(addr: address): u64 acquires UserTrustRecord {

        // If user has no trust record, they have no score
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));

        // If we've calculated the score recently
        // and it's not stale, return the cached score
        let user_record = borrow_global<UserTrustRecord>(addr);
        if (!user_record.is_stale) {
            return user_record.cached_score
        };
        set_score(addr)
    }

    /// Recalculates and updates the trust score for an address.
    /// Traverses the trust graph from roots to the target and updates the cache.
    /// This is a costly operation and should be used sparingly.
    fun set_score(addr: address): u64 acquires UserTrustRecord {
        // If user has no trust record, they have no score
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));
        // Cache is stale or expired - compute fresh score
        // Default roots to system account if no registry
        let roots = root_of_trust::get_current_roots_at_registry(@diem_framework);
        // Compute score using selected algorithm
        let score = traverse_graph(&roots, addr);
        // Update the cache
        let user_record_mut = borrow_global_mut<UserTrustRecord>(addr);
        user_record_mut.cached_score = score;
        user_record_mut.score_computed_at_timestamp = timestamp::now_seconds();
        user_record_mut.is_stale = false;

        score
    }

    /// Traverses the trust graph from the target user back to the roots of trust.
    /// Uses received vouches to avoid dead-end paths and reduce processing.
    fun traverse_graph(
        roots: &vector<address>,
        target: address,
    ): u64 {
        let processed_count: u64 = 0;
        let max_depth_reached: u64 = 0;
        let visited = vector::empty<address>();

        // Start the reverse walk from the target and extract just the score
        walk_backwards_from_target_with_stats(
            target, roots, &mut visited, 2 * MAX_VOUCH_SCORE, 0, &mut processed_count, &mut max_depth_reached
        )
    }

    /// Like `traverse_graph`, but also returns statistics: (score, max_depth, processed_count).
    fun traverse_graph_with_stats(
        roots: &vector<address>,
        target: address,
    ): (u64, u64, u64) {
        let processed_count: u64 = 0;
        let max_depth_reached: u64 = 0;
        let visited = vector::empty<address>();

        // Start the reverse walk from the target
        let score = walk_backwards_from_target_with_stats(
            target, roots, &mut visited, 2 * MAX_VOUCH_SCORE, 0, &mut processed_count, &mut max_depth_reached
        );

        (score, max_depth_reached, processed_count)
    }

    /// Walks backwards from the target toward roots of trust, accumulating trust score.
    /// - Starts from the target user.
    /// - Uses received vouches for traversal.
    /// - Accumulates score when reaching a root.
    /// - Avoids cycles and dead ends.
    /// - Applies trust decay and limits neighbor exploration to control complexity.
    fun walk_backwards_from_target_with_stats(
        current: address,
        roots: &vector<address>,
        visited: &mut vector<address>,
        current_power: u64,
        current_depth: u64,
        processed_count: &mut u64,
        max_depth_reached: &mut u64
    ): u64 {
        // Track maximum depth reached
        if (current_depth > *max_depth_reached) {
            *max_depth_reached = current_depth;
        };

        // Early terminations that don't consume processing budget
        if (current_depth >= MAX_PATH_DEPTH) return 0;
        if (vector::contains(visited, &current)) return 0;
        if (!vouch::is_init(current)) return 0;
        if (current_power < 2) return 0;

        // Check if we've reached a root of trust - this is our success condition!
        if (vector::contains(roots, &current) && current_depth > 0) {
            return current_power
        };

        // Budget check and consumption
        if (*processed_count >= MAX_PROCESSED_ADDRESSES) return 0;
        *processed_count = *processed_count + 1;

        // Get who vouched FOR this current user (backwards direction)
        let (received_from, _) = vouch::get_received_vouches(current);
        let neighbor_count = vector::length(&received_from);

        if (neighbor_count == 0) return 0;

        let total_score = 0;
        let next_power = current_power / 2;
        let next_depth = current_depth + 1;

        // Add current to visited and explore received vouches (backwards)
        vector::push_back(visited, current);

        // CRITICAL: Limit number of neighbors explored to prevent exponential explosion
        let max_neighbors_to_explore = if (current_depth < 2) { 10 } else { 5 };
        let neighbors_explored = 0;
        let i = 0;

        while (i < neighbor_count && neighbors_explored < max_neighbors_to_explore) {
            if (*processed_count >= MAX_PROCESSED_ADDRESSES - 5) break;

            let neighbor = *vector::borrow(&received_from, i);
            if (!vector::contains(visited, &neighbor)) {
                // Create a copy of visited for this path
                let visited_copy = *visited;
                let path_score = walk_backwards_from_target_with_stats(
                    neighbor,
                    roots,
                    &mut visited_copy,
                    next_power,
                    next_depth,
                    processed_count,
                    max_depth_reached
                );
                total_score = total_score + path_score;
                neighbors_explored = neighbors_explored + 1;
            };
            i = i + 1;
        };

        total_score
    }

    /// Marks a user's trust score as stale and propagates staleness to downstream accounts.
    /// Uses cycle detection and a processing limit to prevent infinite recursion.
    public(friend) fun mark_as_stale(user: address) acquires UserTrustRecord {
        let visited = vector::empty<address>();
        let processed_count: u64 = 0; // Initialize as a mutable local variable
        walk_stale(user, &mut visited, &mut processed_count); // Pass as a mutable reference
    }

    /// Helper for `mark_as_stale`. Recursively marks downstream nodes as stale.
    /// - Avoids revisiting nodes (cycle detection).
    /// - Stops if the processing limit is reached.
    /// - Only processes nodes initialized in the vouch system.
    fun walk_stale(
        user: address,
        visited: &mut vector<address>,
        processed_count: &mut u64
    ) acquires UserTrustRecord {
        // Skip if we've already visited this node in the current traversal (cycle detection)
        // This also ensures we only count/process each unique node once.
        if (vector::contains(visited, &user)) {
            return
        };

        // Check if the global limit for processed nodes has been reached *before* processing this one.
        // If *processed_count is already at the limit, we can't process another new node.
        assert!(*processed_count < MAX_PROCESSED_ADDRESSES, error::invalid_state(EMAX_PROCESSED_ADDRESSES));

        // This node is new and will be processed. Increment the global count.
        *processed_count = *processed_count + 1;

        // Process the current 'user' node:
        // 1. Mark its UserTrustRecord as stale if it exists.
        if (exists<UserTrustRecord>(user)) {
            let record = borrow_global_mut<UserTrustRecord>(user);
            record.is_stale = true;
        };

        // 2. Add this node to the visited set for the current traversal.
        vector::push_back(visited, user);

        // If the user is not initialized in the vouch system, they cannot have outgoing vouches.
        // Staleness propagation stops here for this path, but 'user' itself has been processed and counted.
        if (!vouch::is_init(user)) {
            return
        };

        // Now walk their outgoing vouches
        let (outgoing_vouches, _) = vouch::get_given_vouches(user);
        if (vector::length(&outgoing_vouches) == 0) {
            return
        };

        // Recursively process downstream addresses
        let i = 0;
        let len = vector::length(&outgoing_vouches);
        while (i < len) {
            // Check again if we've hit the processing limit
            if (*processed_count >= MAX_PROCESSED_ADDRESSES - 5) break;

            let each_vouchee = vector::borrow(&outgoing_vouches, i);
            // Pass the same mutable reference to processed_count.
            // The checks at the beginning of the recursive call (visited and limit)
            // will handle whether to proceed for 'each_vouchee'.
            walk_stale(*each_vouchee, visited, processed_count);
            i = i + 1;
        };
    }

    //////// CACHE ////////

    /// Refreshes the cached trust score for a user by recalculating it.
    /// Only callable by the user.
    public entry fun refresh_cache(user: address) acquires UserTrustRecord{
      // assert initialized
      assert!(exists<UserTrustRecord>(user), error::invalid_state(ENOT_INITIALIZED));
      // get_score
      let _score = set_score(user);
    }

    //////// GETTERS ////////
    #[view]
    /// Returns the cached trust score for a user.
    public fun get_cached_score(addr: address): u64 acquires UserTrustRecord {
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));
        let record = borrow_global<UserTrustRecord>(addr);
        record.cached_score
    }

    #[view]
    /// Calculates a fresh trust score for a user without updating the cache.
    /// Returns (score, max_depth_reached, accounts_processed).
    /// Intended for diagnostics and testing only.
    public fun calculate_score(addr: address): (u64, u64, u64) {
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));
        // Cache is stale or expired - compute fresh score
        // Default roots to system account if no registry
        let roots = root_of_trust::get_current_roots_at_registry(@diem_framework);
        // Compute score using selected algorithm
        let (score, max_depth, processed_count) = traverse_graph_with_stats(&roots, addr);
        (score, max_depth, processed_count)
    }

    #[view]
    /// Returns true if the user's trust score is marked as stale.
    public fun is_stale(addr: address): bool acquires UserTrustRecord {
        assert!(exists<UserTrustRecord>(addr), error::invalid_state(ENOT_INITIALIZED));
        let record = borrow_global<UserTrustRecord>(addr);
        record.is_stale
    }

    #[view]
    /// Returns the maximum possible score for a single vouch.
    public fun get_max_single_score(): u64 {
        MAX_VOUCH_SCORE
    }

    //////// TEST HELPERS ///////

    /// Sets up a mock trust network for testing.
    /// - Initializes trust records and vouch structures for all test accounts.
    /// - Sets up vouching relationships and unrelated ancestry for each account.
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
