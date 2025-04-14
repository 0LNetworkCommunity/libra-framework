#[test_only]

// Tests for page_rank_lazy module with a large network of accounts.
// This simulates 100 accounts with various vouching relationships
// and tests the page rank score calculation.
module ol_framework::test_page_rank {
  // use diem_framework::account;

  use ol_framework::root_of_trust;
  use ol_framework::mock;
  use ol_framework::vouch;
  use ol_framework::page_rank_lazy;
  use std::signer;
  use std::timestamp;
  use std::vector;
  // use std::string::utf8;

  use diem_std::debug::print;

  // sets up a network with 10 root of trust accounts (which are
  // not the validators). Returns a list of signers from the 10 roots.
  fun test_base(framework: &signer): vector<signer> {
    // do a test genesis with 3 validators
    mock::genesis_n_vals(framework, 3);
    mock::ol_initialize_coin_and_fund_vals(framework, 100, false);

    // create 10 root of trust accounts
    let roots_sig = mock::create_test_end_users(framework, 10, 0);
    // get a vec of addresses from roots_sig
    let root_users = mock::collect_addresses(&roots_sig);

    // make these accounts root of trust
    root_of_trust::framework_migration(framework, root_users, 1, 30);

    return roots_sig
  }

  // create a test base and check we have 10 root of trust accounts.
  #[test(framework = @ol_framework)]
  fun c(framework: &signer) {
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
    page_rank_lazy::initialize_user_trust_record(&new_user_sig);

    // // Setup one vouch from the first root

    vouch::vouch_for(root_sig, new_user_addr);

    // // Now check the page rank score (should be 100)
    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);
    print(&page_rank_score);
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
    page_rank_lazy::initialize_user_trust_record(&new_user_sig);

    // // Setup one vouch from the first root

    vouch::vouch_for(root_sig, new_user_addr);

    // Now check the page rank score (should be 50)
    let page_rank_score = page_rank_lazy::get_trust_score(new_user_addr);
    print(&page_rank_score);
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

    // // Check vouch quality score (will always be 100 with vouch_metrics)
    // let vouch_quality_score = vouch_metrics::calculate_total_vouch_quality(new_user_addr);
    // print(&vouch_quality_score);

    // // Now check the page rank score (should be 1000 = 10 roots * 100 points)
    // fast forward into the future
    let current_timestamp = 10000;
    timestamp::fast_forward_seconds(current_timestamp);
    let page_rank_score_later = page_rank_lazy::get_trust_score(new_user_addr);
    print(&page_rank_score_later);
    // NOTE: this should be 10X the previous test
    assert!(page_rank_score_later == 500, 7357003);
    assert!(page_rank_score_later == (count_roots * page_rank_score), 7357004);
  }

  // // starts a test base, and creates an additional 10 user accounts
  // // then each of root of trust accounts will issue vouches
  // // The first root, will vouch only for the first user account.
  // // the second root, will vouch for both the first and second user accounts.
  // // returns the new users created
  // fun add_users_and_vouches_matrix(framework: &signer, root_sigs: &vector<signer>, vouch_column: u64, distance_row: u64, user_id_idx: u64): vector<signer> {
  //   // need to create signer types outside of the loop below
  //   let new_users_sig = mock::create_test_end_users(framework,(vouch_column * distance_row), user_id_idx);
  //   let users_len = vector::length(&new_users_sig);
  //   print(&utf8(b"users_len"));
  //   print(&users_len);

  //   // we will mock these users depth-first
  //   let v = 0;
  //   // start with users that will receive 1 vouch
  //   while (v < vouch_column) {

  //     print(&11111);
  //     print(&utf8(b"root"));

  //     // the first grantor in this column is a root of trust account
  //     let grantor = vector::borrow(root_sigs, v);

  //     // populates all remaining accounts after a column is
  //     // completed, such that the last column accumulates all vouches.
  //     let user_idx = v*distance_row;
  //     print(&signer::address_of(grantor));

  //     print(&utf8(b"start index:"));
  //     print(&user_idx);


  //     while (user_idx < users_len) {
  //       print(&22222);
  //             print(&utf8(b"grantor:"));
  //             print(&signer::address_of(grantor));


  //       let recipient_sig = vector::borrow(&new_users_sig, user_idx);

  //       let recipient_addr = signer::address_of(recipient_sig);
  //       print(&utf8(b"recipient:"));

  //       print(&recipient_addr);
  //       vouch::init(grantor);
  //       vouch::init(recipient_sig);

  //       vouch::vouch_for(grantor, recipient_addr);

  //       let (v_list, _) = vouch::get_received_vouches(recipient_addr);
  //       print(&v_list);
  //       // users should have the number of vouches in this column
  //       assert!(vector::length(&v_list) == v+1, 1002);
  //       // then next level down, the grantor will be the current recipient
  //       grantor = recipient_sig;
  //       user_idx = user_idx + 1;
  //     };
  //     v = v + 1;
  //   };

  //   new_users_sig
  // }

  // // test we can populate vouch columns vertically with one column
  // #[test(framework = @ol_framework)]
  // fun test_populate_one_column(framework: &signer) {
  //   // Set up the test base
  //   let roots_sig = test_base(framework);
  //   let next_idx = vector::length(&roots_sig);
  //   let columns = 1;
  //   let rows = 3;
  //   let new_users = add_users_and_vouches_matrix(framework, &roots_sig, columns, rows, next_idx);
  //   assert!(vector::length(&new_users) == 3, 7357001);

  // }
}
