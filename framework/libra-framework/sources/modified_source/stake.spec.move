// TODO: rename this to validator.move if
// diem-platform allows
spec diem_framework::stake {
    // -----------------
    // Global invariants
    // -----------------
    use diem_framework::chain_status;
    use diem_framework::timestamp;

    spec module {
        // The validator set should satisfy its desired invariant.
        invariant [suspendable] exists<ValidatorSet>(@diem_framework) ==> validator_set_is_valid();
        // After genesis, `DiemCoinCapabilities`, `ValidatorPerformance` and `ValidatorSet` exist.
        // invariant [suspendable] chain_status::is_operating() ==> exists<DiemCoinCapabilities>(@diem_framework);
        invariant [suspendable] chain_status::is_operating() ==> exists<ValidatorPerformance>(@diem_framework);
        invariant [suspendable] chain_status::is_operating() ==> exists<ValidatorSet>(@diem_framework);
    }

    // A desired invariant for the validator set.
    spec fun validator_set_is_valid(): bool {
        let validator_set = global<ValidatorSet>(@diem_framework);
        spec_validators_are_initialized(validator_set.active_validators) &&
            // spec_validators_are_initialized(validator_set.pending_inactive) &&
            // spec_validators_are_initialized(validator_set.pending_active) &&
            spec_validator_indices_are_valid(validator_set.active_validators)
            // spec_validator_indices_are_valid(validator_set.pending_inactive)
    }


    // -----------------------
    // Function specifications
    // -----------------------

    // `Validator` is initialized once.
    spec initialize(diem_framework: &signer) {
        let diem_addr = signer::address_of(diem_framework);
        aborts_if !system_addresses::is_diem_framework_address(diem_addr);
        aborts_if exists<ValidatorSet>(diem_addr);
        aborts_if exists<ValidatorPerformance>(diem_addr);
    }

    spec update_network_and_fullnode_addresses(
        operator: &signer,
        validator_address: address,
        new_network_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    ) {
        let pre_stake_pool = global<ValidatorState>(validator_address);
        let post validator_info = global<ValidatorConfig>(validator_address);
        modifies global<ValidatorConfig>(validator_address);

        // Only the true operator address can update the network and full node addresses of the validator.
        aborts_if !exists<ValidatorState>(validator_address);
        aborts_if !exists<ValidatorConfig>(validator_address);
        aborts_if signer::address_of(operator) != pre_stake_pool.operator_address;

        ensures validator_info.network_addresses == new_network_addresses;
        ensures validator_info.fullnode_addresses == new_fullnode_addresses;
    }

    spec set_operator_with_cap(owner_cap: &OwnerCapability, new_operator: address) {
        let validator_address = owner_cap.validator_address;
        let post stake_pool = global<ValidatorState>(validator_address);
        modifies global<ValidatorState>(validator_address);
        ensures stake_pool.operator_address == new_operator;
    }

    // spec reactivate_stake_with_cap(owner_cap: &OwnerCapability, amount: u64) {
    //     let validator_address = owner_cap.validator_address;
    //     aborts_if !stake_pool_exists(validator_address);

    //     let pre_stake_pool = global<ValidatorState>(validator_address);
    //     let post stake_pool = global<ValidatorState>(validator_address);
    //     modifies global<ValidatorState>(validator_address);
    //     let min_amount = diem_std::math64::min(amount,pre_stake_pool.pending_inactive.value);

    //     ensures stake_pool.active.value == pre_stake_pool.active.value + min_amount;
    // }

    spec rotate_consensus_key(
        operator: &signer,
        validator_address: address,
        new_consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
    ) {
        let pre_stake_pool = global<ValidatorState>(validator_address);
        let post validator_info = global<ValidatorConfig>(validator_address);
        modifies global<ValidatorConfig>(validator_address);

        ensures validator_info.consensus_pubkey == new_consensus_pubkey;
    }

    // spec set_delegated_voter_with_cap(owner_cap: &OwnerCapability, new_voter: address) {
    //     let validator_address = owner_cap.validator_address;
    //     let post stake_pool = global<ValidatorState>(validator_address);
    //     modifies global<ValidatorState>(validator_address);
    //     ensures stake_pool.delegated_voter == new_voter;
    // }

    // spec on_new_epoch {
    //     pragma disable_invariants_in_body;
    //     // The following resource requirement cannot be discharged by the global
    //     // invariants because this function is called during genesis.
    //     include ResourceRequirement;
    //     // include staking_config::StakingRewardsConfigRequirement;
    //     // This function should never abort.
    //     aborts_if false;
    // }

    spec update_performance_statistics {
        // This function is expected to be used after genesis.
        requires chain_status::is_operating();
        // This function should never abort.
        aborts_if false;
    }

    spec find_validator {
        pragma opaque;
        aborts_if false;
        ensures option::is_none(result) ==> (forall i in 0..len(v): v[i].addr != addr);
        ensures option::is_some(result) ==> v[option::borrow(result)].addr == addr;
        // Additional postcondition to help the quantifier instantiation.
        ensures option::is_some(result) ==> spec_contains(v, addr);
    }


    spec is_current_epoch_validator {
        include ResourceRequirement;
        aborts_if !spec_has_stake_pool(addr);
        ensures result == spec_is_current_epoch_validator(addr);
    }

    spec initialize_stake_owner {
        include ResourceRequirement;
    }

    spec assert_stake_pool_exists(validator_address: address) {
        aborts_if !stake_pool_exists(validator_address);
    }

    spec configure_allowed_validators(diem_framework: &signer, accounts: vector<address>) {
        let diem_framework_address = signer::address_of(diem_framework);
        aborts_if !system_addresses::is_diem_framework_address(diem_framework_address);
        let post allowed = global<AllowedValidators>(diem_framework_address);
        // Make sure that the accounts of AllowedValidators are always the passed parameter.
        ensures allowed.accounts == accounts;
    }

    spec assert_owner_cap_exists(owner: address) {
        aborts_if !exists<OwnerCapability>(owner);
    }

    // ---------------------------------
    // Spec helper functions and schemas
    // ---------------------------------

    // A predicate that all given validators have been initialized.
    spec fun spec_validators_are_initialized(validators: vector<ValidatorInfo>): bool {
        forall i in 0..len(validators):
            spec_has_stake_pool(validators[i].addr) &&
                spec_has_validator_config(validators[i].addr)
    }

    // A predicate that the validator index of each given validator in-range.
    spec fun spec_validator_indices_are_valid(validators: vector<ValidatorInfo>): bool {
        forall i in 0..len(validators):
            global<ValidatorConfig>(validators[i].addr).validator_index < spec_validator_index_upper_bound()
    }

    // The upper bound of validator indices.
    spec fun spec_validator_index_upper_bound(): u64 {
        len(global<ValidatorPerformance>(@diem_framework).validators)
    }

    spec fun spec_has_stake_pool(a: address): bool {
        exists<ValidatorState>(a)
    }

    spec fun spec_has_validator_config(a: address): bool {
        exists<ValidatorConfig>(a)
    }

    // An uninterpreted spec function to represent the stake reward formula.
    spec fun spec_rewards_amount(
        stake_amount: u64,
        num_successful_proposals: u64,
        num_total_proposals: u64,
        rewards_rate: u64,
        rewards_rate_denominator: u64,
    ): u64;

    spec fun spec_contains(validators: vector<ValidatorInfo>, addr: address): bool {
        exists i in 0..len(validators): validators[i].addr == addr
    }

    spec fun spec_is_current_epoch_validator(validator_address: address): bool {
        let validator_set = global<ValidatorSet>(@diem_framework);
        spec_contains(validator_set.active_validators, validator_address)
    }

    // These resources are required to successfully execute `on_new_epoch`, which cannot
    // be discharged by the global invariants because `on_new_epoch` is called in genesis.
    spec schema ResourceRequirement {
        // requires exists<DiemCoinCapabilities>(@diem_framework);
        requires exists<ValidatorPerformance>(@diem_framework);
        requires exists<ValidatorSet>(@diem_framework);
        // requires exists<StakingConfig>(@diem_framework);
        // requires exists<StakingRewardsConfig>(@diem_framework) || !features::spec_periodical_reward_rate_decrease_enabled();
        requires exists<timestamp::CurrentTimeMicroseconds>(@diem_framework);
        requires exists<ValidatorFees>(@diem_framework);
    }
}
