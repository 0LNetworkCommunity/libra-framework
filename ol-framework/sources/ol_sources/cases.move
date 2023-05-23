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
    use aptos_framework::stake;
    use std::vector;
    // use DiemFramework::Roles; // todo v7

    const VALIDATOR_COMPLIANT: u64 = 1;
    // const VALIDATOR_HALF_COMPLIANT: u64 = 2;
    // const VALIDATOR_NOT_COMPLIANT: u64 = 3;
    const VALIDATOR_DOUBLY_NOT_COMPLIANT: u64 = 4;

    const INVALID_DATA: u64 = 0;

    // TODO: temp just to get a build
    fun node_above_thresh(node_addr: address): bool {
      let idx = stake::get_validator_index(node_addr);
      let (proposed, failed) = stake::get_current_epoch_proposal_counts(idx);

      proposed > failed
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
}
