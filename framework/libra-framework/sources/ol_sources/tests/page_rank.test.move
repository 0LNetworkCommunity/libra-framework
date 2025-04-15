#[test_only]

// Tests for page_rank_lazy module with a large network of accounts.
// This simulates 100 accounts with various vouching relationships
// and tests the page rank score calculation.
module ol_framework::test_page_rank {
  use ol_framework::filo_migration;
  use ol_framework::root_of_trust;
  use ol_framework::mock;
  use ol_framework::vouch;
  use ol_framework::page_rank_lazy;
  use std::signer;
  use std::timestamp;
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
      filo_migration::maybe_migrate(sig);
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
    let root_sig = vector::borrow(&roots_sig, 0);

    vouch::init(root_sig);
    vouch::init(&new_user_sig);
    // // Initialize page rank for the new user
    page_rank_lazy::maybe_initialize_trust_record(&new_user_sig);

    // // Setup one vouch from the first root

    vouch::vouch_for(root_sig, new_user_addr);

    // // Now check the page rank score (should be 100)
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
    // // Initialize page rank for the new user
    page_rank_lazy::maybe_initialize_trust_record(&new_user_sig);

    // // Setup one vouch from the first root

    vouch::vouch_for(root_sig, new_user_addr);

    // Now check the page rank score (should be 50)
    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);
    assert!(page_rank_score == 50, 7357001);
    // Setup NINE vouches (from remaining roots)
    let i = 1; // start at SECOND root of trust
    while (i < vector::length(&roots_sig)) {
      let grantor = vector::borrow(&roots_sig, i);
      vouch::init(grantor);
      vouch::vouch_for(grantor, new_user_addr);

      i = i + 1;
    };

    // // Verify we have all 10 vouches
    let (received, _) = vouch::get_received_vouches(new_user_addr);
    assert!(vector::length(&received) == count_roots, 7357002);

    // // Now check the page rank score (should be 1000 = 10 roots * 100 points)
    // fast forward into the future
    let current_timestamp = 10000;
    timestamp::fast_forward_seconds(current_timestamp);
    let page_rank_score_later = page_rank_lazy::get_trust_score(new_user_addr);
    // NOTE: this should be 10X the previous test
    assert!(page_rank_score_later == 500, 7357003);
    assert!(page_rank_score_later == (count_roots * page_rank_score), 7357004);
  }
}
