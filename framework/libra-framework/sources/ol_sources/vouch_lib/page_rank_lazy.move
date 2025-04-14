module ol_framework::page_rank_lazy {
    use std::signer;
    use std::timestamp;
    use std::vector;
    use ol_framework::vouch;
    use ol_framework::root_of_trust;

    // Constants
    const DEFAULT_WALK_DEPTH: u64 = 4;
    const DEFAULT_NUM_WALKS: u64 = 10;
    const SCORE_TTL_SECONDS: u64 = 1000; // Score validity period in seconds
    const MAX_PROCESSED_ADDRESSES: u64 = 1000; // Circuit breaker to prevent stack overflow
    const DEFAULT_ROOT_REGISTRY: address = @0x1; // Default registry address for root of trust

    // Algorithm selection constants
    const ALGORITHM_MONTE_CARLO: u8 = 0;
    const ALGORITHM_FULL_GRAPH: u8 = 1;
    const DEFAULT_ALGORITHM: u8 = 0;

    // Full graph walk constants
    const FULL_WALK_MAX_DEPTH: u64 = 6; // Maximum path length for full graph traversal

    // Error codes
    const ENODE_NOT_FOUND: u64 = 2;
    const ENOT_INITIALIZED: u64 = 4;
    const EPROCESSING_LIMIT_REACHED: u64 = 6;

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

    // Mark a user's trust record as stale when their vouching relationships change
    public fun mark_record_stale(user: address) acquires UserTrustRecord {
        if (exists<UserTrustRecord>(user)) {
            let user_record = borrow_global_mut<UserTrustRecord>(user);
            user_record.is_stale = true;
        };
    }

    // Calculate or retrieve cached trust score
    // TODO: remove this
    public fun get_trust_score(addr: address): u64 acquires UserTrustRecord {
        get_trust_score_with_algorithm(addr)
    }

    // Calculate or retrieve cached trust score with specific algorithm
    public fun get_trust_score_with_algorithm(addr: address): u64 acquires UserTrustRecord {
        let current_timestamp = timestamp::now_seconds();

        // If user has no trust record, they have no score
        if (!exists<UserTrustRecord>(addr)) {
            return 0
        };

        let user_record = borrow_global<UserTrustRecord>(addr);

        // Check if cached score is still valid and not stale
        if (!user_record.is_stale
            && current_timestamp < user_record.score_computed_at_timestamp + SCORE_TTL_SECONDS
            && user_record.score_computed_at_timestamp > 0) {
            // Cache is fresh, return it
            return user_record.cached_score
        };

        // Cache is stale or expired - compute fresh score
        // Default roots to system account if no registry
        let roots = root_of_trust::get_current_roots_at_registry(@diem_framework);

        // Compute score using selected algorithm
        let score = compute_trust_score(&roots, addr, 0);

        // Update the cache
        let user_record_mut = borrow_global_mut<UserTrustRecord>(addr);
        user_record_mut.cached_score = score;
        user_record_mut.score_computed_at_timestamp = current_timestamp;
        user_record_mut.is_stale = false;

        score
    }

    // Unified function to compute trust score with selected algorithm
    fun compute_trust_score(roots: &vector<address>, target: address, _algorithm: u8): u64 {
        // Ignore algorithm type for now - always use exhaustive walk
        traverse_graph(roots, target, FULL_WALK_MAX_DEPTH)
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

            // Case 1: Direct match - target is a root
            if (root == target) {
                total_score = total_score + 100; // Full score for being a root
            } else {
                // Case 2: Not a direct match - start an exhaustive search from this root
                let visited = vector::empty<address>();
                vector::push_back(&mut visited, root);

                // Initial trust power is 100 (full trust from root)
                total_score = total_score + walk_from_node(
                    root, target, &mut visited, 1, max_depth, 100
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
        // Stop conditions
        if (current_depth >= max_depth || current_power == 0) {
            return 0
        };

        // Target found - return current trust power
        if (current == target) {
            return current_power
        };

        // Get all neighbors this node vouches for
        if (!vouch::is_init(current)) {
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

        // Check ALL neighbors for paths to target
        let i = 0;
        while (i < neighbor_count) {
            let neighbor = *vector::borrow(&neighbors, i);

            // Only visit if not already in path (avoid cycles)
            if (!vector::contains(visited, &neighbor)) {
                // Mark as visited
                vector::push_back(visited, neighbor);

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

                // Remove from visited for backtracking
                let last_idx = vector::length(visited) - 1;
                vector::remove(visited, last_idx);
            };

            i = i + 1;
        };

        total_score
    }

    // Get a random unvisited neighbor that this user vouches for
    // Now uses vouch module instead of local storage
    fun get_random_unvisited_neighbor(user: address, visited: &vector<address>): (address, bool) {
        // Get the active vouches from the vouch module
        let (active_vouches, _) = vouch::get_given_vouches(user);

        let vouches_len = vector::length(&active_vouches);

        if (vouches_len == 0) {
            return (@0x0, false) // Return dummy address with false flag
        };

        // Try to find an unvisited neighbor
        let i = 0;
        while (i < vouches_len) {
            let neighbor = *vector::borrow(&active_vouches, i);

            // If this neighbor hasn't been visited yet, return it
            if (!vector::contains(visited, &neighbor)) {
                return (neighbor, true)
            };

            i = i + 1;
        };

        // No unvisited neighbors found
        (@0x0, false)
    }

    // Mark a user's trust score as stale
    fun mark_as_stale(user: address) acquires UserTrustRecord {
        // Use an internal helper with visited tracking to avoid cycles
        // Initialize a counter to track processed addresses
        let processed_count = 0;
        mark_as_stale_with_visited(user, &vector::empty<address>(), &mut processed_count);
    }

    // Internal helper function with cycle detection for marking nodes as stale
    // Uses vouch module to get outgoing vouches
    fun mark_as_stale_with_visited(
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
            // Pass the updated visited list to avoid cycles
            mark_as_stale_with_visited(*vector::borrow(&outgoing_vouches, i), &new_visited, processed_count);

            // If we've hit the circuit breaker, stop processing
            if (*processed_count >= MAX_PROCESSED_ADDRESSES) {
                break
            };

            i = i + 1;
        };
    }

    // For testing only - initialize a user trust record for testing
    #[test_only]
    public fun initialize_user_trust_record(account: &signer) {
        let addr = signer::address_of(account);

        if (!exists<UserTrustRecord>(addr)) {
            move_to(account, UserTrustRecord {
                cached_score: 0,
                score_computed_at_timestamp: 0,
                is_stale: true,
            });
        };
    }

    // Check if a trust record exists
    public fun has_trust_record(addr: address): bool {
        exists<UserTrustRecord>(addr)
    }

    // Check if a trust record is fresh (not stale and not expired)
    public fun is_fresh_record(addr: address, current_timestamp: u64): bool acquires UserTrustRecord {
        if (!exists<UserTrustRecord>(addr)) {
            return false
        };

        let user_record = borrow_global<UserTrustRecord>(addr);

        !user_record.is_stale
            && current_timestamp < user_record.score_computed_at_timestamp + SCORE_TTL_SECONDS
            && user_record.score_computed_at_timestamp > 0
    }

    // Registry existence check helper for other modules
    public fun registry_exists(_registry_addr: address): bool {
        // Simplified implementation
        true
    }

    // Helper for other modules to check if an address is a root node
    public fun is_root_node(addr: address): bool {
        // Simplified implementation - only system account is root
        addr == @0x1
    }

    // Accessor functions for use by other modules - now using vouch module
    public fun vouches_for(voucher_addr: address, target_addr: address): bool {
        vouch::is_valid_voucher_for(voucher_addr, target_addr)
    }

    // Testing helpers
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
        initialize_user_trust_record(root);
        initialize_user_trust_record(user1);
        initialize_user_trust_record(user2);
        initialize_user_trust_record(user3);

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

    // #[test(framework = @0x1)]
    // fun test_connection_scoring(framework: &signer) {
    //     // Setup test environment and accounts with our family tree:
    //     // ALICE_AT_GENESIS -> BOB_ALICES_CHILD -> CAROL_BOBS_CHILD
    //     // DAVE_AT_GENESIS -> EVE_DAVES_CHILD
    //     setup_test_ancestry(framework);

    //     // Initialize framework root of trust with Alice and Dave as roots
    //     let initial_roots = vector::empty();
    //     vector::push_back(&mut initial_roots, ALICE_AT_GENESIS);
    //     vector::push_back(&mut initial_roots, DAVE_AT_GENESIS);

    //     root_of_trust::framework_migration(
    //         framework,
    //         initial_roots,
    //         2,  // minimum_cohort size
    //         5,  // days until next rotation
    //     );

    //     // Test scoring for direct children of roots
    //     let bob_score = vouch_metrics::calculate_total_vouch_quality(BOB_ALICES_CHILD);

    //     let maybe_degree = ancestry::get_degree(ALICE_AT_GENESIS, BOB_ALICES_CHILD);
    //     let bob_d = *option::borrow(&maybe_degree);
    //     assert!(bob_d == 1, 7357001);
    //     // Direct descendants of roots get 50 points (100/2 - one hop away)
    //     assert!(bob_score == 100, 7357002);

    //     let maybe_degree = ancestry::get_degree(DAVE_AT_GENESIS, EVE_DAVES_CHILD);
    //     let eve_d = *option::borrow(&maybe_degree);
    //     assert!(eve_d == 1, 7357001);

    //     let eve_score = vouch_metrics::calculate_total_vouch_quality( EVE_DAVES_CHILD);
    //     assert!(eve_score == 100, 7357003);

    //     let maybe_degree = ancestry::get_degree(ALICE_AT_GENESIS, CAROL_BOBS_CHILD);
    //     let carol_d = *option::borrow(&maybe_degree);
    //     assert!(carol_d == 2, 3);
    //     // Test scoring for grandchild (two degrees of separation)
    //     let carol_score = vouch_metrics::calculate_total_vouch_quality( CAROL_BOBS_CHILD);

    //     diem_std::debug::print(&carol_score);
    //     // Grandchild gets 50 points (100/2, two degrees away)
    //     assert!(carol_score == 50, 3);

    //     // Test scoring for root members themselves
    //     let alice_score = vouch_metrics::calculate_total_vouch_quality(ALICE_AT_GENESIS);
    //     // Root members get 100 points (direct connection)
    //     print(&alice_score);
    //     // assert!(alice_score == 100, 4);
    // }
}
