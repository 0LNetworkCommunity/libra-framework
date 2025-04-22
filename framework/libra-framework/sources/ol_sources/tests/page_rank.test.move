#[test_only]
module ol_framework::test_page_rank {
  use ol_framework::activity;
  use ol_framework::founder;
  use ol_framework::root_of_trust;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::vouch;
  use ol_framework::vouch_txs;
  use ol_framework::page_rank_lazy;
  use std::signer;
  use std::vector;

  // sets up a network with 10 root of trust accounts (which are
  // not the validators). Returns a list of signers from the 10 roots.
  fun test_base(framework: &signer): vector<signer> {
    // do a test genesis with 3 validators
    mock::genesis_n_vals(framework, 3);
    mock::ol_initialize_coin_and_fund_vals(framework, 100, false);

    // create 10 root of trust accounts
    let roots_sig = mock::create_test_end_users(framework, 10, 0);
    let root_users = mock::collect_addresses(&roots_sig);

    vector::for_each_ref(&roots_sig, |sig| {
      // make each account a v8 address
      mock::simulate_transaction_validation(sig);
    });

    // make these accounts root of trust
    root_of_trust::framework_migration(framework, root_users, 1, 30);

    return roots_sig
  }

  // create a test base and check we have 10 root of trust accounts.
  #[test(framework = @ol_framework)]
  fun meta_check_base_setup(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);

    // Check that we have exactly 10 root accounts
    assert!(vector::length(&roots_sig) == 10, 1000);

    // Check that all accounts are registered as root of trust
    let i = 0;
    while (i < 10) {
      let addr = signer::address_of(vector::borrow(&roots_sig, i));
      // Pass both the registry address (@ol_framework) and the account being checked
      assert!(root_of_trust::is_root_at_registry(@ol_framework, addr), 1001);

      let (received, _) = vouch::get_received_vouches(addr);
      assert!(vector::length(&received) == 0, 1002);

      let page_rank_score = page_rank_lazy::get_trust_score(addr);

      assert!(page_rank_score == 0, 1003);


      i = i + 1;
    }
  }

  // Test with one user with one root vouch - both for vouch quality and page rank score
  #[test(framework = @ol_framework)]
  fun test_one_user_one_root(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let new_user_sig = mock::create_user_from_u64(framework, 11);
    let new_user_addr = signer::address_of(&new_user_sig);

    // the root sigs already have vouch initialized
    let root_sig = vector::borrow(&roots_sig, 0);

    vouch::init(&new_user_sig);
    // Initialize page rank for the new user
    page_rank_lazy::maybe_initialize_trust_record(&new_user_sig);

    // Setup one vouch from the first root
    vouch::vouch_for(root_sig, new_user_addr);

    // Now check the page rank score (should be 100)
    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);

    assert!(page_rank_score == 50, 7357001);
  }

  // Test with one user with ten root vouches - both for vouch quality and page rank score
  #[test(framework = @ol_framework)]
  fun test_one_user_ten_root(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let count_roots = vector::length(&roots_sig);
    let new_user_sig = mock::create_user_from_u64(framework, 11);
    let new_user_addr = signer::address_of(&new_user_sig);
    let root_sig = vector::borrow(&roots_sig, 0);

    vouch::init(root_sig);
    vouch::init(&new_user_sig);
    // Initialize page rank for the new user
    page_rank_lazy::maybe_initialize_trust_record(&new_user_sig);

    let i = 0;
    while (i < vector::length(&roots_sig)) {
      let grantor = vector::borrow(&roots_sig, i);
      vouch::init(grantor);
      vouch::vouch_for(grantor, new_user_addr);

      i = i + 1;
    };
        // // Verify we have all 10 vouches
    let (received, _) = vouch::get_received_vouches(new_user_addr);
    assert!(vector::length(&received) == count_roots, 7357002);

    let page_rank_score_later = page_rank_lazy::get_trust_score(new_user_addr);
    // NOTE: this should be 10X the previous test
    assert!(page_rank_score_later == 500, 7357003);
  }

  // creates a user that is one hop away from the user with 10 vouches from root of trust.
  #[test(framework = @ol_framework)]
  fun test_one_hop_to_one_user_ten_root(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let count_roots = vector::length(&roots_sig);

    let new_user_sig = mock::create_user_from_u64(framework, 11);
    let new_user_addr = signer::address_of(&new_user_sig);
    let root_sig = vector::borrow(&roots_sig, 0);

    vouch::init(root_sig);
    vouch::init(&new_user_sig);
    // // Initialize page rank for the new user
    page_rank_lazy::maybe_initialize_trust_record(&new_user_sig);

    let i = 0;
    while (i < vector::length(&roots_sig)) {
      let grantor = vector::borrow(&roots_sig, i);
      vouch::init(grantor);
      vouch::vouch_for(grantor, new_user_addr);

      i = i + 1;
    };

    // // Verify we have all 10 vouches
    let (received, _) = vouch::get_received_vouches(new_user_addr);
    assert!(vector::length(&received) == count_roots, 7357002);

    let page_rank_score_later = page_rank_lazy::get_trust_score(new_user_addr);
    // NOTE: this should be 10X the previous test
    assert!(page_rank_score_later == 500, 7357003);

    //// THE TEST
    // This user is friends with new_user above, which has 10 vouches from root
    let another_user_sig = mock::create_user_from_u64(framework, 12);
    let another_user_addr = signer::address_of(&another_user_sig);
    vouch::init(&another_user_sig);
    page_rank_lazy::maybe_initialize_trust_record(&another_user_sig);
    let score_pre = page_rank_lazy::get_trust_score(another_user_addr);
    assert!(score_pre == 0, 7357004);

    vouch_txs::vouch_for(&new_user_sig, another_user_addr);
    let score_post = page_rank_lazy::get_trust_score(another_user_addr);
    assert!(score_post == 250, 7357005);
    assert!(score_post == page_rank_score_later/2, 7357006);
  }


  #[test(framework = @ol_framework)]
  /// tests the case that all the root users are reciprocally
  /// vouching for one another.
  fun test_root_reciprocal_vouch(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let count_roots = vector::length(&roots_sig);

    // First, have all root accounts initialize page rank
    let i = 0;
    while (i < count_roots) {
      let root_sig = vector::borrow(&roots_sig, i);
      page_rank_lazy::maybe_initialize_trust_record(root_sig);
      i = i + 1;
    };

    // Make each root vouch for all other roots (reciprocal vouching)
    let i = 0;
    while (i < count_roots) {
      let grantor = vector::borrow(&roots_sig, i);
      let _grantor_addr = signer::address_of(grantor);

      let j = 0;
      while (j < count_roots) {
        if (i != j) { // Don't vouch for yourself
          let beneficiary = vector::borrow(&roots_sig, j);
          let beneficiary_addr = signer::address_of(beneficiary);

          vouch::vouch_for(grantor, beneficiary_addr);
        };
        j = j + 1;
      };

      i = i + 1;
    };

    // Verify each root has received vouches from all other roots
    let i = 0;
    while (i < count_roots) {
      let root_addr = signer::address_of(vector::borrow(&roots_sig, i));

      let (received, _) = vouch::get_received_vouches(root_addr);
      assert!(vector::length(&received) == 9, 7357009);

      // Each root should receive vouches from all other roots (count_roots - 1)
      assert!(vector::length(&received) == count_roots - 1, 7357010);

      // Check page rank score is properly calculated
      // Each root should have a high score since they're all vouched for by other roots
      let page_rank_score = page_rank_lazy::get_trust_score(root_addr);
      // With 9 other roots vouching, score should be 9 * 50 = 450

      assert!(page_rank_score == 450, 7357011);

      i = i + 1;
    };
  }



  #[test(framework = @ol_framework)]
  fun test_stale_trust(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let new_user_sig = mock::create_user_from_u64(framework, 11);
    let new_user_addr = signer::address_of(&new_user_sig);
    let root_sig = vector::borrow(&roots_sig, 0);

    vouch::init(root_sig);
    vouch::init(&new_user_sig);
    // Initialize page rank for the new user
    page_rank_lazy::maybe_initialize_trust_record(&new_user_sig);

    // Setup one vouch from the first root

    vouch::vouch_for(root_sig, new_user_addr);

    // Now check the page rank score (should be 100)
    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);
    assert!(page_rank_score == 50, 7357001);
    let stale = page_rank_lazy::is_stale(new_user_addr);
    assert!(!stale, 7357002);

    ///// THE TEST
    // should mark stale
    vouch_txs::revoke(root_sig, new_user_addr);
    ///////////////

    let stale = page_rank_lazy::is_stale(new_user_addr);
    assert!(stale, 7357003);

    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);
    assert!(page_rank_score == 0, 7357004);

    ///// THE TEST
    // should mark stale
    vouch_txs::vouch_for(root_sig, new_user_addr);
    ///////////////

    // will be stale until next computation
    let stale = page_rank_lazy::is_stale(new_user_addr);
    assert!(stale, 7357005);

    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);
    assert!(page_rank_score == 50, 7357006);

    // should no longer be stale
    let stale = page_rank_lazy::is_stale(new_user_addr);
    assert!(!stale, 7357005);
  }


  // check that the page rank can be used to
  // reauthorize a founder
  #[test(framework = @ol_framework)]
  fun test_founder_reauth(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let one_root_sig = vector::borrow(&roots_sig, 0);

    let seven_user_sig = ol_account::test_emulate_v7_account(framework, @0xabcd1234);
    let user_addr = signer::address_of(&seven_user_sig);

    // a v7 user touches the account to get structs created
    mock::simulate_transaction_validation(&seven_user_sig);
    // check lazy migration worked
    let is_init = activity::is_initialized(user_addr);
    let pre = activity::is_prehistoric(user_addr);
    let is_founder = founder::is_founder(user_addr);
    assert!(is_init, 7357001);
    assert!(pre, 7357002);
    assert!(is_founder, 7357003);

    // page rank score should be 0
    let page_rank_score = page_rank_lazy::get_trust_score(user_addr);
    assert!(page_rank_score == 0, 7357003);
    // check that sybil resistance did not yet pass
    let has_friends = founder::has_friends(user_addr);
    assert!(!has_friends, 7357004);

    vouch_txs::vouch_for(one_root_sig, user_addr);
    let page_rank_score = page_rank_lazy::get_trust_score(user_addr);
    assert!(page_rank_score == 50, 7357005);

    let has_friends = founder::has_friends(user_addr);
    assert!(has_friends, 7357006);
  }

  #[test(framework = @ol_framework)]
  /// Test to verify that users don't gain trust score from roots vouching for each other
  /// This test demonstrates that the root check in page_rank_lazy::walk_from_node prevents
  /// score accumulation through root-to-root vouches
  fun test_no_accumulation_from_root_vouches(framework: &signer) {
    // Set up the test base with 10 root accounts
    let roots_sig = test_base(framework);
    let count_roots = vector::length(&roots_sig);

    // Create two users - one will be directly vouched by a root,
    // the other will be vouched by multiple roots that vouch for each other
    let user1_sig = mock::create_user_from_u64(framework, 11);
    let user2_sig = mock::create_user_from_u64(framework, 12);
    let user1_addr = signer::address_of(&user1_sig);
    let _user2_addr = signer::address_of(&user2_sig);

    // Initialize both users
    vouch::init(&user1_sig);
    vouch::init(&user2_sig);
    page_rank_lazy::maybe_initialize_trust_record(&user1_sig);
    page_rank_lazy::maybe_initialize_trust_record(&user2_sig);


    // Make each root vouch for all other roots (reciprocal vouching)
    let i = 0;
    while (i < count_roots) {
      let grantor = vector::borrow(&roots_sig, i);
      let j = 0;
      while (j < count_roots) {
        if (i != j) { // Don't vouch for yourself
          let beneficiary = vector::borrow(&roots_sig, j);
          let beneficiary_addr = signer::address_of(beneficiary);
          vouch::vouch_for(grantor, beneficiary_addr);
        };
        j = j + 1;
      };
      i = i + 1;
    };


    // First hop: the root vouches for user1 (at first hop)
    let root0 = vector::borrow(&roots_sig, 0);
    vouch::vouch_for(root0, user1_addr);

    let root0_score = page_rank_lazy::get_trust_score(signer::address_of(root0));
    assert!(root0_score == 450, 7357100);

    // Check user1's score - should be 50 (100/2 from a direct root vouch)
    let user1_score = page_rank_lazy::get_trust_score(user1_addr);
    assert!(user1_score == 50, 7357100);
    assert!(user1_score < root0_score, 7357101);

    // // Now have multiple roots (who vouch for each other) vouch for user2
    // vouch::vouch_for(vector::borrow(&roots_sig, 0), user2_addr);
    // vouch::vouch_for(vector::borrow(&roots_sig, 1), user2_addr);

    // // Get user2's score - should only count direct vouches from roots (2 * 50 = 100)
    // // If root-to-root vouches were counted, the score would be much higher
    // let user2_score = page_rank_lazy::get_trust_score(user2_addr);
    // assert!(user2_score == 100, 7357101);

    // // Additional test: Have a non-root user who is vouched by a root vouch for user2
    // // This should add additional score to user2 (indirect vouch from root)
    // vouch::vouch_for(&user1_sig, user2_addr);

    // // User2's score should now include the indirect vouch from user1
    // // Direct: 2 roots * 50 = 100
    // // Indirect: user1's contribution = 50/2 = 25
    // // Total: 125
    // let user2_score_after = page_rank_lazy::get_trust_score(user2_addr);
    // assert!(user2_score_after == 125, 7357102);
  }

    #[test(framework = @ol_framework)]
  /// Tests the diminishing power as the number of hops increases
  fun test_diminishing_power(framework: &signer) {
    // Set up the test base with 10 root accounts
    let roots_sig = test_base(framework);

    // create 10 root of trust accounts
    let users_sig = mock::create_test_end_users(framework, 10, 11);
    let users_addr = mock::collect_addresses(&users_sig);

    // First hop: the root vouches for user1 (at first hop)
    let grantor_sig = vector::borrow(&roots_sig, 0);
    let root0_score = page_rank_lazy::get_trust_score(signer::address_of(grantor_sig));
    // in a simple initialization root doesn't have any vouches themselves
    assert!(root0_score == 0, 7357100);


    let prev_score = 100;
    // loop through the users and have the previous
    // user vouch for the next user
    let i = 0;
    while (i < vector::length(&users_sig)) {
      let beneficiary_addr = vector::borrow(&users_addr, i);
      let beneficiary_sig = vector::borrow(&users_sig, i);

      vouch::init(beneficiary_sig);
      page_rank_lazy::maybe_initialize_trust_record(beneficiary_sig);
      vouch::vouch_for(grantor_sig, *beneficiary_addr);

      let user1_score = page_rank_lazy::get_trust_score(*beneficiary_addr);
      // in a simple initialization root doesn't have any vouches themselves
      if (prev_score > 0) {
        assert!(user1_score < prev_score, 7357101);
        if (user1_score > 0) {
          assert!(user1_score == prev_score/2, 7357102);
        };
      };

      prev_score = user1_score;

      grantor_sig = vector::borrow(&users_sig, i);

      i = i + 1;
    };
  }
}
