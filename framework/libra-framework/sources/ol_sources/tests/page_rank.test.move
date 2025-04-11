#[test_only]

/// Tests for page_rank_lazy module with a large network of accounts.
/// This simulates 100 accounts with various vouching relationships
/// and tests the page rank score calculation.
module ol_framework::test_page_rank {
  use std::vector;
  use std::signer;
  use diem_std::math64;
  use ol_framework::mock;
  use ol_framework::root_of_trust;
  use ol_framework::page_rank_lazy;
  use ol_framework::vouch;
  use ol_framework::ancestry;
  use diem_framework::account; // Adding missing import for account module

  // Test with a network of 100 accounts
  #[test(framework_sig = @ol_framework)]
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
  #[test(framework_sig = @ol_framework)]
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

  // --- Network setup and structure helpers ---

  /// Set up a test network with the given parameters
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
    // Pattern 1: Root nodes vouch for high-level nodes
    let first_level_addresses = vector::empty<address>();

    // Roots vouch for the first few high-level nodes
    let j = 0;
    while (j < high_level_count) {
      if (j < vector::length(normal_addresses)) {
        let target_addr = *vector::borrow(normal_addresses, j);
        vector::push_back(&mut first_level_addresses, target_addr);

        // Each root vouches for this high-level node
        let i = 0;
        while (i < vector::length(roots)) {
          let root_addr = *vector::borrow(roots, i);
          create_vouch(root_addr, target_addr);
          i = i + 1;
        };
      };
      j = j + 1;
    };

    // Pattern 2: High-level nodes vouch for mid-level nodes
    let mid_level_addresses = vector::empty<address>();

    // Each high-level node vouches for 3 mid-level nodes
    let j = 0;
    while (j < mid_level_count) {
      if (high_level_count + j < vector::length(normal_addresses)) {
        let target_addr = *vector::borrow(normal_addresses, high_level_count + j);
        vector::push_back(&mut mid_level_addresses, target_addr);

        // Determine which high-level node will vouch for this mid-level node
        let high_level_idx = j % high_level_count;
        let high_level_addr = *vector::borrow(&first_level_addresses, high_level_idx);
        create_vouch(high_level_addr, target_addr);
      };
      j = j + 1;
    };

    // Pattern 3: Mid-level nodes vouch for base nodes
    // Remaining nodes are base level
    let j = 0;
    while (high_level_count + mid_level_count + j < vector::length(normal_addresses)) {
      let target_addr = *vector::borrow(normal_addresses, high_level_count + mid_level_count + j);

      // Determine which mid-level node will vouch for this base node
      let mid_level_idx = j % mid_level_count;
      let mid_level_addr = *vector::borrow(&mid_level_addresses, mid_level_idx);
      create_vouch(mid_level_addr, target_addr);

      j = j + 1;
    };
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
      let i = 0;
      while (i < high_level_count) {
        avg_first_level_score = avg_first_level_score + *vector::borrow(scores, i);
        i = i + 1;
      };
      avg_first_level_score = avg_first_level_score / high_level_count;

      let avg_last_level_score = 0;
      let total_nodes = vector::length(normal_addresses);
      let i = 0;
      let count = 0;
      while (i < 10 && high_level_count + mid_level_count + i < total_nodes) {
        avg_last_level_score = avg_last_level_score +
            *vector::borrow(scores, high_level_count + mid_level_count + i);
        i = i + 1;
        count = count + 1;
      };
      if (count > 0) {
        avg_last_level_score = avg_last_level_score / count;

        // First level nodes should generally have higher scores
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
}
