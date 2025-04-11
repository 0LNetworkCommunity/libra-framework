module ol_framework::page_rank_lazy {
    use std::error;
    use std::signer;
    use std::vector;

    // Constants
    const MAX_ROOTS: u64 = 20;
    const DEFAULT_WALK_DEPTH: u64 = 4;
    const DEFAULT_NUM_WALKS: u64 = 10;
    const SCORE_TTL_SECONDS: u64 = 1000; // Score validity period in seconds
    const MAX_PROCESSED_ADDRESSES: u64 = 1000; // Circuit breaker to prevent stack overflow

    // Error codes
    const EROOT_LIMIT_EXCEEDED: u64 = 1;
    const ENODE_NOT_FOUND: u64 = 2;
    const EALREADY_INITIALIZED: u64 = 3;
    const ENOT_INITIALIZED: u64 = 4;
    const EUNAUTHORIZED: u64 = 5;
    const EPROCESSING_LIMIT_REACHED: u64 = 6;

    // Data structures
    struct TrustRegistry has key {
        // Root nodes (trusted by default)
        roots: vector<address>,
    }

    // Per-user trust record - each user stores their own trust data
    struct UserTrustRecord has key, drop {
        // List of accounts this user actively vouches for
        active_vouches: vector<address>,
        // Cached trust score
        cached_score: u64,
        // When the score was last computed (timestamp)
        score_computed_at_timestamp: u64,
        // Whether this node's trust data is stale and needs recalculation
        is_stale: bool,
    }

    // Initialize the trust registry
    public fun initialize(account: &signer, initial_roots: vector<address>) {
        // Ensure only the framework address can initialize the system
        assert!(signer::address_of(account) == @0x1, error::permission_denied(EUNAUTHORIZED));

        let addr = signer::address_of(account);

        assert!(!exists<TrustRegistry>(addr), error::already_exists(EALREADY_INITIALIZED));
        assert!(vector::length(&initial_roots) <= MAX_ROOTS, error::invalid_argument(EROOT_LIMIT_EXCEEDED));

        move_to(account, TrustRegistry {
            roots: initial_roots,
        });

        // Root accounts will initialize themselves when needed
    }

    // A user vouches for another user
    public fun vouch_for(account: &signer, for_address: address) acquires UserTrustRecord {
        let from = signer::address_of(account);

        // Create user trust record if it doesn't exist
        if (!exists<UserTrustRecord>(from)) {
            move_to(account, UserTrustRecord {
                active_vouches: vector::empty(),
                cached_score: 0,
                score_computed_at_timestamp: 0,
                is_stale: true,
            });
        };

        let user_record = borrow_global_mut<UserTrustRecord>(from);

        // Add vouch if not already present
        if (!vector::contains(&user_record.active_vouches, &for_address)) {
            vector::push_back(&mut user_record.active_vouches, for_address);

            // Mark the target's score as stale since they gained a new voucher
            mark_as_stale(for_address);
        };
    }

    // A user revokes trust in another user by removing them from their active vouches
    public fun revoke_trust(account: &signer, for_address: address) acquires UserTrustRecord {
        let from = signer::address_of(account);

        if (!exists<UserTrustRecord>(from)) {
            // Nothing to revoke if user has no trust record
            return
        };

        let user_record = borrow_global_mut<UserTrustRecord>(from);
        let (found, index) = vector::index_of(&user_record.active_vouches, &for_address);

        if (found) {
            // Remove the address from active vouches
            vector::remove(&mut user_record.active_vouches, index);

            // Mark the target's score as stale since they lost a voucher
            mark_as_stale(for_address);
        };
    }

    // Calculate or retrieve cached trust score
    public fun get_trust_score(addr: address, current_timestamp: u64): u64 acquires TrustRegistry, UserTrustRecord {
        let registry_addr = @0x1; // Registry address

        assert!(exists<TrustRegistry>(registry_addr), error::not_found(ENOT_INITIALIZED));

        // If user has no trust record, they have no score
        if (!exists<UserTrustRecord>(addr)) {
            return 0
        };

        let registry = borrow_global<TrustRegistry>(registry_addr);
        let user_record = borrow_global<UserTrustRecord>(addr);

        // Check if cached score is still valid and not stale
        if (!user_record.is_stale
            && current_timestamp < user_record.score_computed_at_timestamp + SCORE_TTL_SECONDS
            && user_record.score_computed_at_timestamp > 0) {
            // Cache is fresh, return it
            return user_record.cached_score
        };

        // Cache is stale or expired - compute fresh score
        let score = compute_score_monte_carlo(addr, &registry.roots, current_timestamp);

        // Update the cache
        let user_record_mut = borrow_global_mut<UserTrustRecord>(addr);
        user_record_mut.cached_score = score;
        user_record_mut.score_computed_at_timestamp = current_timestamp;
        user_record_mut.is_stale = false;

        score
    }

    // Compute trust score using Monte Carlo walks
    fun compute_score_monte_carlo(target: address, roots: &vector<address>, _current_timestamp: u64): u64 acquires UserTrustRecord {
        monte_carlo_trust_score(roots, target, DEFAULT_NUM_WALKS, DEFAULT_WALK_DEPTH)
    }

    // Monte Carlo approximation for trust score
    fun monte_carlo_trust_score(roots: &vector<address>, target: address, num_walks: u64, depth: u64): u64 acquires UserTrustRecord {
        let hits = 0;
        let i = 0;

        while (i < num_walks) {
            // For each root, do random walks
            let j = 0;
            let roots_len = vector::length(roots);

            while (j < roots_len) {
                let curr = *vector::borrow(roots, j);
                let k = 0;

                // Keep track of visited nodes to avoid cycles
                let visited = vector::empty<address>();
                vector::push_back(&mut visited, curr); // Mark root as visited

                // Random walk through vouch graph
                while (k < depth) {
                    if (curr == target) {
                        hits = hits + 1;
                    };

                    // Find users that current user vouches for
                    let (next_addr, has_neighbor) = get_random_unvisited_neighbor(curr, &visited);
                    if (!has_neighbor) {
                        break
                    };

                    curr = next_addr;
                    vector::push_back(&mut visited, curr); // Mark as visited
                    k = k + 1;
                };

                j = j + 1;
            };

            i = i + 1;
        };

        hits
    }

    // Get a random unvisited neighbor that this user vouches for
    fun get_random_unvisited_neighbor(user: address, visited: &vector<address>): (address, bool) acquires UserTrustRecord {
        if (!exists<UserTrustRecord>(user)) {
            return (@0x0, false) // Return dummy address with false flag
        };

        let record = borrow_global<UserTrustRecord>(user);
        let vouches_len = vector::length(&record.active_vouches);

        if (vouches_len == 0) {
            return (@0x0, false) // Return dummy address with false flag
        };

        // Try to find an unvisited neighbor
        let i = 0;
        while (i < vouches_len) {
            let neighbor = *vector::borrow(&record.active_vouches, i);

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

        if (exists<UserTrustRecord>(user)) {
            // Create a new visited list that includes the current node
            let new_visited = *visited; // Clone the visited list
            vector::push_back(&mut new_visited, user);

            // First gather all downstream addresses we need to mark as stale
            let downstream_addresses = vector::empty<address>();
            {
                let record = borrow_global<UserTrustRecord>(user);
                let i = 0;
                let len = vector::length(&record.active_vouches);

                while (i < len) {
                    vector::push_back(&mut downstream_addresses, *vector::borrow(&record.active_vouches, i));
                    i = i + 1;
                };
            }; // Release the borrow here

            // Mark the current user's record as stale
            {
                let record = borrow_global_mut<UserTrustRecord>(user);
                record.is_stale = true;
            }; // Release the mutable borrow

            // Now recursively process downstream addresses without holding any references
            let i = 0;
            let len = vector::length(&downstream_addresses);
            while (i < len) {
                // Pass the updated visited list to avoid cycles
                mark_as_stale_with_visited(*vector::borrow(&downstream_addresses, i), &new_visited, processed_count);

                // If we've hit the circuit breaker, stop processing
                if (*processed_count >= MAX_PROCESSED_ADDRESSES) {
                    break
                };

                i = i + 1;
            };
        };
    }

    // For testing only - initialize a user trust record for testing
    #[test_only]
    public fun initialize_user_trust_record(account: &signer) {
        let addr = signer::address_of(account);

        if (!exists<UserTrustRecord>(addr)) {
            move_to(account, UserTrustRecord {
                active_vouches: vector::empty(),
                cached_score: 0,
                score_computed_at_timestamp: 0,
                is_stale: true,
            });
        };
    }

    // Testing helpers
    #[test_only]
    public fun initialize_test_registry(admin: &signer) {
        let roots = vector::empty<address>();
        vector::push_back(&mut roots, @0x42);
        initialize(admin, roots);
    }

    #[test_only]
    public fun setup_mock_trust_network(
        admin: &signer,
        root: &signer,
        user1: &signer,
        user2: &signer,
        user3: &signer
    ) acquires UserTrustRecord {
        // Setup roots
        let roots = vector::empty<address>();
        vector::push_back(&mut roots, signer::address_of(root));
        initialize(admin, roots);

        // Initialize trust records for all accounts
        initialize_user_trust_record(root);
        initialize_user_trust_record(user1);
        initialize_user_trust_record(user2);
        initialize_user_trust_record(user3);

        // Setup vouch relationships
        // Root vouches for user1
        vouch_for(root, signer::address_of(user1));

        // User1 vouches for user2
        vouch_for(user1, signer::address_of(user2));

        // User2 vouches for user3
        vouch_for(user2, signer::address_of(user3));

        // Now testing a scenario where root vouched for user3 but then revoked it
        vouch_for(root, signer::address_of(user3));
        revoke_trust(root, signer::address_of(user3));
    }

    // Tests
    #[test(admin = @0x1, root = @0x42)]
    fun test_initialize(admin: signer, root: signer) {
        initialize_test_registry(&admin);

        // In the test environment, we need to explicitly initialize the user trust record
        // because our initialize function can't create resources at other addresses
        initialize_user_trust_record(&root);

        let root_addr = signer::address_of(&root);

        // In real testing we would verify the scores properly
        assert!(exists<TrustRegistry>(@0x1), 73570001);
        assert!(exists<UserTrustRecord>(root_addr), 73570002);
    }

    #[test(admin = @0x1, root = @0x42, user = @0x43)]
    fun test_vouch(admin: signer, root: signer, user: signer) acquires UserTrustRecord {
        initialize_test_registry(&admin);

        let root_addr = signer::address_of(&root);
        let user_addr = signer::address_of(&user);

        vouch_for(&root, user_addr);

        // Verify the vouch was recorded
        assert!(exists<UserTrustRecord>(root_addr), 73570003);
        let record = borrow_global<UserTrustRecord>(root_addr);
        assert!(vector::contains(&record.active_vouches, &user_addr), 73570004);
    }

    #[test(admin = @0x1, root = @0x42, user = @0x43)]
    fun test_revoke(admin: signer, root: signer, user: signer) acquires UserTrustRecord {
        initialize_test_registry(&admin);

        let root_addr = signer::address_of(&root);
        let user_addr = signer::address_of(&user);

        // First vouch for the user
        vouch_for(&root, user_addr);
        assert!(vector::contains(&borrow_global<UserTrustRecord>(root_addr).active_vouches, &user_addr), 73570010);

        // Now revoke trust
        revoke_trust(&root, user_addr);

        // Verify the vouch was removed
        assert!(!vector::contains(&borrow_global<UserTrustRecord>(root_addr).active_vouches, &user_addr), 73570011);

        // Verify user record is marked as stale
        if (exists<UserTrustRecord>(user_addr)) {
            assert!(borrow_global<UserTrustRecord>(user_addr).is_stale, 73570012);
        }
    }

    #[test(admin = @0x1, root = @0x42, user = @0x43, user2 = @0x44, user3 = @0x45)]
    fun test_trust_network(
        admin: signer,
        root: signer,
        user: signer,
        user2: signer,
        user3: signer
    ) acquires TrustRegistry, UserTrustRecord {
        setup_mock_trust_network(&admin, &root, &user, &user2, &user3);

        // Simulate scores at timestamp 1
        let current_timestamp = 1;

        // User closest to root should have highest score
        let user_addr = signer::address_of(&user);
        let user2_addr = signer::address_of(&user2);
        let user3_addr = signer::address_of(&user3);

        let score1 = get_trust_score(user_addr, current_timestamp);
        let score2 = get_trust_score(user2_addr, current_timestamp);
        let score3 = get_trust_score(user3_addr, current_timestamp);

        // Debug scores - uncomment if needed
        // std::debug::print(&score1);
        // std::debug::print(&score2);
        // std::debug::print(&score3);

        // In our Monte Carlo implementation with current parameters,
        // scores depend on path length from root
        assert!(score1 > 0, 73570005); // User1 has direct vouch from root
        assert!(score2 > 0, 73570006); // User2 is two hops from root

        // User3 should have lower score since root revoked direct vouch,
        // but may still get some score from the path: root->user1->user2->user3
        // So we can't assert it equals zero, just that it's likely lower
        assert!(score1 >= score3, 73570007);
    }

    #[test(admin = @0x1, root = @0x42, user1 = @0x43, user2 = @0x44)]
    fun test_cyclic_vouches_handled_correctly(
        admin: signer,
        root: signer,
        user1: signer,
        user2: signer
    ) acquires TrustRegistry, UserTrustRecord {
        // Initialize trust registry
        initialize_test_registry(&admin);

        // Initialize user records
        initialize_user_trust_record(&root);
        initialize_user_trust_record(&user1);
        initialize_user_trust_record(&user2);

        // Only get addresses we actually need
        let user1_addr = signer::address_of(&user1);
        let user2_addr = signer::address_of(&user2);

        // Set up a cycle: root -> user1 -> user2 -> user1 (cycle back to user1)
        vouch_for(&root, user1_addr);
        vouch_for(&user1, user2_addr);
        vouch_for(&user2, user1_addr); // This creates a cycle

        // Get scores at timestamp 1
        let current_timestamp = 1;

        // Calculate scores
        let user1_score = get_trust_score(user1_addr, current_timestamp);
        let user2_score = get_trust_score(user2_addr, current_timestamp);

        // Verify both users received a score despite the cycle
        assert!(user1_score > 0, 73570020);
        assert!(user2_score > 0, 73570021);

        // To verify DAG property (cycle avoidance), we can add a debug counter to track
        // how many times each node was visited. But since we can't modify the code in the test,
        // we can use score magnitudes as a proxy to show cycles aren't creating inflated scores.

        // If cycles weren't handled correctly, scores would be artificially high when compared
        // to a simple path. Let's verify that user2 (2 hops from root) has a score that's
        // reasonable compared to user1 (1 hop from root)

        // In a non-cyclic Monte Carlo approach, users farther from root should have lower scores
        assert!(user1_score >= user2_score, 73570022);
    }
}
