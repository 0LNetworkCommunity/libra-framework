#[test_only]

module ol_framework::test_turnout_tally {
  use ol_framework::turnout_tally;
  use ol_framework::turnout_tally_demo;
  use ol_framework::mock;
    #[test]
    fun test_threshold() {

      // confirm upperbound
      let y = turnout_tally::get_threshold_from_turnout(8750, 10000);
      assert!(y == 5100, 0);

      // confirm lowerbound
      let y = turnout_tally::get_threshold_from_turnout(1250, 10000);
      // print(&y);
      assert!(y == 10000, 0);

      let y = turnout_tally::get_threshold_from_turnout(1500, 10000);
      // print(&y);
      assert!(y == 9837, 0);

      let y = turnout_tally::get_threshold_from_turnout(5000, 10000);
      // print(&y);
      assert!(y == 7550, 0);

      let y = turnout_tally::get_threshold_from_turnout(7500, 10000);
      // print(&y);
      assert!(y == 5917, 0);

      // cannot be below the minimum treshold. I.e. more than 100%
      let y = turnout_tally::get_threshold_from_turnout(500, 10000);
      // print(&y);
      assert!(y == 10000, 0);

      // same for maximum. More votes cannot go below 51% approval
      let y = turnout_tally::get_threshold_from_turnout(9000, 10000);
      // print(&y);
      assert!(y == 5100, 0);
  }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun test_tally_never_expires(root: &signer, alice: &signer, bob: &signer) {
      let _vals = mock::genesis_n_vals(root, 1);
      mock::ol_initialize_coin(root);

      turnout_tally_demo::init(alice);

      // ZERO HERE MEANS IT NEVER EXPIRES
      let uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 0);

      // many epochs later
      mock::trigger_epoch(root);
      mock::trigger_epoch(root);
      mock::trigger_epoch(root);

      // Bob votes, and the epoch should be 2 now, and the vote expired at end of 1.
      turnout_tally_demo::vote(bob, @0x1000a, &uid, 22, true);

  }
}