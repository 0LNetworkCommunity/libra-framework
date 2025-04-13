#[test_only]

// Tests for page_rank_lazy module with a large network of accounts.
// This simulates 100 accounts with various vouching relationships
// and tests the page rank score calculation.
module ol_framework::test_page_rank {
  // use diem_framework::account;
  use ol_framework::root_of_trust;
  use ol_framework::mock;
  use ol_framework::vouch;
  use std::signer;
  use std::vector;

  use diem_std::debug::print;

  // sets up a network with 10 root of trust accounts (which are
  // not the validators). Returns a list of signers from the 10 roots.
  fun test_base(framework: &signer): vector<signer> {
    // do a test genesis with 3 validators
    mock::genesis_n_vals(framework, 3);
    mock::ol_initialize_coin_and_fund_vals(framework, 100, false);

    // create 10 root of trust accounts
    let roots_sig = mock::create_test_signers(framework, 10, 0);
    // get a vec of addresses from roots_sig
    let root_users = mock::collect_addresses(&roots_sig);

    // make these accounts root of trust
    root_of_trust::framework_migration(framework, root_users, 1, 30);

    return roots_sig
  }

  // create a test base and check we have 10 root of trust accounts.
  #[test(framework = @ol_framework)]
  fun test_base_creates_ten_root_accounts(framework: &signer) {
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

  // starts a test base, and creates an additional 10 user accounts
  // then each of root of trust accounts will issue vouches
  // The first root, will vouch only for the first user account.
  // the second root, will vouch for both the first and second user accounts.
  fun add_users_and_vouches(framework: &signer, root_sigs: &vector<signer>, vouch_column: u64, distance_row: u64, user_id_idx: u64) {
    let new_users_sig = mock::create_test_signers(framework,(vouch_column * distance_row), user_id_idx);
    let users_len = vector::length(&new_users_sig);
    print(&users_len);

    // we will mock these users depth-first
    let v = 0;
    // start with users that will receive 1 vouch
    while (v < vouch_column) {
      print(&11111);
      // the first grantor in this column is a root of trust account
      let grantor = vector::borrow(root_sigs, v);

      // populates all remaining accounts after a column is
      // completed, such that the last column accumulates all vouches.
      let user_idx = v*distance_row;
      print(&user_idx);
      print(&signer::address_of(grantor));

      while (user_idx < users_len) {
        print(&22222);

        let recipient_sig = vector::borrow(&new_users_sig, user_idx);

        let recipient_addr = signer::address_of(recipient_sig);
        print(&recipient_addr);
        vouch::init(grantor);
        vouch::init(recipient_sig);

        vouch::test_helper_vouch_for(grantor, recipient_addr);

        let (v_list, _) = vouch::get_received_vouches(recipient_addr);
        print(&v_list);
        // users should have the number of vouches in this column
        assert!(vector::length(&v_list) == v+1, 1002);
        // then next level down, the grantor will be the current recipient
        grantor = recipient_sig;
        user_idx = user_idx + 1;
      };
      v = v + 1;
    }
  }

  // test we can populate vouch columns vertically with one column
  #[test(framework = @ol_framework)]
  fun test_populate_one_column(framework: &signer) {
    // Set up the test base
    let roots_sig = test_base(framework);
    let next_idx = vector::length(&roots_sig);
    add_users_and_vouches(framework, &roots_sig, 2, 3, next_idx);

  }
}
