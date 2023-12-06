/////////////////////////////////////////////////////////////////////////
// 0L Module
// CASES of validation and mining
/////////////////////////////////////////////////////////////////////////

/// # Summary
/// This module can be used by root to determine whether a validator is compliant
/// Validators who are no longer compliant may be kicked out of the validator
/// set and/or jailed. To be compliant, validators must be BOTH validating and mining.
module ol_framework::grade {
    use diem_framework::stake;
    use std::fixed_point32::{Self, FixedPoint32};

    // use diem_std::debug::print;

    /// what threshold of failed props should the network allow
    /// one validator before jailing?
    const FAILED_PROPS_THRESHOLD_PCT: u64 =20;

    /// how far behind the leading validator by net proposals
    /// should the trailing validator be allowed
    const TRAILING_VALIDATOR_THRESHOLD: u64 = 5;


    #[view]
    /// returns if the validator passed or failed, and the number of proposals
    /// and failures, and the ratio.
    /// @return: tuple
    /// bool: is the validator compliant,
    /// u64: proposed blocks,
    /// u64: failed blocks, and
    /// FixedPoint32: the ratio of proposed to failed.

    public fun get_validator_grade(node_addr: address, highest_net_props: u64): (bool, u64, u64, FixedPoint32) {
      let idx = stake::get_validator_index(node_addr);
      let (proposed, failed) = stake::get_current_epoch_proposal_counts(idx);

      // first pass: should have accepted proposals
      if (proposed < failed) {
        return (false, proposed, failed, fixed_point32::create_from_raw_value(0))
      };

      let compliant = has_good_success_ratio(proposed, failed) &&
      does_not_trail(proposed, failed, highest_net_props);

      // make failed at leat 1 to avoid division by zero
      (compliant, proposed, failed, fixed_point32::create_from_rational(proposed, (failed + 1)))
    }

    /// Does the validator produce far more valid proposals than failed ones.
    /// suggesting here an initial 80% threshold. (i.e. only allow 20% failed
    /// proposals)
    /// It's unclear what the right ratio should be. On problematic epochs
    /// (i.e. low consensus) we may want to have some allowance for errors.
    fun has_good_success_ratio(proposed: u64, failed: u64): bool {

      let fail_ratio = if (proposed >= failed) {
        // +1 to prevent denomiator zero error
        fixed_point32::create_from_rational(failed, (proposed + 1))
      } else { return false };

      // failure ratio shoul dbe BELOW FAILED_PROPS_THRESHOLD_PCT
      let is_above = fixed_point32::multiply_u64(100, fail_ratio) < FAILED_PROPS_THRESHOLD_PCT;

      is_above

    }

    /// is this user very far behind the leading proposer.
    /// in 0L we give all validator seats equal chances of proposing (leader).
    /// However, the "leader reputation" algorithm in LibraBFT, will score
    /// validators. The score is based on successful signing and propagation of
    /// blocks. So people with lower scores might a) not be very competent or,
    /// b) are in networks that are further removed from the majority (remote
    /// data centers, home labs, etc). So we shouldn't bias too heavily on this
    // performance metric, because it will reduce heterogeneity (the big D).
    /// Unfortunately the reputation heuristic score is not on-chain
    /// (perhaps a future optimization).
    /// A good solution would be to drop the lowest % of RMS (root mean
    /// squared) of proposal. Perhaps the lowest 10% of the validator set RMS.
    /// But that's harder to understand by the validators (comprehension is
    // important when we are trying to shape behavior).
    /// So a simpler and more actionable metric might be:
    /// drop anyone who proposed less than X% of the LEADING proposal.
    /// This has an additional quality, which allows for the highest performing
    // validators to be a kind of "vanguard" rider ("forerider"?) which sets the
    // pace for the lowest performer.
    fun does_not_trail(proposed: u64, failed: u64, highest_net_props: u64): bool {
      if ((proposed > failed) && (highest_net_props > 0)) {
        let net = proposed - failed;
        let net_props_vs_leader= fixed_point32::create_from_rational(net,
        highest_net_props);
        fixed_point32::multiply_u64(100, net_props_vs_leader) > FAILED_PROPS_THRESHOLD_PCT
      } else { false }
    }
}
