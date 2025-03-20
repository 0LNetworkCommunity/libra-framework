module ol_framework::vouch_score {
  use std::option;
  use std::vector;
  use diem_std::fixed_point32;
  use ol_framework::ancestry;
  use ol_framework::vouch;

  /// the threshold score for a user to be considered vouched
  // const THRESHOLD_SCORE: u64 = 2;

  /// get a users's distance score from an arbitrary voucher
  /// score is percent, out of 100
  fun calculate_single_score(root: address, user: address): u64 {
    let maybe_degree = ancestry::get_degree(root, user);
    if (option::is_none(&maybe_degree)) {
        0
    } else {
        let degree = *option::borrow(&maybe_degree);
        if (degree == 0) {
            0
        } else {
            diem_std::debug::print(&degree);

            // Create fixed point representation of 100
            let score = fixed_point32::create_from_rational(100, degree);
            fixed_point32::floor(score)
        }
    }
  }

  /// Get the total score for a user from any list of roots
  /// This is now public to allow root_of_trust to use it
  public fun get_total_score_from_list_of_root(roots: vector<address>, user: address): u64 {
    let total_score = 0;
    let i = 0;
    while (i < vector::length(&roots)) {
      let root = vector::borrow(&roots, i);
      let score = calculate_single_score(*root, user);
      total_score = total_score + score;
      i = i + 1;
    };

    total_score
  }

  #[view]
  /// Evaluate score against a specific registry's roots
  public fun evaluate_score_for_registry(registry_roots: vector<address>, user: address): u64 {
    get_total_score_from_list_of_root(registry_roots, user)
  }

  #[view]
  /// evaluate the score of the cohort vouching for this
  /// user
  public fun evaluate_users_vouchers(roots: vector<address>, user: address): u64 {
    // we only want the vouchers which are not expired
    // and do not belong to the same family
    let valid_vouchers = vouch::true_friends(user);
    let total_score = 0;
    let i = 0;
    while (i < vector::length(&valid_vouchers)) {
      let one_voucher = vector::borrow(&valid_vouchers, i);
      let score = get_total_score_from_list_of_root(roots, *one_voucher);
      total_score = total_score + score;
      i = i + 1;
    };

    total_score
  }


  // #[view]

  // /// counting all vouchers and their score
  // /// does it add up to 2 root of trust accounts?
  // /// @return true if the user is vouched
  // public fun is_voucher_score_valid(user: address): bool {
  //   get_total_vouch_score(user) > THRESHOLD_SCORE
  // }

  // #[test_only]
  // use diem_framework::genesis;
  // #[test_only]
  // use std::signer;

  // #[test(account = @0x1)]
  // fun test_registry_specific_scoring(account: signer) {
  //     // Initialize test environment
  //     genesis::setup();

  //     // Set up test accounts and create signers
  //     let registry = @0x123;
  //     let user = @0x456;
  //     let root1 = @0x789;
  //     let root2 = @0x987;

  //     // Create signers for accounts that need initialization
  //     let user_signer = genesis::create_account_for_test(user);
  //     let root1_signer = genesis::create_account_for_test(root1);
  //     let root2_signer = genesis::create_account_for_test(root2);

  //     // Set up ancestry relationships
  //     // root1 -> user (direct relationship)
  //     ancestry::set_tree(&user_signer, root1);

  //     let roots = vector::empty();
  //     vector::push_back(&mut roots, root1);
  //     vector::push_back(&mut roots, root2);

  //     // Test scoring - now with established ancestry
  //     let score = get_total_score_from_list_of_root(roots, user);
  //     assert!(score == 100, 1); // Direct relationship should score 100

  //     // Test evaluation against threshold
  //     let is_valid = evaluate_score_for_registry(roots, user);
  //     assert!(is_valid == (score >= THRESHOLD_SCORE), 2);
  // }
}
