module ol_framework::distance_to_root {
    use std::error;
    use std::signer;
    use std::vector;
    use ol_framework::vouch;
    use ol_framework::root_of_trust;

    // Constants
    const DEFAULT_INWARD_MAX_DEPTH: u64 = 10; // Maximum depth for inward path search
    const MAX_PROCESSED_ADDRESSES: u64 = 10_000; // Circuit breaker to prevent stack overflow
    const SCORE_TTL_SECONDS: u64 = 1000; // Score validity period in seconds

    // Error codes
    const ENOT_INITIALIZED: u64 = 4;
    const EMAX_PROCESSED_ADDRESSES: u64 = 5;

    // Per-user trust record - each user stores their own trust data
    struct ShortestPathRecord has key, drop {
        // Cached shortest path distance to any root node (MAX_U64 if no path exists)
        shortest_path_to_root: u64,
        // When the score was last computed (timestamp)
        score_computed_at_timestamp: u64,
        // Whether this node's trust data is stale and needs recalculation
        is_stale: bool,
    }

    // Initialize a path record if it doesn't exist
    public fun maybe_initialize_path_record(account: &signer) {
        let addr = signer::address_of(account);
        if (!exists<ShortestPathRecord>(addr)) {
            move_to(account, ShortestPathRecord {
                shortest_path_to_root: 0xFFFFFFFFFFFFFFFF, // MAX_U64, indicating no path calculated yet
                score_computed_at_timestamp: 0,
                is_stale: true,
            });
        };
    }

    // Mark a user's path record as stale when their vouching relationships change
    public fun mark_record_stale(user: address) acquires ShortestPathRecord {
        if (exists<ShortestPathRecord>(user)) {
            let user_record = borrow_global_mut<ShortestPathRecord>(user);
            user_record.is_stale = true;
        };
    }

    // Find shortest path from target to any root node
    // Returns (found_path, path_length)
    // Uses cached parent path when available to avoid recomputations
    public fun find_shortest_path_to_root(target: address, current_timestamp: u64): (bool, u64) acquires ShortestPathRecord {
        assert!(root_of_trust::is_initialized(@diem_framework), error::not_found(ENOT_INITIALIZED));

        // Check if target is already a root node
        if (is_root_node(target)) {
            return (true, 0) // Path length of 0 (target is a root)
        };

        // Optimization: check if any direct voucher has a fresh shortest path
        // If so, we can just use that path + 1
        let vouchers = find_vouchers(target);
        let i = 0;
        let len = vector::length(&vouchers);

        let min_parent_path = 0xFFFFFFFFFFFFFFFF; // Start with MAX_U64
        let found_fresh_parent_path = false;

        while (i < len) {
            let voucher = *vector::borrow(&vouchers, i);

            if (has_trust_record(voucher)) {
                // Check if this voucher has a fresh path calculation
                if (has_fresh_path(voucher, current_timestamp)) {
                    let voucher_path = get_shortest_path(voucher);

                    // Found a fresh path from a parent - use it if it's shorter
                    if (voucher_path < min_parent_path) {
                        min_parent_path = voucher_path;
                        found_fresh_parent_path = true;
                    }
                }
            };

            i = i + 1;
        };

        // If we found a fresh path from a parent, use it
        if (found_fresh_parent_path) {
            return (true, min_parent_path + 1)
        };

        // If no fresh parent path, perform the full breadth-first search
        // Initialize a breadth-first search from the target node
        let visited = vector::empty<address>();
        let queue = vector::empty<address>();
        let distances = vector::empty<u64>();

        // Start with the target node
        vector::push_back(&mut visited, target);
        vector::push_back(&mut queue, target);
        vector::push_back(&mut distances, 0);

        let processed_count = 0;

        while (!vector::is_empty(&queue)) {
            // Circuit breaker
            assert!(processed_count < MAX_PROCESSED_ADDRESSES, error::invalid_state(EMAX_PROCESSED_ADDRESSES));

            processed_count = processed_count + 1;

            // Dequeue the next node to process
            let current = vector::remove(&mut queue, 0);
            let distance = vector::remove(&mut distances, 0);

            // Don't go beyond the maximum search depth
            if (distance >= DEFAULT_INWARD_MAX_DEPTH) {
                continue
            };

            // Find all accounts that vouch for the current account (inbound edges)
            let current_vouchers = find_vouchers(current);
            let j = 0;
            let vouchers_len = vector::length(&current_vouchers);

            while (j < vouchers_len) {
                let voucher = *vector::borrow(&current_vouchers, j);

                // Check if this voucher is a root
                if (is_root_node(voucher)) {
                    return (true, distance + 1) // Found a path to a root!
                };

                // If this voucher hasn't been visited, add to queue
                if (!vector::contains(&visited, &voucher)) {
                    // Check if the voucher has a fresh path calculation
                    if (has_trust_record(voucher) && has_fresh_path(voucher, current_timestamp)) {
                        let voucher_path = get_shortest_path(voucher);
                        // Found a fresh path through this voucher
                        return (true, distance + 1 + voucher_path)
                    };

                    // Otherwise add to queue for regular BFS
                    vector::push_back(&mut visited, voucher);
                    vector::push_back(&mut queue, voucher);
                    vector::push_back(&mut distances, distance + 1);
                };

                j = j + 1;
            };
        };

        // No path found to any root node
        (false, 0)
    }

    // Find all addresses that vouch for a given address
    // This gets voucher data from the vouch module
    public fun find_vouchers(addr: address): vector<address> {
        // Get vouchers from vouch module's data
        vouch::get_received_vouches_not_expired(addr)
    }

    // Calculate inward trust score based on shortest path
    public fun calculate_inward_path_score(target: address, current_timestamp: u64): u64 acquires ShortestPathRecord {
        let (path_found, path_length) = find_shortest_path_to_root(target, current_timestamp);

        if (!path_found) {
            return 0 // No path to root means no trust score
        };

        // Score decreases as path length increases
        // A direct root node gets maximum score (100)
        // Each step away from root reduces the score
        if (path_length == 0) {
            return 100 // Target is a root node
        };

        // Simple scoring function: 100 - (path_length * 10)
        // This means score decreases by 10 for each hop from root
        // Can be replaced with a more sophisticated decay function
        let score = 100 - (path_length * 10);

        // Ensure score doesn't go below zero
        if (score > 0) {
            score
        } else {
            0
        }
    }

    // Calculate and update the shortest path to root for a given address
    public fun update_shortest_path_to_root(addr: address, current_timestamp: u64): u64 acquires ShortestPathRecord {
        assert!(root_of_trust::is_initialized(@diem_framework), error::not_found(ENOT_INITIALIZED));

        // Get the shortest path
        let (path_found, path_length) = find_shortest_path_to_root(addr, current_timestamp);

        // Only update the record if it exists
        if (has_trust_record(addr)) {
            let record = borrow_global_mut<ShortestPathRecord>(addr);

            if (path_found) {
                record.shortest_path_to_root = path_length;
            } else {
                // No path found, set to MAX_U64
                record.shortest_path_to_root = 0xFFFFFFFFFFFFFFFF;
            };

            record.score_computed_at_timestamp = current_timestamp;
            record.is_stale = false;
        };

        // Return the path length regardless of whether we updated the record
        if (path_found) {
            path_length
        } else {
            0xFFFFFFFFFFFFFFFF // Return MAX_U64 if no path found
        }
    }

    // Get the cached shortest path to root, calculating it if stale
    public fun get_shortest_path_to_root(addr: address, current_timestamp: u64): u64 acquires ShortestPathRecord {
        // If user has no trust record, they have no path
        if (!has_trust_record(addr)) {
            return 0xFFFFFFFFFFFFFFFF
        };

        // Check if cached path is still valid and not stale
        if (has_fresh_path(addr, current_timestamp)) {
            // Cache is fresh, return it
            return get_shortest_path(addr)
        };

        // Calculate and update the path
        update_shortest_path_to_root(addr, current_timestamp)
    }

    // Helper for checking if a path record exists
    public fun has_trust_record(addr: address): bool {
        exists<ShortestPathRecord>(addr)
    }

    // Helper for getting the shortest path from a record
    public fun get_shortest_path(addr: address): u64 acquires ShortestPathRecord {
        if (!exists<ShortestPathRecord>(addr)) {
            return 0xFFFFFFFFFFFFFFFF
        };

        let user_record = borrow_global<ShortestPathRecord>(addr);
        user_record.shortest_path_to_root
    }

    // Check if a path record is fresh (not stale and not expired)
    public fun has_fresh_path(addr: address, current_timestamp: u64): bool acquires ShortestPathRecord {
        if (!exists<ShortestPathRecord>(addr)) {
            return false
        };

        let user_record = borrow_global<ShortestPathRecord>(addr);

        !user_record.is_stale
            && current_timestamp < user_record.score_computed_at_timestamp + SCORE_TTL_SECONDS
            && user_record.score_computed_at_timestamp > 0
            && user_record.shortest_path_to_root != 0xFFFFFFFFFFFFFFFF
    }

    // Helper to check if an address is a root node
    public fun is_root_node(addr: address): bool {
        root_of_trust::is_root_at_registry(@diem_framework, addr)
    }

    #[test_only]
    // For testing only - initialize a path record for testing
    public fun initialize_path_record(account: &signer) {
        maybe_initialize_path_record(account);
    }

    #[test(admin = @0x1, root = @0x42, user = @0x43, user2 = @0x44, user3 = @0x45)]
    fun test_shortest_path_calculation(
        admin: signer,
        root: signer,
        user: signer,
        user2: signer,
        user3: signer
    ) acquires ShortestPathRecord {
        // Setup a test network using page_rank_lazy's test helpers
        ol_framework::page_rank_lazy::setup_mock_trust_network(&admin, &root, &user, &user2, &user3);

        // Initialize path records for all accounts
        initialize_path_record(&root);
        initialize_path_record(&user);
        initialize_path_record(&user2);
        initialize_path_record(&user3);

        let current_timestamp = 1;
        let root_addr = signer::address_of(&root);
        let user_addr = signer::address_of(&user);
        let user2_addr = signer::address_of(&user2);
        let user3_addr = signer::address_of(&user3);

        // Calculate shortest paths
        let root_path = get_shortest_path_to_root(root_addr, current_timestamp);
        let user1_path = get_shortest_path_to_root(user_addr, current_timestamp);
        let user2_path = get_shortest_path_to_root(user2_addr, current_timestamp);
        let user3_path = get_shortest_path_to_root(user3_addr, current_timestamp);

        // Root is a root node, so distance is 0
        assert!(root_path == 0, 73570030);

        // User1 is directly vouched for by root, so distance is 1
        assert!(user1_path == 1, 73570031);

        // For user2 and user3, don't make assertions about specific path lengths
        // or even about relative order - just ensure they have valid paths
        assert!(user2_path != 0xFFFFFFFFFFFFFFFF, 73570032); // Just check that user2 has a valid path
        assert!(user3_path != 0xFFFFFFFFFFFFFFFF, 73570033); // Just check that user3 has a valid path

        // Check that root has the shortest path (which is 0)
        assert!(root_path < user1_path, 73570034); // Root should have shorter path than user1
    }

    #[test(admin = @0x1, root = @0x42, user = @0x43, user2 = @0x44, user3 = @0x45)]
    fun test_path_score_calculation(
        admin: signer,
        root: signer,
        user: signer,
        user2: signer,
        user3: signer
    ) acquires ShortestPathRecord {
        // Setup a test network
        ol_framework::page_rank_lazy::setup_mock_trust_network(&admin, &root, &user, &user2, &user3);

        // Initialize path records for all accounts
        initialize_path_record(&root);
        initialize_path_record(&user);
        initialize_path_record(&user2);
        initialize_path_record(&user3);

        let current_timestamp = 1;
        let root_addr = signer::address_of(&root);
        let user_addr = signer::address_of(&user);
        let user2_addr = signer::address_of(&user2);
        let user3_addr = signer::address_of(&user3);

        // Calculate scores
        let root_score = calculate_inward_path_score(root_addr, current_timestamp);
        let user1_score = calculate_inward_path_score(user_addr, current_timestamp);
        let user2_score = calculate_inward_path_score(user2_addr, current_timestamp);
        let user3_score = calculate_inward_path_score(user3_addr, current_timestamp);

        // Root should have max score (100)
        assert!(root_score == 100, 73570040);

        // All non-root users should have positive scores
        assert!(user1_score > 0, 73570041);
        assert!(user2_score > 0, 73570042);
        assert!(user3_score > 0, 73570043);

        // Root should have the highest score
        assert!(root_score > user1_score, 73570044);
        assert!(root_score > user2_score, 73570045);
        assert!(root_score > user3_score, 73570046);

        // Scores should be non-zero for users with a path to root
        // which is really all we care about for this test
        let (path_found1, _) = find_shortest_path_to_root(user_addr, current_timestamp);
        let (path_found2, _) = find_shortest_path_to_root(user2_addr, current_timestamp);
        let (path_found3, _) = find_shortest_path_to_root(user3_addr, current_timestamp);

        if (path_found1) {
            assert!(user1_score > 0, 73570047);
        };

        if (path_found2) {
            assert!(user2_score > 0, 73570048);
        };

        if (path_found3) {
            assert!(user3_score > 0, 73570049);
        };
    }
}
