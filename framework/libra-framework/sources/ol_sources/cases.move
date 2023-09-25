/////////////////////////////////////////////////////////////////////////
// 0L Module
// CASES of validation and mining
// Error code for File: 0300
/////////////////////////////////////////////////////////////////////////

// TODO: v7: this is legacy, since we only now have two cases: performing and non-performing.

/// # Summary
/// This module can be used by root to determine whether a validator is compliant
/// Validators who are no longer compliant may be kicked out of the validator
/// set and/or jailed. To be compliant, validators must be BOTH validating and mining.
module ol_framework::cases{
    // use DiemFramework::TowerState;
    use diem_framework::stake;
    use std::vector;
    use std::fixed_point32::{Self, FixedPoint32};
    // use DiemFramework::Roles; // todo v7

    const VALIDATOR_COMPLIANT: u64 = 1;
    // const VALIDATOR_HALF_COMPLIANT: u64 = 2;
    // const VALIDATOR_NOT_COMPLIANT: u64 = 3;
    const VALIDATOR_DOUBLY_NOT_COMPLIANT: u64 = 4;

    const INVALID_DATA: u64 = 0;

    /// struct for use with performance
    struct PerformanceRank has drop, copy {
      val: address,
      rank: u64,
      ratio: FixedPoint32,
      compliant: bool,
    }

    // TODO: temp just to get a build
    fun node_above_thresh(node_addr: address): bool {
      let idx = stake::get_validator_index(node_addr);
      let (proposed, failed) = stake::get_current_epoch_proposal_counts(idx);

      let _ = proposed > failed;
      true
    }

    #[view]
    /// returns if the validator passed or failed, and the number of proposals
    /// and failures, and the ratio.
    public fun get_validator_grade(node_addr: address): (bool, u64, u64, FixedPoint32) {
      let idx = stake::get_validator_index(node_addr);
      let (proposed, failed) = stake::get_current_epoch_proposal_counts(idx);

      let compliant = proposed > failed;
      // make failed at leat 1 to avoid division by zero
      (compliant, proposed, failed, fixed_point32::create_from_rational(proposed, (failed + 1)))
    }

    // Determine the consensus case for the validator.
    // This happens at an epoch prologue, and labels the validator based on
    // performance in the outgoing epoch.
    // The consensus case determines if the validator receives transaction
    // fees or subsidy for performance, inclusion in following epoch, and
    // at what voting power.
    // Permissions: Public, VM Only
    public fun get_case(
        node_addr: address): u64 {

        // this is a failure mode. Only usually seen in rescue missions,
        // where epoch counters are reconfigured by writeset offline.
        // if (height_end < height_start) return INVALID_DATA;

        // Roles::assert_diem_root(vm); // todo v7
        // did the validator sign blocks above threshold?
        let signs = node_above_thresh(node_addr);

        // let mines = TowerState::node_above_thresh(node_addr);

        if (signs) {
            // compliant: in next set, gets paid, weight increments
            VALIDATOR_COMPLIANT
        }
        // V6: Simplify compliance cases by removing mining.

        // }
        // else if (signs && !mines) {
        //     // half compliant: not in next set, does not get paid, weight
        //     // does not increment.
        //     VALIDATOR_HALF_COMPLIANT
        // }
        // else if (!signs && mines) {
        //     // not compliant: jailed, not in next set, does not get paid,
        //     // weight increments.
        //     VALIDATOR_NOT_COMPLIANT
        // }
        else {
            // not compliant: jailed, not in next set, does not get paid,
            // weight does not increment.
            VALIDATOR_DOUBLY_NOT_COMPLIANT
        }
    }

    #[view]
    /// get the validators which are not compliant
    public fun get_jailed_set(): vector<address> {
      let validator_set = stake::get_current_validators();
      let jailed_set = vector::empty<address>();
      let k = 0;
      while(k < vector::length(&validator_set)){
        let addr = *vector::borrow<address>(&validator_set, k);
        // consensus case 1 allow inclusion into the next validator set.
        if (get_case(addr) == 4){
          vector::push_back<address>(&mut jailed_set, addr)
        };
        k = k + 1;
      };
      jailed_set
    }

    /// returns a list of PerformanceRank, of an individual validator's performance
    /// only_perf, returns only the performant
    /// only_jail, return only the jailed
    /// cannot both be true
    public fun rank_validators(n: u64, only_perf: bool, only_jail: bool): vector<PerformanceRank> {
      if (only_perf && only_jail) {
        return vector::empty<PerformanceRank>()
      };


      let eligible_validators = stake::get_current_validators();
      let length = vector::length<address>(&eligible_validators);
      let weights = vector::empty<PerformanceRank>();

      let k = 0;
      while (k < length) {

        let val = *vector::borrow<address>(&eligible_validators, k);

        let (compliant, _proposed, _failed, ratio) = get_validator_grade(val);

        // skip jailed when we only want performant validators
        if (!compliant && only_perf ) {
          k = k + 1;
          continue
        };
        // skip compliant when we only want jailed validators
        if (compliant && only_jail) {
          k = k + 1;
          continue
        };

        let pr = PerformanceRank {
          val,
          rank: k,
          ratio,
          compliant, // assume true as failover
        };

        vector::push_back<PerformanceRank>(&mut weights, pr);
        k = k + 1;
      };

      // Sorting the accounts vector based on value (weights).
      // Bubble sort algorithm
      let len_filtered = vector::length(&weights);
      if (len_filtered < 2) return weights;
      let i = 0;
      while (i < len_filtered){
        let j = 0;
        while(j < len_filtered-i-1){

          let value_j = fixed_point32::multiply_u64(1000, (vector::borrow(&weights, j)).ratio);

          let value_jp1 = fixed_point32::multiply_u64(1000, (vector::borrow(&weights, j+1)).ratio);

          if(value_j > value_jp1){

            vector::swap(&mut weights, j, j+1);

          };
          j = j + 1;

        };
        i = i + 1;

      };


      // Reverse to have sorted order - high to low.
      vector::reverse(&mut weights);

      // Return the top n validators
      // if 0 is passed, then return the full set.
      if (n == 0) {
        return weights
      };

      let final = vector::empty();
      let m = 0;
      while ( m < n) {
         vector::push_back(&mut final, *vector::borrow(&weights, m));
         m = m + 1;
      };

      return final
    }

    //////// GETTERS ////////

    /// returns the validator's rank, ratio, compliant, and address as tuple
    public fun extract(pr: &PerformanceRank): (address, bool, u64, FixedPoint32) {
      (pr.val, pr.compliant,  pr.rank, pr.ratio)
    }

}
