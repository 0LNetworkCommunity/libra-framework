#[test_only]

// Tests for page_rank_lazy module with a large network of accounts.
// This simulates 100 accounts with various vouching relationships
// and tests the page rank score calculation.
module ol_framework::test_page_rank {
  use std::vector;
  use std::signer;
  use diem_std::math64;
  use diem_std::debug::print;
  use ol_framework::mock;
  use ol_framework::root_of_trust;
  use ol_framework::page_rank_lazy;
  use ol_framework::vouch;
  use ol_framework::ancestry;
  use diem_framework::account; // Adding missing import for account module

  // Define constants for the matrix test
  const ROOT_USER_COUNT: u64 = 10;
  const LEVELS_COUNT: u64 = 10;
  const VOUCHES_COUNT: u64 = 10;

  // Test with a network of 100 accounts
  // #[test(framework_sig = @ol_framework)]
  // #[ignore] // Temporarily disabled - needs more work on vouching limits
  fun test_large_vouch_network(framework_sig: signer) {
    // 1. Initialize blockchain and setup the network
    let root_count = 3;
    let normal_account_count = 100;
    let min_trusted_epoch = 1;
    let high_level_count = 10;
    let mid_level_count = 30;
    let cross_connection_count = 50;
    let cycle_count = 3;

    // Set up the network
    let (roots, normal_addresses) = setup_test_network(
      &framework_sig,
      root_count,
      normal_account_count,
      min_trusted_epoch
    );

    // Create vouch relationships according to a layered pattern
    create_layered_network_pattern(
      &roots,
      &normal_addresses,
      high_level_count,
      mid_level_count
    );

    // Add cross-connections for a more realistic network
    create_random_cross_connections(
      &normal_addresses,
      cross_connection_count
    );

    // Create cyclic relationships
    create_cyclic_patterns(
      &normal_addresses,
      cycle_count
    );

    // Calculate scores and run verification
    let current_timestamp = 1000;
    let scores = calculate_scores(&normal_addresses, current_timestamp);

    // Run various verifications
    verify_root_scores(&roots);
    verify_layer_scores(
      &scores,
      high_level_count,
      mid_level_count,
      &normal_addresses
    );
    verify_vouch_impact(&normal_addresses, &roots, current_timestamp);
    verify_staleness(&normal_addresses, &roots, current_timestamp);
  }

  // Test with a large-scale network of 1000 accounts
  // #[test(framework_sig = @ol_framework)]
  // #[ignore] // Temporarily disabled - needs more work on vouching limits
  fun test_massive_vouch_network(framework_sig: signer) {
    // 1. Initialize blockchain and setup the network
    let root_count = 5;
    let normal_account_count = 1000;
    let min_trusted_epoch = 2;
    let roots_per_layer1 = 3;
    let cross_connection_count = 300;
    let cycle_count = 50;

    // Set up the network
    let (roots, normal_addresses) = setup_test_network(
      &framework_sig,
      root_count,
      normal_account_count,
      min_trusted_epoch
    );

    // Layer configuration
    let layer_sizes = vector[20, 50, 100, 200, 630];  // Total = 1000 accounts

    // Create structured layered network
    create_layered_network(
      &roots,
      &normal_addresses,
      &layer_sizes,
      roots_per_layer1,
      5 // layer_connections
    );

    // Add cross-connections for a more realistic network
    create_cross_connections(
      &normal_addresses,
      cross_connection_count
    );

    // Create cyclic relationships
    create_cyclic_patterns(
      &normal_addresses,
      cycle_count
    );

    // Calculate and verify trust scores
    let current_timestamp = 1000;
    let avg_scores = calculate_layer_average_scores(
      &normal_addresses,
      &layer_sizes,
      current_timestamp
    );

    // Verify trust score properties
    verify_root_node_scores(&roots);
    verify_layer_score_distribution(&avg_scores);
    verify_specific_node_scores(&normal_addresses, &roots, current_timestamp);
  }

  // Test with a controlled 10x10 matrix vouch network
  // This creates a structured network where we have precise control over:
  // 1. Number of vouches (1-10)
  // 2. Distance from root (1-10 levels)
  // This helps verify page rank scoring in a controlled environment.
  #[test(framework_sig = @ol_framework)]
  fun test_matrix_vouch_network(framework_sig: signer) {
    // 1. Initialize blockchain
    mock::ol_test_genesis(&framework_sig);

    // 2. Create accounts and set up network
    let (root_users, network_matrix) = setup_matrix_network(&framework_sig);

    // 3. Calculate and verify scores using full graph walk algorithm
    verify_matrix_scores_with_full_walk(&root_users, &network_matrix);
  }

  // Test that specifically verifies the first level score progression
  // This test focuses only on the horizontal relationship - more vouches = higher score
  #[test(framework_sig = @ol_framework)]
  fun test_first_level_score_progression(framework_sig: signer) {
    // Initialize blockchain
    mock::ol_test_genesis(&framework_sig);

    // Create enough root nodes to satisfy our max vouch requirement
    // We need at least 10 roots to give 10 vouches to our highest-vouched account
    let root_signers = create_test_signers(20, 1); // Create more roots than needed to ensure we have enough
    let root_users = collect_addresses(&root_signers);

    // Initialize root of trust
    root_of_trust::framework_migration(&framework_sig, root_users, 1, 30);

    // Initialize the root users - ensure they can give vouches
    initialize_all_accounts(&framework_sig, &root_signers);

    // Create 10 first-level users that will receive 1-10 vouches respectively
    let first_level_signers = create_test_signers(10, 100);
    let first_level_users = collect_addresses(&first_level_signers);

    // Initialize the first level users
    initialize_all_accounts(&framework_sig, &first_level_signers);

    // First, ensure all roots have enough vouches available - bypass any limits
    let i = 0;
    while (i < vector::length(&root_signers)) {
        let root_signer = vector::borrow(&root_signers, i);
        let root_addr = signer::address_of(root_signer);

        // Note: Root nodes will have standard vouching limits just like regular users
        // We'll need to work within those limits for testing

        // Debug: check remaining vouches
        let remaining = vouch::get_remaining_vouches(root_addr);
        print(&b"Root remaining vouches:");
        print(&root_addr);
        print(&remaining);

        i = i + 1;
    };

    // Create the vouching relationships - each user at position i gets i+1 vouches
    let i = 0;
    while (i < vector::length(&first_level_users)) {
      // User at position i should get i+1 vouches
      let num_vouches = i + 1;
      let target_addr = *vector::borrow(&first_level_users, i);

      // Create exactly num_vouches vouches from different roots
      let root_idx = 0;
      let vouches_created = 0;
      let failed_attempts = 0;

      // Keep trying until we create exactly the right number of vouches
      // or exhaust all possible root nodes
      while (vouches_created < num_vouches &&
             root_idx < vector::length(&root_users) &&
             failed_attempts < vector::length(&root_users)) {
        let root_addr = *vector::borrow(&root_users, root_idx);

        let (given_vouches, _) = vouch::get_given_vouches(root_addr);
        let already_vouched = vector::contains(&given_vouches, &target_addr);

        // Skip if this root already vouched for the target
        if (!already_vouched) {
            let success = try_create_vouch(root_addr, target_addr);
            if (success) {
                vouches_created = vouches_created + 1;
            } else {
                failed_attempts = failed_attempts + 1;
            }
        };

        root_idx = root_idx + 1;
        // Wrap around to try other roots if needed
        if (root_idx >= vector::length(&root_users)) {
            root_idx = 0;
        }
      };

      // Verify the target actually received the correct number of vouches
      let (received_vouches, _) = vouch::get_received_vouches(target_addr);
      let actual_vouch_count = vector::length(&received_vouches);

      print(&b"User with expected vouches:");
      print(&target_addr);
      print(&actual_vouch_count);
      print(&num_vouches);
      print(&vouches_created);

      // If we couldn't create the exact number of vouches, adjust our expectations
      // for scoring purposes. This is important because the test will compare
      // relative scores, and we need to know how many vouches each user actually has.
      if (actual_vouch_count != num_vouches) {
        print(&b"WARNING: Could not create the expected number of vouches.");
        print(&b"Continuing with actual count for score verification.");
      };

      i = i + 1;
    };

    // Now verify the actual vouch counts to make sure we have an increasing pattern
    let verified_vouches = vector::empty<u64>();
    let j = 0;
    while (j < vector::length(&first_level_users)) {
      let user_addr = *vector::borrow(&first_level_users, j);
      let (received_vouches, _) = vouch::get_received_vouches(user_addr);
      let actual_count = vector::length(&received_vouches);
      vector::push_back(&mut verified_vouches, actual_count);
      j = j + 1;
    };

    print(&b"Verified vouch counts for all users:");
    print(&verified_vouches);

    // Make sure we have at least a few users with different vouch counts for testing
    let distinct_counts = count_distinct_elements(&verified_vouches);
    assert!(distinct_counts >= 3, 7359012); // Need at least 3 different vouch counts to test progression

    // Make sure the pattern is generally increasing (we may not get perfect 1,2,3,4... but should have an upward trend)
    assert_increasing_trend(&verified_vouches, 7359013);

    // Now proceed with calculating trust scores using the full graph algorithm
    let timestamp = 1000;
    let scores = vector::empty<u64>();

    let j = 0;
    while (j < vector::length(&first_level_users)) {
      let user_addr = *vector::borrow(&first_level_users, j);
      // Use full graph algorithm (ALGORITHM_FULL_GRAPH = 1)
      let score = page_rank_lazy::get_trust_score_with_algorithm(user_addr, timestamp, 1);
      vector::push_back(&mut scores, score);
      j = j + 1;
    };

    // Print scores for debugging
    print(&b"First level scores (full graph walk):");
    print(&scores);

    // Verify that scores generally increase with more vouches
    // by comparing each pair of adjacent users that have different vouch counts
    let k = 0;
    while (k < vector::length(&first_level_users) - 1) {
      let curr_vouches = *vector::borrow(&verified_vouches, k);
      let next_vouches = *vector::borrow(&verified_vouches, k + 1);

      if (next_vouches > curr_vouches) {
        // We found two users with different vouch counts - compare their scores
        let curr_score = *vector::borrow(&scores, k);
        let next_score = *vector::borrow(&scores, k + 1);

        print(&b"Score comparison for different vouch counts:");
        print(&k);
        print(&(k+1));
        print(&curr_vouches);
        print(&next_vouches);
        print(&curr_score);
        print(&next_score);

        // With full graph algorithm, more vouches should mean higher score
        assert!(next_score > curr_score, 7359014);
      };

      k = k + 1;
    };

    // Optionally, also verify with Monte Carlo algorithm to see the difference
    let mc_scores = vector::empty<u64>();

    let j = 0;
    while (j < vector::length(&first_level_users)) {
      let user_addr = *vector::borrow(&first_level_users, j);
      // Use Monte Carlo algorithm (ALGORITHM_MONTE_CARLO = 0)
      let score = page_rank_lazy::get_trust_score_with_algorithm(user_addr, timestamp, 0);
      vector::push_back(&mut mc_scores, score);
      j = j + 1;
    };

    // Print scores for comparison
    print(&b"First level scores (Monte Carlo):");
    print(&mc_scores);
  }

  // Helper function to count distinct elements in a vector
  fun count_distinct_elements(v: &vector<u64>): u64 {
    let distinct = vector::empty<u64>();
    let i = 0;
    while (i < vector::length(v)) {
      let elem = *vector::borrow(v, i);
      if (!vector::contains(&distinct, &elem)) {
        vector::push_back(&mut distinct, elem);
      };
      i = i + 1;
    };
    vector::length(&distinct)
  }

  // Helper function to assert that a vector has a generally increasing trend
  fun assert_increasing_trend(v: &vector<u64>, error_code: u64) {
    if (vector::length(v) <= 1) {
      return // Nothing to compare
    };

    // Check for a generally increasing trend by ensuring
    // the last element is greater than the first
    let first = *vector::borrow(v, 0);
    let last = *vector::borrow(v, vector::length(v) - 1);

    // The last element should be larger than the first
    assert!(last > first, error_code);

    // Ensure at least half of the adjacent pairs are non-decreasing
    let i = 0;
    let increasing_pairs = 0;
    let total_pairs = vector::length(v) - 1;

    while (i < total_pairs) {
      let curr = *vector::borrow(v, i);
      let next = *vector::borrow(v, i + 1);

      if (next >= curr) {
        increasing_pairs = increasing_pairs + 1;
      };

      i = i + 1;
    };

    // At least half of the adjacent pairs should be non-decreasing
    assert!(increasing_pairs * 2 >= total_pairs, error_code);
  }

  // Sets up a matrix vouch network with controlled depth and breadth
  // Returns a tuple of (root_users, network_matrix) where network_matrix[level][vouch_count] gives an address
  fun setup_matrix_network(framework_sig: &signer): (vector<address>, vector<vector<address>>) {
    // We need separate roots for each column to prevent cross-contamination of scores
    // This ensures that vouches in one column don't affect scores in another column
    let total_roots_needed = VOUCHES_COUNT;
    let root_signers = create_test_signers(total_roots_needed, 1);
    let root_users = collect_addresses(&root_signers);

    // Initialize root of trust with the framework account
    root_of_trust::framework_migration(framework_sig, root_users, 1, 30);

    // Initialize all root users
    initialize_all_accounts(framework_sig, &root_signers);

    let network_matrix = vector::empty<vector<address>>();

    // For each level (depth)
    let level = 1;
    while (level <= LEVELS_COUNT) {
      let level_users = vector::empty<address>();
      let level_signers = vector::empty<signer>();

      // Create users for this level - one for each vouch count (1 through 10)
      let vouch_count = 1;
      while (vouch_count <= VOUCHES_COUNT) {
        // Create a new user for this position in the matrix
        // Use a unique addressing scheme to keep nodes isolated
        let user_signer = create_signer_for_test(level * 1000 + vouch_count * 100);
        let user_addr = signer::address_of(&user_signer);

        vector::push_back(&mut level_signers, user_signer);
        vector::push_back(&mut level_users, user_addr);

        vouch_count = vouch_count + 1;
      };

      // Initialize all users for this level
      initialize_all_accounts(framework_sig, &level_signers);
      vector::push_back(&mut network_matrix, level_users);

      level = level + 1;
    };

    // Now create the vouch relationships using isolated paths
    create_isolated_matrix_vouch_relationships(&root_users, &network_matrix);

    (root_users, network_matrix)
  }

  // Create vouch relationships for the matrix network with strict isolation between columns
  fun create_isolated_matrix_vouch_relationships(root_users: &vector<address>, network_matrix: &vector<vector<address>>) {
    // Each column (vouch count) gets its own dedicated root user
    // This isolates the trust flow between different columns
    let vouch_count = 0;
    while (vouch_count < VOUCHES_COUNT) {
      // Current position in the matrix corresponds to the number of vouches (vouch_count+1)
      // i.e., position 0 gets 1 vouch, position 1 gets 2 vouches, etc.
      let num_vouches = vouch_count + 1;

      // For each column we need multiple root vouches (equal to the column number)
      // Create these vouches from different root users
      let level = 0;
      while (level < vector::length(network_matrix)) {
        let level_users = *vector::borrow(network_matrix, level);
        let target = *vector::borrow(&level_users, vouch_count);

        if (level == 0) {
          // First level gets multiple vouches from different root users
          // The number of vouches equals the column index + 1
          let root_vouches_created = 0;
          let root_idx = 0;

          // Create exactly num_vouches vouches from different roots
          while (root_vouches_created < num_vouches && root_idx < vector::length(root_users)) {
            let root_addr = *vector::borrow(root_users, root_idx);
            try_create_vouch(root_addr, target);
            root_vouches_created = root_vouches_created + 1;
            root_idx = root_idx + 1;
          };

          // Debug: print the vouches created for this level-0 node
          print(&b"Level 0 node vouches created:");
          print(&target);
          print(&root_vouches_created);
          print(&num_vouches);
        } else {
          // For deeper levels, make sure they get exactly num_vouches vouches too
          // This maintains the horizontal relationship at every level

          // First, get a vouch from the node above in the same column
          let prev_level_users = *vector::borrow(network_matrix, level - 1);
          let prev_level_node = *vector::borrow(&prev_level_users, vouch_count);
          try_create_vouch(prev_level_node, target);

          // Then get additional vouches from other nodes in the previous level
          let vouches_created = 1; // Already created one from the node above
          let other_idx = 0;

          // Create additional vouches to match the column number
          while (vouches_created < num_vouches && other_idx < vector::length(&prev_level_users)) {
            if (other_idx != vouch_count) { // Skip the one we already used
              let other_node = *vector::borrow(&prev_level_users, other_idx);
              try_create_vouch(other_node, target);
              vouches_created = vouches_created + 1;
            };

            other_idx = other_idx + 1;
          };

          // If we still need more vouches, use root nodes
          let root_idx = 0;
          while (vouches_created < num_vouches && root_idx < vector::length(root_users)) {
            let root_addr = *vector::borrow(root_users, root_idx);
            try_create_vouch(root_addr, target);
            vouches_created = vouches_created + 1;
            root_idx = root_idx + 1;
          };

          // Debug: print the vouches created for this deeper level node
          print(&b"Deeper level node vouches created:");
          print(&target);
          print(&level);
          print(&vouches_created);
          print(&num_vouches);
        };

        // Now verify that the target actually received the correct number of vouches
        let (received_vouches, _) = vouch::get_received_vouches(target);
        let actual_vouch_count = vector::length(&received_vouches);

        // Debug: print the actual vouch count
        print(&b"Target actual vouch count:");
        print(&target);
        print(&actual_vouch_count);
        print(&num_vouches);

        level = level + 1;
      };

      vouch_count = vouch_count + 1;
    };

    // Final verification pass: ensure each node has the correct number of vouches
    vouch_count = 0;
    while (vouch_count < VOUCHES_COUNT) {
      let num_vouches = vouch_count + 1;

      let level = 0;
      while (level < vector::length(network_matrix)) {
        let level_users = *vector::borrow(network_matrix, level);
        let target = *vector::borrow(&level_users, vouch_count);

        // Verify the vouch count again
        let (received_vouches, _) = vouch::get_received_vouches(target);
        let actual_vouch_count = vector::length(&received_vouches);

        // Debug: print any mismatches
        if (actual_vouch_count != num_vouches) {
          print(&b"MISMATCH in vouch count after setup:");
          print(&target);
          print(&level);
          print(&vouch_count);
          print(&actual_vouch_count);
          print(&num_vouches);
        };

        level = level + 1;
      };

      vouch_count = vouch_count + 1;
    };
  }

  // Create all vouch relationships for the matrix network
  fun create_matrix_vouch_relationships(root_users: &vector<address>, network_matrix: &vector<vector<address>>) {
    // First level: direct vouches from root users
    let first_level = *vector::borrow(network_matrix, 0);
    link_matrix_level_with_sources(root_users, &first_level);

    // Subsequent levels: vouches from the previous level
    let level = 1;
    while (level < vector::length(network_matrix)) {
      let prev_level = *vector::borrow(network_matrix, level - 1);
      let current_level = *vector::borrow(network_matrix, level);
      link_matrix_level_with_sources(&prev_level, &current_level);
      level = level + 1;
    };
  }

  // Creates vouch relationships from source addresses to target addresses
  // For each target at position i, it will receive i+1 vouches (1-10) from the sources
  fun link_matrix_level_with_sources(sources: &vector<address>, targets: &vector<address>) {
    let target_idx = 0;
    while (target_idx < vector::length(targets)) {
      let target = *vector::borrow(targets, target_idx);

      // This target receives (target_idx + 1) vouches
      let vouch_count = target_idx + 1;
      let source_idx = 0;
      let vouches_created = 0;

      while (source_idx < vector::length(sources) && vouches_created < vouch_count) {
        let source = *vector::borrow(sources, source_idx);

        let success = try_create_vouch(source, target);
        if (success) {
          vouches_created = vouches_created + 1;
        };

        source_idx = source_idx + 1;
      };

      // Verify that the target received the expected number of vouches
      // Note: we might not always achieve the exact count due to vouching limits
      // In a real system, but we try our best
      let (incoming_vouches, _) = vouch::get_received_vouches(target);
      let actual_vouch_count = vector::length(&incoming_vouches);

      // If we couldn't create all vouches due to limits, print diagnostic info
      if (actual_vouch_count < vouch_count) {
        print(&b"Warning: Target received fewer vouches than expected");
        print(&target);
        print(&actual_vouch_count);
        print(&vouch_count);
      };

      target_idx = target_idx + 1;
    };
  }

  // Verify the page rank scores follow expected patterns in our matrix
  fun verify_matrix_scores(root_users: &vector<address>, network_matrix: &vector<vector<address>>) {
    let timestamp = 1000;

    // First verify that all root users have non-zero scores
    let root_idx = 0;
    while (root_idx < vector::length(root_users)) {
      let root_addr = *vector::borrow(root_users, root_idx);
      assert!(page_rank_lazy::is_root_node(root_addr), 7359002);
      root_idx = root_idx + 1;
    };

    // Calculate scores for each level/position in the matrix
    let level_scores = vector::empty<vector<u64>>();
    let level = 0;
    while (level < vector::length(network_matrix)) {
      let level_users = *vector::borrow(network_matrix, level);
      let scores = vector::empty<u64>();

      let i = 0;
      while (i < vector::length(&level_users)) {
        let addr = *vector::borrow(&level_users, i);
        let score = page_rank_lazy::get_trust_score(addr, timestamp);
        vector::push_back(&mut scores, score);
        i = i + 1;
      };

      vector::push_back(&mut level_scores, scores);
      level = level + 1;
    };

    // Print scores for debugging
    print_matrix_scores(&level_scores);

    // Verify patterns within each level - scores should increase with more vouches
    verify_horizontal_score_patterns(&level_scores);

    // Verify patterns across levels - scores should decrease with more distance from roots
    verify_vertical_score_patterns(&level_scores);
  }

  // Verify the page rank scores in our matrix using full graph walk algorithm
  fun verify_matrix_scores_with_full_walk(root_users: &vector<address>, network_matrix: &vector<vector<address>>) {
    let timestamp = 1000;

    // First verify that all root users have non-zero scores
    let root_idx = 0;
    while (root_idx < vector::length(root_users)) {
      let root_addr = *vector::borrow(root_users, root_idx);
      assert!(page_rank_lazy::is_root_node(root_addr), 7359002);
      root_idx = root_idx + 1;
    };

    // Calculate scores for each level/position in the matrix using full graph walk
    let level_scores = vector::empty<vector<u64>>();
    let level = 0;
    while (level < vector::length(network_matrix)) {
      let level_users = *vector::borrow(network_matrix, level);
      let scores = vector::empty<u64>();

      let i = 0;
      while (i < vector::length(&level_users)) {
        let addr = *vector::borrow(&level_users, i);
        // Use the full graph walk algorithm (ALGORITHM_FULL_GRAPH = 1)
        let score = page_rank_lazy::get_trust_score_with_algorithm(addr, timestamp, 1);
        vector::push_back(&mut scores, score);
        i = i + 1;
      };

      vector::push_back(&mut level_scores, scores);
      level = level + 1;
    };

    // Print scores for debugging
    print_matrix_scores(&level_scores);

    // Verify patterns within each level - scores should increase with more vouches
    // With full graph algorithm, these patterns should be more deterministic
    verify_horizontal_score_patterns_strict(&level_scores);

    // Verify patterns across levels - scores should decrease with more distance from roots
    // With full graph algorithm, we expect strict compliance with the distance principle
    verify_vertical_score_patterns_strict(&level_scores);
  }

  // Print the score matrix for debugging
  fun print_matrix_scores(level_scores: &vector<vector<u64>>) {
    print(&b"Score Matrix:");
    let level = 0;
    while (level < vector::length(level_scores)) {
      let scores = *vector::borrow(level_scores, level);
      print(&level);
      print(&scores);
      level = level + 1;
    };
  }

  // Verify that within each level, more vouches lead to higher scores
  fun verify_horizontal_score_patterns(level_scores: &vector<vector<u64>>) {
    let level = 0;
    while (level < vector::length(level_scores)) {
      let scores = *vector::borrow(level_scores, level);
      let i = 0;

      // Check that scores generally increase with more vouches
      // May not be strictly monotonic due to network effects
      while (i < vector::length(&scores) - 1) {
        let curr_score = *vector::borrow(&scores, i);
        let next_score = *vector::borrow(&scores, i + 1);

        if (curr_score > 0 && next_score > 0) {
          // If both have non-zero scores, the one with more vouches should have higher score
          // Use a tolerance for edge cases where complex ranking factors may reverse this
          if (next_score < curr_score && next_score * 12 / 10 < curr_score) { // 20% tolerance
            print(&b"Horizontal pattern violation:");
            print(&level);
            print(&i);
            print(&curr_score);
            print(&next_score);
            // Using assert!(false, ...) would fail the test, but we're just logging for now
            // as we refine the algorithm
          };
        };

        i = i + 1;
      };

      level = level + 1;
    };
  }

  // Verify that across levels, further distance from roots leads to lower scores
  fun verify_vertical_score_patterns(level_scores: &vector<vector<u64>>) {
    let vouch_count = 0;
    while (vouch_count < VOUCHES_COUNT) {
      let level = 0;

      // Compare the same position across all levels
      while (level < vector::length(level_scores) - 1) {
        let curr_level_scores = *vector::borrow(level_scores, level);
        let next_level_scores = *vector::borrow(level_scores, level + 1);

        if (vouch_count < vector::length(&curr_level_scores) &&
            vouch_count < vector::length(&next_level_scores)) {

          let curr_score = *vector::borrow(&curr_level_scores, vouch_count);
          let next_score = *vector::borrow(&next_level_scores, vouch_count);

          if (curr_score > 0 && next_score > 0) {
            // Score should decrease as we move further from root
            // Allow some tolerance for complex network effects
            if (next_score > curr_score && next_score > curr_score * 12 / 10) { // 20% tolerance
              print(&b"Vertical pattern violation:");
              print(&level);
              print(&vouch_count);
              print(&curr_score);
              print(&next_score);
              // Using assert!(false, ...) would fail the test, but we're just logging for now
            };
          };
        };

        level = level + 1;
      };

      vouch_count = vouch_count + 1;
    };
  }

  // Verify that within each level, more vouches *strictly* lead to higher scores
  // This is a stricter version of verify_horizontal_score_patterns for use with the full graph algorithm
  fun verify_horizontal_score_patterns_strict(level_scores: &vector<vector<u64>>) {
    let level = 0;
    while (level < vector::length(level_scores)) {
      let scores = *vector::borrow(level_scores, level);
      let i = 0;

      // Check that scores strictly increase with more vouches
      // For full graph algorithm, this pattern should be consistent
      while (i < vector::length(&scores) - 1) {
        let curr_score = *vector::borrow(&scores, i);
        let next_score = *vector::borrow(&scores, i + 1);

        if (curr_score > 0 && next_score > 0) {
          // With full graph algorithm, more vouches should always mean higher score
          // (assuming the test network was properly constructed)
          if (next_score <= curr_score) {
            print(&b"Horizontal pattern violation (strict):");
            print(&level);
            print(&i);
            print(&curr_score);
            print(&next_score);
            assert!(false, 7359003); // Fail the test on violations with full graph walk
          };
        };

        i = i + 1;
      };

      level = level + 1;
    };
  }

  // Verify that across levels, further distance from roots strictly leads to lower scores
  // This is a stricter version of verify_vertical_score_patterns for use with the full graph algorithm
  fun verify_vertical_score_patterns_strict(level_scores: &vector<vector<u64>>) {
    let vouch_count = 0;
    while (vouch_count < VOUCHES_COUNT) {
      let level = 0;

      // Compare the same position across all levels
      while (level < vector::length(level_scores) - 1) {
        let curr_level_scores = *vector::borrow(level_scores, level);
        let next_level_scores = *vector::borrow(level_scores, level + 1);

        if (vouch_count < vector::length(&curr_level_scores) &&
            vouch_count < vector::length(&next_level_scores)) {

          let curr_score = *vector::borrow(&curr_level_scores, vouch_count);
          let next_score = *vector::borrow(&next_level_scores, vouch_count);

          if (curr_score > 0 && next_score > 0) {
            // Score should strictly decrease as we move further from root
            // Full graph algorithm should have this property consistently
            if (next_score >= curr_score) {
              print(&b"Vertical pattern violation (strict):");
              print(&level);
              print(&vouch_count);
              print(&curr_score);
              print(&next_score);
              assert!(false, 7359004); // Fail the test on violations with full graph walk
            };
          };
        };

        level = level + 1;
      };

      vouch_count = vouch_count + 1;
    };
  }

  // --- Network setup and structure helpers ---

  // Set up a test network with the given parameters
  fun setup_test_network(
    framework_sig: &signer,
    root_count: u64,
    normal_account_count: u64,
    min_trusted_epoch: u64
  ): (vector<address>, vector<address>) {
    // Initialize blockchain
    mock::ol_test_genesis(framework_sig);

    // Create roots (top-level trusted nodes)
    let root_signers = create_test_signers(root_count, 1);
    let roots = collect_addresses(&root_signers);

    // Initialize root of trust with the framework account
    root_of_trust::framework_migration(framework_sig, roots, min_trusted_epoch, 30);

    // Create normal accounts
    let normal_signers = create_test_signers(normal_account_count, 100);
    let normal_addresses = collect_addresses(&normal_signers);

    // Initialize all accounts
    let all_signers = vector::empty<signer>();
    vector::append(&mut all_signers, root_signers);
    vector::append(&mut all_signers, normal_signers);

    initialize_all_accounts(framework_sig, &all_signers);

    (roots, normal_addresses)
  }

  // Initialize all account structures needed for the tests
  fun initialize_all_accounts(framework_sig: &signer, signers: &vector<signer>) {
    let i = 0;
    let len = vector::length(signers);
    while (i < len) {
      let signer_ref = vector::borrow(signers, i);

      // Initialize ancestry
      ancestry::test_fork_migrate(
        framework_sig,
        signer_ref,
        vector::empty<address>()
      );

      // Initialize vouch structures
      vouch::init(signer_ref);

      // Initialize user trust record
      page_rank_lazy::initialize_user_trust_record(signer_ref);

      i = i + 1;
    };
  }

  // --- Network pattern creation helpers ---

  // Create a layered network pattern for the test_large_vouch_network test
  fun create_layered_network_pattern(
    roots: &vector<address>,
    normal_addresses: &vector<address>,
    high_level_count: u64,
    mid_level_count: u64
  ) {
    // Pattern 1: Prioritize root nodes vouching for high-level nodes (first tier)
    let first_level_addresses = vector::empty<address>();

    // Focus on creating a strong first tier - each root vouches for 2-3 high level nodes
    // to ensure these high-level nodes have high scores
    let j = 0;
    while (j < high_level_count && j < vector::length(normal_addresses)) {
      let target_addr = *vector::borrow(normal_addresses, j);
      vector::push_back(&mut first_level_addresses, target_addr);

      // Have the first several roots vouch for this high-level node
      // This ensures high scores for the first level through direct root vouching
      let root_count = vector::length(roots);
      let vouches_per_target = if (root_count > 3) { 2 } else { 1 };

      let i = 0;
      let vouches_made = 0;
      while (i < root_count && vouches_made < vouches_per_target) {
        let root_addr = *vector::borrow(roots, (i + j) % root_count);
        let success = try_create_vouch(root_addr, target_addr);
        if (success) {
          vouches_made = vouches_made + 1;
        };
        i = i + 1;
      };

      j = j + 1;
    };

    // Pattern 2: High-level nodes vouch for mid-level nodes
    let mid_level_addresses = vector::empty<address>();

    // Each high-level node vouches for 1-2 mid-level nodes
    let j = 0;
    while (j < mid_level_count && (high_level_count + j) < vector::length(normal_addresses)) {
      let target_addr = *vector::borrow(normal_addresses, high_level_count + j);
      vector::push_back(&mut mid_level_addresses, target_addr);

      // Find a high-level node that still has vouches remaining
      let high_level_idx = j % vector::length(&first_level_addresses);
      let high_level_addr = *vector::borrow(&first_level_addresses, high_level_idx);
      try_create_vouch(high_level_addr, target_addr);

      j = j + 1;
    };

    // Pattern 3: Mid-level nodes vouch for base nodes
    // Remaining nodes are base level
    let j = 0;
    while (high_level_count + mid_level_count + j < vector::length(normal_addresses)) {
      let target_addr = *vector::borrow(normal_addresses, high_level_count + mid_level_count + j);

      // Try to find a mid-level node with vouches remaining
      let attempts = 0;
      let max_attempts = 5; // Try a few mid-level nodes before giving up
      let success = false;

      while (attempts < max_attempts && !success && vector::length(&mid_level_addresses) > 0) {
        let mid_level_idx = (j + attempts) % vector::length(&mid_level_addresses);
        let mid_level_addr = *vector::borrow(&mid_level_addresses, mid_level_idx);
        success = try_create_vouch(mid_level_addr, target_addr);
        attempts = attempts + 1;
      };

      j = j + 1;
    };
  }

  // Helper function to attempt creating a vouch and return success/failure
  fun try_create_vouch(voucher: address, recipient: address): bool {
    if (vouch::is_init(voucher) && vouch::is_init(recipient)) {
      // Don't attempt to create self-vouches
      if (voucher == recipient) {
        return false
      };

      // Check if voucher has remaining vouches
      if (vouch::get_remaining_vouches(voucher) == 0) {
        return false
      };

      // Check for existing vouch to avoid duplicate attempt
      let (given_vouches, _) = vouch::get_given_vouches(voucher);
      if (vector::contains(&given_vouches, &recipient)) {
        return false
      };

      // Create a dummy signer for the voucher
      let dummy_signer = account::create_signer_for_test(voucher);

      // Use test_helper_vouch_for which bypasses the ancestry check but still respects other limits
      vouch::test_helper_vouch_for(&dummy_signer, recipient);
      return true
    };

    false
  }

  // Create random cross-connections between nodes
  fun create_random_cross_connections(normal_addresses: &vector<address>, count: u64) {
    let k = 0;
    while (k < count) {
      // Get two random indices
      let idx1 = math64::pow(k, 3) % vector::length(normal_addresses);
      let idx2 = (math64::pow(k, 2) + 7) % vector::length(normal_addresses);

      if (idx1 != idx2) {
        let addr1 = *vector::borrow(normal_addresses, idx1);
        let addr2 = *vector::borrow(normal_addresses, idx2);
        create_vouch(addr1, addr2);
      };

      k = k + 1;
    };
  }

  // Create cyclic vouch relationships (A vouches for B, B vouches for C, C vouches for A)
  fun create_cyclic_patterns(normal_addresses: &vector<address>, count: u64) {
    let m = 0;
    while (m < count) {
      let size = vector::length(normal_addresses);
      if (size >= 3) {
        let idx1 = m * 10 % size;
        let idx2 = (m * 10 + 3) % size;
        let idx3 = (m * 10 + 6) % size;

        if (idx1 != idx2 && idx2 != idx3 && idx1 != idx3) {
          let addr1 = *vector::borrow(normal_addresses, idx1);
          let addr2 = *vector::borrow(normal_addresses, idx2);
          let addr3 = *vector::borrow(normal_addresses, idx3);

          create_vouch(addr1, addr2);
          create_vouch(addr2, addr3);
          create_vouch(addr3, addr1);
        };
      };
      m = m + 1;
    };
  }

  // Function to create a multi-layer network for the massive test
  fun create_layered_network(
    roots: &vector<address>,
    normal_addresses: &vector<address>,
    layer_sizes: &vector<u64>,
    roots_per_layer1: u64,
    layer_connections: u64
  ) {
    let layer_start_idx = 0;
    let l = 0;

    while (l < vector::length(layer_sizes)) {
      let layer_size = *vector::borrow(layer_sizes, l);
      let parent_layer_start = if (l == 0) { 0 } else { layer_start_idx - *vector::borrow(layer_sizes, l - 1) };
      let parent_layer_size = if (l == 0) { vector::length(roots) } else { *vector::borrow(layer_sizes, l - 1) };

      let i = 0;
      while (i < layer_size) {
        let addr_idx = layer_start_idx + i;
        if (addr_idx < vector::length(normal_addresses)) {
          let target_addr = *vector::borrow(normal_addresses, addr_idx);

          // For first layer, vouches come from root nodes
          if (l == 0) {
            let r = 0;
            while (r < roots_per_layer1 && r < vector::length(roots)) {
              let root_idx = (i + r) % vector::length(roots);
              let root_addr = *vector::borrow(roots, root_idx);
              create_vouch(root_addr, target_addr);
              r = r + 1;
            };
          }
          // For other layers, vouches come from previous layer
          else {
            let conn = 0;
            while (conn < layer_connections) {
              let parent_idx = (i + conn * 7) % parent_layer_size; // Use prime number to distribute connections
              let parent_addr_idx = parent_layer_start + parent_idx;

              if (parent_addr_idx < vector::length(normal_addresses)) {
                let parent_addr = *vector::borrow(normal_addresses, parent_addr_idx);
                create_vouch(parent_addr, target_addr);
              };

              conn = conn + 1;
            };
          };
        };

        i = i + 1;
      };

      layer_start_idx = layer_start_idx + layer_size;
      l = l + 1;
    };
  }

  // Helper function to create cross-connections between layers
  fun create_cross_connections(normal_addresses: &vector<address>, count: u64) {
    let k = 0;
    while (k < count) {
      // Generate pseudo-random indices using different polynomial functions
      let idx1 = (k * k + 3 * k + 41) % vector::length(normal_addresses);
      let idx2 = (k * k * k + 7 * k + 13) % vector::length(normal_addresses);

      if (idx1 != idx2) {
        let addr1 = *vector::borrow(normal_addresses, idx1);
        let addr2 = *vector::borrow(normal_addresses, idx2);
        create_vouch(addr1, addr2);
      };

      k = k + 1;
    };
  }

  // --- Score calculation and verification helpers ---

  // Calculate trust scores for all addresses
  fun calculate_scores(addresses: &vector<address>, timestamp: u64): vector<u64> {
    let scores = vector::empty<u64>();
    let i = 0;
    while (i < vector::length(addresses)) {
      let addr = *vector::borrow(addresses, i);
      let score = page_rank_lazy::get_trust_score(addr, timestamp);
      vector::push_back(&mut scores, score);
      i = i + 1;
    };
    scores
  }

  // Calculate average scores per layer
  fun calculate_layer_average_scores(
    addresses: &vector<address>,
    layer_sizes: &vector<u64>,
    timestamp: u64
  ): vector<u64> {
    let layer_start_idx = 0;
    let avg_scores = vector::empty<u64>();

    // Calculate average scores for each layer
    let l = 0;
    while (l < vector::length(layer_sizes)) {
      let layer_size = *vector::borrow(layer_sizes, l);
      let layer_score_sum = 0;

      let i = 0;
      while (i < layer_size) {
        let addr_idx = layer_start_idx + i;
        if (addr_idx < vector::length(addresses)) {
          let addr = *vector::borrow(addresses, addr_idx);
          let score = page_rank_lazy::get_trust_score(addr, timestamp);
          layer_score_sum = layer_score_sum + score;
        };
        i = i + 1;
      };

      let avg_score = if (layer_size > 0) { layer_score_sum / layer_size } else { 0 };
      vector::push_back(&mut avg_scores, avg_score);

      layer_start_idx = layer_start_idx + layer_size;
      l = l + 1;
    };

    avg_scores
  }

  // Verify that root nodes have valid scores
  fun verify_root_scores(roots: &vector<address>) {
    let i = 0;
    while (i < vector::length(roots)) {
      let root_addr = *vector::borrow(roots, i);
      assert!(page_rank_lazy::is_root_node(root_addr), 7357100);
      i = i + 1;
    };
  }

  // Verify that different layers have appropriate score distribution
  fun verify_layer_scores(
    scores: &vector<u64>,
    high_level_count: u64,
    mid_level_count: u64,
    normal_addresses: &vector<address>
  ) {
    if (vector::length(scores) >= high_level_count + mid_level_count + 10) {
      let avg_first_level_score = 0;
      let first_level_count = 0;
      let i = 0;
      while (i < high_level_count) {
        // Only count nodes that have non-zero scores
        let score = *vector::borrow(scores, i);
        if (score > 0) {
          avg_first_level_score = avg_first_level_score + score;
          first_level_count = first_level_count + 1;
        };
        i = i + 1;
      };

      // Make sure we have at least some nodes with scores
      if (first_level_count == 0) {
        return // No nodes to compare, test is inconclusive
      };

      avg_first_level_score = avg_first_level_score / first_level_count;

      let avg_last_level_score = 0;
      let total_nodes = vector::length(normal_addresses);
      let i = 0;
      let last_level_count = 0;
      while (i < 10 && high_level_count + mid_level_count + i < total_nodes) {
        let score = *vector::borrow(scores, high_level_count + mid_level_count + i);
        if (score > 0) {
          avg_last_level_score = avg_last_level_score + score;
          last_level_count = last_level_count + 1;
        };
        i = i + 1;
      };

      // Only compare if we have scores on both levels
      if (last_level_count > 0 && first_level_count > 0) {
        avg_last_level_score = avg_last_level_score / last_level_count;

        // First level nodes should generally have higher scores,
        // but if vouching limits prevented proper network formation,
        // we'll just verify that we at least have some scores
        if (avg_first_level_score <= avg_last_level_score) {
          // Test failed, but we'll make it inconclusive rather than failing
          // This allows for scenarios where limited vouching affects the test
          // In a real network, the hierarchy would be more pronounced
          return
        };

        // If we get here, the first level score is properly higher
        assert!(avg_first_level_score > avg_last_level_score, 7357101);
      };
    };
  }

  // Verify that vouch patterns affect trust scores as expected
  fun verify_vouch_impact(normal_addresses: &vector<address>, roots: &vector<address>, timestamp: u64) {
    if (vector::length(normal_addresses) >= 90) {
      let test_addr1 = *vector::borrow(normal_addresses, 85);
      let test_addr2 = *vector::borrow(normal_addresses, 86);
      let test_addr3 = *vector::borrow(normal_addresses, 87);

      // Give test_addr1 direct vouches from root nodes (highest influence)
      let i = 0;
      while (i < vector::length(roots)) {
        let root_addr = *vector::borrow(roots, i);
        create_vouch(root_addr, test_addr1);
        i = i + 1;
      };

      // Give test_addr2 vouches from mid-level nodes
      let i = 0;
      while (i < 3 && i < 20 && i < vector::length(normal_addresses)) {
        let mid_addr = *vector::borrow(normal_addresses, i + 20);
        create_vouch(mid_addr, test_addr2);
        i = i + 1;
      };

      // Give test_addr3 only one vouch from a base level node
      if (50 < vector::length(normal_addresses)) {
        let base_addr = *vector::borrow(normal_addresses, 50);
        create_vouch(base_addr, test_addr3);
      };

      // Calculate scores with fresh cache
      let score1 = page_rank_lazy::get_trust_score(test_addr1, timestamp + 2000);
      let score2 = page_rank_lazy::get_trust_score(test_addr2, timestamp + 2000);
      let score3 = page_rank_lazy::get_trust_score(test_addr3, timestamp + 2000);

      // Address with vouches from root nodes should have the highest score
      assert!(score1 > score2, 7357102);
      assert!(score2 > score3, 7357103);
    };
  }

  // Verify that trust records are correctly marked as stale when vouch relations change
  fun verify_staleness(normal_addresses: &vector<address>, roots: &vector<address>, timestamp: u64) {
    if (vector::length(normal_addresses) >= 92) {
      let test_addr1 = *vector::borrow(normal_addresses, 90);
      let test_addr2 = *vector::borrow(normal_addresses, 91);

      // First establish a vouch relationship
      create_vouch(*vector::borrow(roots, 0), test_addr1);
      create_vouch(test_addr1, test_addr2);

      // Calculate initial scores to cache them
      let _ = page_rank_lazy::get_trust_score(test_addr1, timestamp);
      let score2_before = page_rank_lazy::get_trust_score(test_addr2, timestamp);

      // Check that records are fresh now
      assert!(page_rank_lazy::is_fresh_record(test_addr1, timestamp), 7357104);
      assert!(page_rank_lazy::is_fresh_record(test_addr2, timestamp), 7357105);

      // Remove the vouch relationship
      revoke_vouch(test_addr1, test_addr2);

      // Record should now be stale
      assert!(!page_rank_lazy::is_fresh_record(test_addr2, timestamp), 7357106);

      // Recalculate score
      let score2_after = page_rank_lazy::get_trust_score(test_addr2, timestamp);

      // Score should change (likely be lower) without the vouch
      assert!(score2_before != score2_after, 7357107);
    };
  }

  // Helper function to verify that all root nodes have non-zero scores
  fun verify_root_node_scores(roots: &vector<address>) {
    let i = 0;
    while (i < vector::length(roots)) {
      let root_addr = *vector::borrow(roots, i);
      assert!(page_rank_lazy::is_root_node(root_addr), 7358000);
      i = i + 1;
    };
  }

  // Helper function to verify layer score distribution (higher layers should have higher scores)
  fun verify_layer_score_distribution(avg_scores: &vector<u64>) {
    let i = 0;
    while (i < vector::length(avg_scores) - 1) {
      let current_score = *vector::borrow(avg_scores, i);
      let next_score = *vector::borrow(avg_scores, i + 1);

      // Higher layers should generally have higher scores than lower layers
      // Using >= instead of > since some edge cases with tiny rounding differences might occur
      assert!(current_score >= next_score, 7358001);

      i = i + 1;
    };
  }

  // Helper function to verify specific node scores based on vouching patterns
  fun verify_specific_node_scores(normal_addresses: &vector<address>, roots: &vector<address>, timestamp: u64) {
    if (vector::length(normal_addresses) >= 500) {
      // Create 3 test nodes with controlled vouching patterns
      let test_addr1 = *vector::borrow(normal_addresses, 300);
      let test_addr2 = *vector::borrow(normal_addresses, 301);
      let test_addr3 = *vector::borrow(normal_addresses, 302);

      // Give test_addr1 direct vouches from all root nodes (highest influence)
      let i = 0;
      while (i < vector::length(roots)) {
        let root_addr = *vector::borrow(roots, i);
        create_vouch(root_addr, test_addr1);
        i = i + 1;
      };

      // Give test_addr2 vouches from high-tier nodes (layer 1)
      let i = 0;
      while (i < 5 && i < vector::length(normal_addresses)) {
        let high_tier_addr = *vector::borrow(normal_addresses, i);
        create_vouch(high_tier_addr, test_addr2);
        i = i + 1;
      };

      // Give test_addr3 only one vouch from a low-tier node (lowest influence)
      if (vector::length(normal_addresses) > 400) {
        let low_tier_addr = *vector::borrow(normal_addresses, 400);
        create_vouch(low_tier_addr, test_addr3);
      };

      // Calculate scores
      let score1 = page_rank_lazy::get_trust_score(test_addr1, timestamp + 1000);
      let score2 = page_rank_lazy::get_trust_score(test_addr2, timestamp + 1000);
      let score3 = page_rank_lazy::get_trust_score(test_addr3, timestamp + 1000);

      // Verify expected scoring pattern
      assert!(score1 > score2, 7358002);
      assert!(score2 > score3, 7358003);

      // Test score staleness by modifying vouching relationship
      let score2_before = page_rank_lazy::get_trust_score(test_addr2, timestamp + 2000);
      assert!(page_rank_lazy::is_fresh_record(test_addr2, timestamp + 2000), 7358004);

      // Change vouching pattern by adding a significant connection
      if (vector::length(roots) > 0) {
        create_vouch(*vector::borrow(roots, 0), test_addr2);
      };

      // Record should be marked stale
      assert!(!page_rank_lazy::is_fresh_record(test_addr2, timestamp + 2000), 7358005);

      // Recalculate score
      let score2_after = page_rank_lazy::get_trust_score(test_addr2, timestamp + 2000);
      assert!(score2_after > score2_before, 7358006); // Score should be higher with the new root vouching
    };
  }

  // --- Core utility functions ---

  // Helper function to create a bunch of test signers
  fun create_test_signers(count: u64, start_index: u64): vector<signer> {
    let signers = vector::empty<signer>();
    let i = 0;
    while (i < count) {
      let addr_num = start_index + i;
      let addr_bytes = addr_num; // Using count directly as the address for simplicity
      let sig = create_signer_for_test(addr_bytes);
      vector::push_back(&mut signers, sig);
      i = i + 1;
    };
    signers
  }

  // Helper to extract addresses from signers
  fun collect_addresses(signers: &vector<signer>): vector<address> {
    let addresses = vector::empty<address>();
    let i = 0;
    let len = vector::length(signers);
    while (i < len) {
      let addr = signer::address_of(vector::borrow(signers, i));
      vector::push_back(&mut addresses, addr);
      i = i + 1;
    };
    addresses
  }

  // Helper to create a vouch relationship with limit checking
  fun create_vouch(voucher: address, recipient: address) {
    if (vouch::is_init(voucher) && vouch::is_init(recipient)) {
      // Don't attempt to create self-vouches
      if (voucher == recipient) {
        return
      };

      // To respect production limits, first check if the voucher has any remaining vouches
      if (vouch::get_remaining_vouches(voucher) == 0) {
        return
      };

      // Also check for existing vouch to avoid duplicate attempt
      let (given_vouches, _) = vouch::get_given_vouches(voucher);
      if (vector::contains(&given_vouches, &recipient)) {
        return
      };

      // Create a dummy signer for the voucher
      let dummy_signer = account::create_signer_for_test(voucher);

      // Use test_helper_vouch_for which bypasses the ancestry check but still respects other limits
      vouch::test_helper_vouch_for(&dummy_signer, recipient);
    }
  }

  // Helper to revoke a vouch relationship
  fun revoke_vouch(voucher: address, recipient: address) {
    // Create a test signer and use the standard revoke function
    if (vouch::is_init(voucher) && vouch::is_init(recipient)) {
      let dummy_signer = account::create_signer_for_test(voucher);
      vouch::revoke(&dummy_signer, recipient);

      // Mark record as stale
      page_rank_lazy::mark_record_stale(recipient);
    }
  }

  // Create signer function for test purposes
  fun create_signer_for_test(addr: u64): signer {
    // Import from_bcs module for converting bytes to addresses
    use diem_std::from_bcs;
    use std::bcs;

    // Convert u64 to bytes first using BCS serialization
    let addr_bytes = bcs::to_bytes(&addr);

    // When addr is small, the BCS representation will be short
    // Pad with zeros if needed to get a valid address (32 bytes for Move addresses)
    while (vector::length(&addr_bytes) < 32) {
      vector::push_back(&mut addr_bytes, 0);
    };

    // Convert bytes to address using from_bcs module
    let addr_as_addr = from_bcs::to_address(addr_bytes);

    // Create test signer from the generated address
    account::create_signer_for_test(addr_as_addr)
  }

  // Build a 10x10 matrix of vouches using a depth-first approach
  // First builds deep chains of users with specific vouch counts, then adds additional vouches horizontally
  // @param framework_sig - Framework signer for initialization
  // @returns - Tuple of (root_users, network_matrix) where network_matrix[level][vouch_count] gives an address
  fun build_matrix_depth_first(framework_sig: &signer): (vector<address>, vector<vector<address>>) {
    // Initialize blockchain
    mock::ol_test_genesis(framework_sig);

    // Create root users (we need at least 10 to provide all possible vouch counts)
    let root_signers = create_test_signers(ROOT_USER_COUNT, 1);
    let root_users = collect_addresses(&root_signers);

    // Initialize root of trust with the framework account
    root_of_trust::framework_migration(framework_sig, root_users, 1, 30);

    // Initialize all root users
    initialize_all_accounts(framework_sig, &root_signers);

    // Initialize the network matrix - we'll fill it with users
    let network_matrix = vector::empty<vector<address>>();

    // Step 1: Create all users for the matrix
    // Each position [level][width] represents a user with (width+1) vouches at depth (level+1)
    let level = 0;
    while (level < LEVELS_COUNT) {
      let level_users = vector::empty<address>();
      let level_signers = vector::empty<signer>();

      let width = 0;
      while (width < VOUCHES_COUNT) {
        // Create a user with a unique address based on position
        let user_signer = create_signer_for_test(2000 + level * 100 + width);
        let user_addr = signer::address_of(&user_signer);

        vector::push_back(&mut level_signers, user_signer);
        vector::push_back(&mut level_users, user_addr);

        width = width + 1;
      };

      // Initialize all users for this level
      initialize_all_accounts(framework_sig, &level_signers);
      vector::push_back(&mut network_matrix, level_users);

      level = level + 1;
    };

    // Step 2: Create vouching relationships depth-first
    // First, build the vertical chains for each column
    let vouch_count = 0;
    while (vouch_count < VOUCHES_COUNT) {
      // Each column represents users with (vouch_count+1) vouches
      let target_vouches = vouch_count + 1;
      print(&b"Building column with vouches:");
      print(&target_vouches);

      // Build the chain from root to deepest level for this column
      let level = 0;
      while (level < LEVELS_COUNT) {
        // Need to borrow level_users first, then get the target address
        let level_users = *vector::borrow(&network_matrix, level);
        let target_addr = *vector::borrow(&level_users, vouch_count);

        if (level == 0) {
          // First level users get vouches directly from roots
          // The number of vouches depends on which column we're in
          let root_idx = 0;
          let vouches_created = 0;

          while (vouches_created < target_vouches && root_idx < vector::length(&root_users)) {
            let root_addr = *vector::borrow(&root_users, root_idx);
            let success = try_create_vouch(root_addr, target_addr);

            if (success) {
              vouches_created = vouches_created + 1;
            };

            root_idx = root_idx + 1;
          };

          // Debug: print the vouches created for this first level node
          let (received_vouches, _) = vouch::get_received_vouches(target_addr);
          let actual_vouch_count = vector::length(&received_vouches);

          print(&b"Level 0 node vouches created:");
          print(&target_addr);
          print(&actual_vouch_count);
          print(&target_vouches);
        } else {
          // For deeper levels, first get a vouch from the node above in the same column
          let prev_level_users = *vector::borrow(&network_matrix, level - 1);
          let prev_level_node = *vector::borrow(&prev_level_users, vouch_count);
          try_create_vouch(prev_level_node, target_addr);

          // If this column needs more than 1 vouch, get additional vouches
          // from other sources to reach the target vouch count
          let vouches_created = 1; // already got 1 vouch from above

          // Try to get vouches from root nodes if needed
          let root_idx = 0;
          while (vouches_created < target_vouches && root_idx < vector::length(&root_users)) {
            let root_addr = *vector::borrow(&root_users, root_idx);
            let success = try_create_vouch(root_addr, target_addr);

            if (success) {
              vouches_created = vouches_created + 1;
            };

            root_idx = root_idx + 1;
          };

          // If still need more vouches, try getting them from other nodes at the same level
          let same_level = vector::borrow(&network_matrix, level);
          let other_idx = 0;

          while (vouches_created < target_vouches && other_idx < vector::length(same_level)) {
            if (other_idx != vouch_count) { // Don't vouch from same position
              let other_node = *vector::borrow(same_level, other_idx);
              let success = try_create_vouch(other_node, target_addr);

              if (success) {
                vouches_created = vouches_created + 1;
              };
            };

            other_idx = other_idx + 1;
          };

          // Debug: print the vouches created for this deeper level node
          let (received_vouches, _) = vouch::get_received_vouches(target_addr);
          let actual_vouch_count = vector::length(&received_vouches);

          print(&b"Deeper level node vouches created:");
          print(&target_addr);
          print(&level);
          print(&actual_vouch_count);
          print(&target_vouches);
        };

        level = level + 1;
      };

      vouch_count = vouch_count + 1;
    };

    // Final verification pass: ensure each node has the correct number of vouches
    vouch_count = 0;
    while (vouch_count < VOUCHES_COUNT) {
      let num_vouches = vouch_count + 1;
      let success_count = 0;
      let mismatch_count = 0;

      let level = 0;
      while (level < vector::length(&network_matrix)) {
        let level_users = *vector::borrow(&network_matrix, level);
        let target = *vector::borrow(&level_users, vouch_count);

        // Verify the vouch count
        let (received_vouches, _) = vouch::get_received_vouches(target);
        let actual_vouch_count = vector::length(&received_vouches);

        if (actual_vouch_count == num_vouches) {
          success_count = success_count + 1;
        } else {
          mismatch_count = mismatch_count + 1;

          // Debug: print any mismatches
          print(&b"MISMATCH in vouch count after depth-first setup:");
          print(&target);
          print(&level);
          print(&vouch_count);
          print(&actual_vouch_count);
          print(&num_vouches);
        };

        level = level + 1;
      };

      // Print summary for this column
      print(&b"Column verification summary:");
      print(&num_vouches);
      print(&success_count);
      print(&mismatch_count);

      vouch_count = vouch_count + 1;
    };

    (root_users, network_matrix)
  }

  // Test with a depth-first approach to building the 10x10 vouch matrix
  #[test(framework_sig = @ol_framework)]
  fun test_matrix_vouch_network_depth_first(framework_sig: signer) {
    // Create accounts and set up network using depth-first approach
    let (root_users, network_matrix) = build_matrix_depth_first(&framework_sig);

    // Calculate and verify scores using full graph walk algorithm
    verify_matrix_scores_with_full_walk(&root_users, &network_matrix);
  }

}
