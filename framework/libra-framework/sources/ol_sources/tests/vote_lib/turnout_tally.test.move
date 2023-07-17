#[test_only]

module ol_framework::test_turnout_tally {
  use ol_framework::turnout_tally;
  use ol_framework::turnout_tally_demo;
  use ol_framework::mock;
  use ol_framework::vote_receipt;
  use std::option;

    #[test]
    fun turnout_threshold() {

      // confirm upperbound
      let y = turnout_tally::get_threshold_from_turnout(8750, 10000);
      assert!(y == 5100, 0);

      // confirm lowerbound
      let y = turnout_tally::get_threshold_from_turnout(1250, 10000);
      assert!(y == 10000, 0);

      let y = turnout_tally::get_threshold_from_turnout(1500, 10000);
      assert!(y == 9837, 0);

      let y = turnout_tally::get_threshold_from_turnout(5000, 10000);
      assert!(y == 7550, 0);

      let y = turnout_tally::get_threshold_from_turnout(7500, 10000);
      assert!(y == 5917, 0);

      // cannot be below the minimum treshold. I.e. more than 100%
      let y = turnout_tally::get_threshold_from_turnout(500, 10000);
      assert!(y == 10000, 0);

      // same for maximum. More votes cannot go below 51% approval
      let y = turnout_tally::get_threshold_from_turnout(9000, 10000);
      assert!(y == 5100, 0);
  }


    #[test(root = @ol_framework, alice = @0x1000a)]
    fun tally_init_happy(root: &signer, alice: &signer) {
      let _vals = mock::genesis_n_vals(root, 1);
      turnout_tally_demo::init(alice);
      // ZERO HERE MEANS IT NEVER EXPIRES
      let _uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 0);
  }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun tally_vote_happy(root: &signer, alice: &signer, bob: &signer) {
      let _vals = mock::genesis_n_vals(root, 1);
      turnout_tally_demo::init(alice);
      // ZERO HERE MEANS IT NEVER EXPIRES
      let uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 0);

      turnout_tally_demo::vote(bob, @0x1000a, &uid, 22, true);
       let (r, w) = vote_receipt::get_receipt_data(@0x1000b, &uid);
      assert!(r == true, 0); // voted in favor
      assert!(w == 22, 1);
  }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun tally_vote_retract(root: &signer, alice: &signer, bob: &signer) {
      let _vals = mock::genesis_n_vals(root, 1);
      turnout_tally_demo::init(alice);
      // ZERO HERE MEANS IT NEVER EXPIRES
      let uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 0);

      turnout_tally_demo::vote(bob, @0x1000a, &uid, 22, true);
       let (r, w) = vote_receipt::get_receipt_data(@0x1000b, &uid);
      assert!(r == true, 0); // voted in favor
      assert!(w == 22, 1);

      turnout_tally_demo::retract(bob, &uid, @0x1000a);
      let (r, _) = vote_receipt::get_receipt_data(@0x1000b, &uid);
      assert!(r == false, 0); // voted in favor
  }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun tally_vote_incomplete(root: &signer, alice: &signer, bob: &signer) {
      let _vals = mock::genesis_n_vals(root, 1);
      turnout_tally_demo::init(alice);
      // ZERO HERE MEANS IT NEVER EXPIRES
      let uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 0);

      // lower vote
      let result_opt = turnout_tally_demo::vote(bob, @0x1000a, &uid, 5, true);
      let (r, w) = vote_receipt::get_receipt_data(@0x1000b, &uid);
      assert!(r == true, 0); // voted in favor
      assert!(w == 5, 1);

      // 5 of 100 is not enough to get over any dynamic threshold.

      assert!(option::is_none(&result_opt), 2);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    #[expected_failure(abort_code = 496618, location = 0x1::turnout_tally)]

    fun tally_vote_expired(root: &signer, alice: &signer, bob: &signer) {
      let _vals = mock::genesis_n_vals(root, 1);
      mock::ol_initialize_coin(root);
      // for this test let's start at epoch 1
      mock::trigger_epoch(root);

      turnout_tally_demo::init(alice);
      let uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 1);

      mock::trigger_epoch(root);
      // we are now in epoch 2 and the vote should have expired in 1
      let _result_opt = turnout_tally_demo::vote(bob, @0x1000a, &uid, 5, true);
      // let (r, w) = vote_receipt::get_receipt_data(@0x1000b, &uid);
      // assert!(r == true, 0); // voted in favor
      // assert!(w == 5, 1);

      // // 5 of 100 is not enough to get over any dynamic threshold.

      // assert!(option::is_none(&result_opt), 2);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun tally_vote_closed_early(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Alice is going to start an election, and create the struct on her account.
      // The election will run until for another 10 epochs
      // The election will close before then once threshold is reached.

      let _vals = mock::genesis_n_vals(root, 1);
      mock::ol_initialize_coin(root);
      // for this test let's start at epoch 1
      mock::trigger_epoch(root);

      turnout_tally_demo::init(alice);
      // ZERO HERE MEANS IT NEVER EXPIRES
      // but we are testing for it to close early
      let uid = turnout_tally_demo::propose_ballot_by_owner(alice, 100, 0);

      // lower vote
      let result_opt = turnout_tally_demo::vote(bob, @0x1000a, &uid, 22, true);
      let (r, w) = vote_receipt::get_receipt_data(@0x1000b, &uid);
      assert!(r == true, 0); // voted in favor
      assert!(w == 22, 1);

      // 22 is enough to pass. HOEVER, the Vote does not close and pass immediately. It requires one more vote in favor AT LEAST ONE day later.

      // 5 of 100 is not enough to get over any dynamic threshold.

      assert!(option::is_none(&result_opt), 2);

      mock::trigger_epoch(root);

      // NOW carol votes
      let result_opt = turnout_tally_demo::vote(carol, @0x1000a, &uid, 15, true);
      let (r, w) = vote_receipt::get_receipt_data(@0x1000c, &uid);
      assert!(r == true, 3); // voted in favor
      assert!(w == 15, 4);
      assert!(option::is_some(&result_opt), 5);
  }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun tally_never_expires(root: &signer, alice: &signer, bob: &signer) {
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