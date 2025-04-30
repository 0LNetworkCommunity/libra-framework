///////////////////////////////////////////////////////////////////////////
// 0L Module
// TurnoutTally
// Binary voting with threshold adjusted for turnout
///////////////////////////////////////////////////////////////////////////


  module ol_framework::turnout_tally {

    // TurnoutTally is a module that constructs a TallyType, for use with VoteLib's Ballot module. It does not store any state itself, you'll need to import it into a module.

    // The tally design single issue referendum, with only votes in favor or against, but with an adaptive threshold for the vote to pass.

    // The design of the policies attempts to accommodate online voting where votes tend to happen:
    // 1. publicly
    // 2. asynchronously
    // 3. with a low turnout
    // 4. of voters with high conviction

    // Low turnouts are not indicative of disinterest (often termed voter apathy), but instead it is just "rational ignorance" of the matters being voted. Adaptive deadlines and thresholds are an attempt to encourage more familiarity with the matters, and thus more participation. At the same time it allows for consensus decisions to be reached in a timely manner by the active and interested participants.

    // The ballots have dynamic deadlines and thresholds.
    // deadlines can be extended by dissenting votes from the current majority vote. This cannot be done indefinitely, as the extension is capped by the max_deadline.
    // The threshold (percent approval/rejection which needs to be met) is determined post-hoc. Referenda are expensive, for communities, leaders, and voters. Instead of scrapping a vote which doesn't achieve a pre-established number of participants (quorum), the vote is allowed to be tallied and be seen as valid. However, the threshold must be higher (more than 51%) in cases where the turnout is low. In blockchain polkadot network has prior art for this: https://wiki.polkadot.network/docs/en/learn-governance#adaptive-quorum-biasing
    // In Polkadot's implementation there are positive and negative biasing. Negative biasing (when turnout is low, a lower amount of votes is needed) seems to be an edge case.

    // Regarding deadlines. The problem with adaptive thresholds is that it favors the engaged community. If you are unaware of the process, or if the process occurred silently, it's very challenging to swing the vote. So the minority vote may be disadvantaged due to lack of engagement, and there should be some accommodation. If they are coming late to the vote, AND in significant numbers, then they can get an extension. The initial design aims to allow an extension if on the day the poll closes, a sufficient amount of the vote was shifted in the minority direction, an extra day is added. This will happen for each new deadline.

    // THE BALLOT
    // Ballot is a single proposal that can be voted on.
    // Each ballot will run for the minimum period of time.
    // The deadline can be extended with dissenting votes from the current majority vote.
    // The turnout can be extended with dissenting votes from the current majority vote.
    // the threshold is dictated by the turnout. In this implementation the curve is fixed, and linear.
    // At 1/8th turnout, the threshold to be met is 100%
    // At 7/8th turnout, the threshold to be met is 51%

    use std::fixed_point32;
    use std::vector;
    use std::signer;
    use std::guid;
    use std::error;
    use std::option::{Self, Option};
    use ol_framework::epoch_helper;
    use ol_framework::testnet;
    use ol_framework::vote_receipt;

    friend ol_framework::donor_voice_governance;
    #[test_only]
    friend ol_framework::test_turnout_tally;
    #[test_only]
    friend ol_framework::turnout_tally_demo;

    /// The ballot has already been completed.
    const ECOMPLETED: u64 = 1;
    /// The number of votes cast cannot be greater than the max number of votes available from enrollment.
    const EVOTES_GREATER_THAN_ENROLLMENT: u64 = 2;
    /// The threshold curve parameters are wrong. The curve is not decreasing.
    const EVOTE_CALC_PARAMS: u64 = 3;
    /// Voters cannot vote twice, but they can retract a vote
    const EALREADY_VOTED: u64 = 4;
    /// The voter has not voted yet. Cannot retract a vote.
    const ENOT_VOTED: u64 = 5;
    /// No ballot found under that guid
    const ENO_BALLOT_FOUND: u64 = 6;
    /// Bad status enum
    const EBAD_STATUS_ENUM: u64 = 7;
    /// initialized with zero
    const EINITIALZE_ZERO: u64 = 8;
    /// The vote approval is not above the threshold
    const ENOT_ABOVE_THRESHOLD: u64 = 9;
    /// The turnout is not above the minimum required
    const EINSUFFICIENT_TURNOUT: u64 = 10;

    // TODO: These may be variable on a per project basis. And these
    // should just be defaults.
    const PCT_SCALE: u64 = 10000;
    const LOW_TURNOUT_X1: u64 = 1250;// 12.5% turnout
    const HIGH_TURNOUT_X2: u64 = 8750;// 87.5% turnout
    const THRESH_AT_LOW_TURNOUT_Y1: u64 = 10000;// 100% pass at low turnout
    const THRESH_AT_HIGH_TURNOUT_Y2: u64 = 5100;// 51% pass at high turnout
    // TODO: this is not implemented, revisit.
    const MINORITY_EXT_MARGIN: u64 = 500; // The change in vote gap between majority and minority must have changed in the last day by this amount to for the minority to get an extension.

    // poor man's enum for the ballot status. Wen enum?
    const PENDING: u8  = 1;
    const APPROVED: u8 = 2;
    const REJECTED: u8 = 3;


    // Participation is a Library for Creating Ballots or Polls
    // Polls is a helper to keep track of Ballots but it is not required.

    // usually a smart contract will be the one to create the ballots
    // connected to some contract logic.

    // A ballot that passes, can be used for lazy triggering of actions related to the ballot.

    // The contract may have multiple ballots at a given time.
    // Historical completed ballots are also stored in a separate vector.
    // Developers can use Poll struct to instantiate an election.
    // or then can use Ballot, for a custom voting solution.
    // lastly the developer can simply wrap referendum into another struct with more context.

    /// for voting to happen with the VoteLib module, the guid creation capability must be passed in, and so the signer for the address (the "sponsor" of the ballot) must move the capability to be accessible by the contract logic.


    struct TurnoutTally<Data> has key, store, drop { // Note, this is a hot potato. Any methods changing it must return the struct to caller.
      data: Data,
      cfg_deadline: u64, // original deadline, which may be extended. Note deadline is at the END of this epoch (cfg_deadline + 1 stops taking votes)
      cfg_max_extensions: u64, // if 0 then no max. Election can run until threshold is met.
      cfg_min_turnout: u64,
      cfg_minority_extension: bool,
      completed: bool,
      enrollment: vector<address>, // optional to check for enrollment.
      max_votes: u64, // what's the entire universe of votes. i.e. 100% turnout
      // vote_tickets: VoteTicket, // the tickets that can be used to vote, which will be deducted as votes are cast. It is initialized with the max_votes.
      // Note the developer needs to be aware that if the right to vote changes throughout the period of the election (more coins, participants etc) then the max_votes and tickets could skew from expected results. Vote tickets can be distributed in advance.
      votes_approve: u64, // the running tally of approving votes,
      votes_reject: u64, // the running tally of rejecting votes,
      extended_deadline: u64, // which epoch was the deadline extended to
      last_epoch_voted: u64, // the last epoch which received a vote
      last_epoch_approve: u64, // what was the approval percentage at the last epoch. For purposes of calculating extensions.
      last_epoch_reject: u64, // what was the rejection percentage at the last epoch. For purposes of calculating extensions.
      provisional_pass_epoch: u64, // once a threshold is met, mark that epoch, a further vote on the next epoch will seal the election, to give time for the minority to vote.
      tally_approve_pct: u64,  // use two decimal places 1234 = 12.34%
      tally_turnout_pct: u64, // use two decimal places 1234 = 12.34%
      tally_pass: bool, // if it passed
    }

    public(friend) fun new_tally_struct<Data: drop + store>(
      data: Data,
      max_vote_enrollment: u64,
      deadline: u64,
      max_extensions: u64,
    ): TurnoutTally<Data> {
        let cfg_min_turnout = if (testnet::is_not_mainnet()) {
          500 //
        } else {
          LOW_TURNOUT_X1
        };

        TurnoutTally<Data> {
          data,
          cfg_deadline: deadline,
          cfg_max_extensions: max_extensions, // 0 means infinite extensions
          cfg_min_turnout, // requires unanimous vote to pass
          cfg_minority_extension: true,
          completed: false,
          enrollment: vector::empty<address>(), // TODO: maybe consider merkle roots, or bloom filters here.
          max_votes: max_vote_enrollment,
          votes_approve: 0,
          votes_reject: 0,
          extended_deadline: deadline, // DEPRECATED
          last_epoch_voted: 0,
          last_epoch_approve: 0,
          last_epoch_reject: 0,
          provisional_pass_epoch: 0, // DEPRECATED
          tally_approve_pct: 0,
          tally_turnout_pct: 0,
          tally_pass: false,
        }
    }

    fun update_enrollment<Data: drop + store>(ballot: &mut TurnoutTally<Data>, enrollment: vector<address>) {
      assert!(!is_poll_closed(ballot), error::invalid_state(ECOMPLETED));
      ballot.enrollment = enrollment;
    }

    // Only the contract, which is the keeper of the TurnoutTally, can allow a user to temporarily hold the TurnoutTally struct to update the vote. The user cannot arbitrarily update the vote, with an arbitrary number of votes.
    // This is a hot potato, it cannot be dropped.

    // the vote flow will return if the ballot passed (on the vote that gets over the threshold). This can be used for triggering actions lazily.

    public(friend) fun vote<Data: drop + store>(
      user: &signer,
      ballot: &mut TurnoutTally<Data>,
      uid: &guid::ID,
      approve_reject: bool,
      weight: u64
    ): Option<bool> {
      // voting should not be complete
      assert!(!is_poll_closed(ballot), error::invalid_state(ECOMPLETED));

      // check if this person voted already.
      // If the vote is the same directionally (approve, reject), exit early.
      // otherwise, need to subtract the old vote and add the new vote.
      let user_addr = signer::address_of(user);
      let (_, is_found) = vote_receipt::find_prior_vote_idx(user_addr, uid);

      // TODO: handle a user changing the vote
      // Commit note: do not abort so that a duplicate voter can tally
      // and close the poll
      if(!is_found) {
        // commit note: deprecated tracking

        // in every case, add the new vote
        ballot.last_epoch_voted = epoch_helper::get_current_epoch();
        if (approve_reject) {
          ballot.votes_approve = ballot.votes_approve + weight;
        } else {
          ballot.votes_reject = ballot.votes_reject + weight;
        };

        // this will handle the case of updating the receipt in case this is a second vote.
        vote_receipt::make_receipt(user, uid, approve_reject, weight);
      };

      // always tally on each vote
      // make sure all extensions happened in previous step.
      save_tally(ballot)
    }

    public fun is_poll_closed<Data: drop + store>(ballot: &TurnoutTally<Data>): bool {
      let epoch = epoch_helper::get_current_epoch();
      // if completed, exit early
      if (ballot.completed) { return true }; // this should be checked above anyways.

      if (ballot.cfg_deadline == 0) {
        // if the deadline is 0, then it never expires.
        return false
      };

      // the poll expires after this time.
      epoch > ballot.cfg_deadline
    }

    public fun maybe_close_poll<Data: drop + store>(ballot: &mut TurnoutTally<Data>): bool {
      let closed = is_poll_closed(ballot);
      ballot.completed = closed;

      closed
    }

    public(friend) fun retract<Data: drop + store>(
      ballot: &mut TurnoutTally<Data>,
      uid: &guid::ID,
      user: &signer
    ) {
      let user_addr = signer::address_of(user);

      let (_idx, is_found) = vote_receipt::find_prior_vote_idx(user_addr, uid);
      assert!(is_found, error::invalid_state(ENOT_VOTED));

      let (approve_reject, weight) = vote_receipt::get_receipt_data(user_addr, uid);

      if (approve_reject) {
        ballot.votes_approve = ballot.votes_approve - weight;
      } else {
        ballot.votes_reject = ballot.votes_reject - weight;
      };

      vote_receipt::remove_vote_receipt(user, uid);
    }

    /// The handler for a third party contract may wish to extend the ballot deadline.
    /// DANGER: the third-party ballot contract needs to know what it is doing. If this ballot object is exposed to end users it's game over.

    public(friend) fun extend_deadline<Data: drop + store>(ballot: &mut TurnoutTally<Data>, new_epoch: u64) {

      ballot.extended_deadline = new_epoch;
    }


    /// Evaluate the poll, and calculate the turnout and threshold.
    /// The tally must be called after the threshold is met.
    /// Each vote will try to tally, since the first vote above the threshold
    /// will cause the poll to close early.
    /// If the poll expires this function can also be called, outside of a vote
    /// can close the poll.
    /// (note: a duplicate vote will also call this function without affecting the result)
    public fun save_tally<Data: drop + store>(ballot: &mut TurnoutTally<Data>): Option<bool> {

      // figure out the turnout
      ballot.tally_turnout_pct = get_current_ballot_participation(ballot);


      // calculate the dynamic threshold needed.
      let thresh = get_current_threshold_required(ballot);
      // check the threshold that needs to be met met turnout
      ballot.tally_approve_pct = get_current_ballot_approval(ballot);

      if (
        // Threshold must be above dynamically calculated threshold
        ballot.tally_approve_pct >= thresh &&
        // before marking it pass, make sure the minimum quorum was met
        // by default 12.50%
        ballot.tally_turnout_pct >= ballot.cfg_min_turnout
        ) {
            ballot.completed = true;
            ballot.tally_pass = true;
        } else {
          maybe_close_poll(ballot);
        };

      if (ballot.completed) { return option::some(ballot.tally_pass) };
      option::none<bool>() // return option::some() if complete, and bool if it passed, so it can be used in a third party contract handler for lazy evaluation.
    }

    /// assert that the ballot would pass with the current tally
    /// if not surface the error for why it does not
    public(friend) fun assert_pass<Data: drop + store>(ballot: & TurnoutTally<Data>) {
      let thresh = get_current_threshold_required(ballot);
      assert!(ballot.tally_approve_pct > thresh, error::invalid_state(ENOT_ABOVE_THRESHOLD));
      assert!(ballot.tally_turnout_pct > ballot.cfg_min_turnout, error::invalid_state(EINSUFFICIENT_TURNOUT));

    }



    //////// GETTERS ////////

    /// get current tally percentage scaled
    public(friend) fun get_current_ballot_participation<Data: store>(ballot: &TurnoutTally<Data>): u64 {
      let total_votes = ballot.votes_approve + ballot.votes_reject;

      if (total_votes == 0) {
        return 0
      };

      assert!(ballot.max_votes >= total_votes, error::invalid_state(EVOTES_GREATER_THAN_ENROLLMENT));

      assert!(ballot.max_votes > 0, error::invalid_state(EINITIALZE_ZERO));


      return fixed_point32::multiply_u64(PCT_SCALE, fixed_point32::create_from_rational(total_votes, ballot.max_votes))
    }

    // with the current participation get the threshold to pass
    public(friend) fun get_current_threshold_required<Data: store>(ballot: &TurnoutTally<Data>): u64 {
      // Get current participation percentage
      let current_participation_pct = get_current_ballot_participation(ballot);

      // 1. Get the minimum participation threshold
      let min_participation = ballot.cfg_min_turnout;

      // 2 & 3. If we haven't reached minimum participation, calculate threshold at minimum
      if (current_participation_pct < min_participation) {
        // Calculate what the threshold would be at the minimum participation
        let min_votes_required = fixed_point32::multiply_u64(
          ballot.max_votes,
          fixed_point32::create_from_rational(min_participation, PCT_SCALE)
        );
        return get_threshold_from_turnout(min_votes_required, ballot.max_votes)
      };

      // 4. If above minimum, calculate threshold based on current turnout
      get_threshold_from_turnout(ballot.votes_approve + ballot.votes_reject, ballot.max_votes)
    }

    /// get current tally percentage scaled
    public(friend) fun get_current_ballot_approval<Data: store>(ballot: &TurnoutTally<Data>): u64 {
      let total = ballot.votes_approve + ballot.votes_reject;
      if (total == 0) {
        return 0
      };
      return fixed_point32::multiply_u64(PCT_SCALE, fixed_point32::create_from_rational(ballot.votes_approve, total))
    }

    public(friend) fun get_minimum_turnout<Data: store>(ballot: &TurnoutTally<Data>): u64 {
      ballot.cfg_min_turnout
    }


    /// Get the current expiration epoch for a ballot
    public(friend) fun get_expiration_epoch<Data: store>(ballot: &TurnoutTally<Data>): u64 {
      // Return the extended deadline if it exists, otherwise the original deadline
      if (ballot.extended_deadline > ballot.cfg_deadline) {
        ballot.extended_deadline
      } else {
        ballot.cfg_deadline
      }
    }

    public(friend) fun get_tally_data<Data: store>(ballot: &TurnoutTally<Data>): &Data {
      &ballot.data
    }

    /// is it complete and what's the result
    public(friend) fun get_result<Data: store>(ballot: &TurnoutTally<Data>): (bool, bool) {
      (ballot.completed, ballot.tally_pass)
    }

    #[view]
    // TODO: this should probably use Decimal.move
    // can't multiply fixed_point32 types directly.
    public fun get_threshold_from_turnout(voters: u64, max_votes: u64): u64 {
      assert!(voters > 0, error::invalid_state(EINITIALZE_ZERO));
      assert!(max_votes > 0, error::invalid_state(EINITIALZE_ZERO));

      let turnout = fixed_point32::create_from_rational(voters, max_votes);
      let turnout_scaled_x = fixed_point32::multiply_u64(PCT_SCALE, turnout); // scale to two decimal points.
      // only implementing the negative slope case. Unsure why the other is needed.

      assert!(THRESH_AT_LOW_TURNOUT_Y1 > THRESH_AT_HIGH_TURNOUT_Y2, error::invalid_state(EVOTE_CALC_PARAMS));

      // the minimum passing threshold is the low turnout threshold.
      // same for the maximum turnout threshold.
      if (turnout_scaled_x < LOW_TURNOUT_X1) {
        return THRESH_AT_LOW_TURNOUT_Y1
      } else if (turnout_scaled_x > HIGH_TURNOUT_X2) {
        return THRESH_AT_HIGH_TURNOUT_Y2
      };


      let abs_m = fixed_point32::create_from_rational(
        (THRESH_AT_LOW_TURNOUT_Y1 - THRESH_AT_HIGH_TURNOUT_Y2), (HIGH_TURNOUT_X2 - LOW_TURNOUT_X1)
      );

      let abs_mx = fixed_point32::multiply_u64(LOW_TURNOUT_X1, *&abs_m);
      let b = THRESH_AT_LOW_TURNOUT_Y1 + abs_mx;
      let y =  b - fixed_point32::multiply_u64(turnout_scaled_x, *&abs_m);

      return y
    }


    //////// TODO ///////
    // TODO: evaluate if we want dynamic deadlines
    // based on competitiveness

    // /// A third party contract can optionally call this function to extend the deadline to extend ballots in competitive situations.
    // /// we may need to extend the ballot if on the last day (TBD a wider window) the vote had a big shift in favor of the minority vote.
    // /// All that needs to be done, is on the return of vote(), to then call this function.
    // /// It's a useful feature, but it will not be included by default in all votes.
    // fun maybe_auto_competitive_extend<Data: drop + store>(ballot: &mut TurnoutTally<Data>):u64  {

    //   let epoch = epoch_helper::get_current_epoch();

    //   // TODO: The extension window below of 1 day is not sufficient to make
    //   // much difference in practice (the threshold is most likely reached at that point).

    //   // Are we on the last day of voting (extension window)? If not exit
    //   if (epoch == ballot.extended_deadline || epoch == ballot.cfg_deadline) { return ballot.extended_deadline };

    //   if (is_competitive(ballot)) {
    //     // we may have extended already, but we don't want to extend more than once per day.
    //     if (ballot.extended_deadline > epoch) { return ballot.extended_deadline };

    //     // extend the deadline by 1 day
    //     ballot.extended_deadline = epoch + 1;
    //   };


    //   ballot.extended_deadline
    // }

    // fun is_competitive<Data: drop + store>(ballot: &TurnoutTally<Data>): bool {
    //   let (prev_lead, prev_trail, prev_lead_updated, prev_trail_updated) = if (ballot.last_epoch_approve > ballot.last_epoch_reject) {
    //     // if the "approve" vote WAS leading.
    //     (ballot.last_epoch_approve, ballot.last_epoch_reject, ballot.votes_approve, ballot.votes_reject)

    //   } else {
    //     (ballot.last_epoch_reject, ballot.last_epoch_approve, ballot.votes_reject, ballot.votes_approve)
    //   };


    //   // no votes yet
    //   if (prev_lead == 0 && prev_trail == 0) { return false };
    //   if (prev_lead_updated == 0 && prev_trail_updated == 0) { return false};

    //   let prior_margin = ((prev_lead - prev_trail) * PCT_SCALE) / (prev_lead + prev_trail);


    //   // the current margin may have flipped, so we need to check the direction of the vote.
    //   // if so then give an automatic extensions
    //   if (prev_lead_updated < prev_trail_updated) {
    //     return true
    //   } else {
    //     let current_margin = (prev_lead_updated - prev_trail_updated) * PCT_SCALE / (prev_lead_updated + prev_trail_updated);

    //     if (current_margin - prior_margin > MINORITY_EXT_MARGIN) {
    //       return true
    //     }
    //   };
    //   false
    // }

  }
