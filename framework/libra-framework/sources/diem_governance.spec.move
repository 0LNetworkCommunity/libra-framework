spec diem_framework::diem_governance {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec store_signer_cap(
        diem_framework: &signer,
        signer_address: address,
        signer_cap: SignerCapability,
    ) {
        aborts_if !system_addresses::is_diem_framework_address(signer::address_of(diem_framework));
        aborts_if !system_addresses::is_framework_reserved_address(signer_address);

        let signer_caps = global<GovernanceResponsbility>(@diem_framework).signer_caps;
        aborts_if exists<GovernanceResponsbility>(@diem_framework) &&
            simple_map::spec_contains_key(signer_caps, signer_address);
        ensures exists<GovernanceResponsbility>(@diem_framework);
    }

    /// Signer address must be @diem_framework.
    /// The signer does not allow these resources (GovernanceProposal, GovernanceConfig, GovernanceEvents, VotingRecords, ApprovedExecutionHashes) to exist.
    /// The signer must have an Account.
    /// Limit addition overflow.
    spec initialize(
        diem_framework: &signer,
        min_voting_threshold: u128,
        voting_duration_secs: u64,
    ) {
        use diem_std::type_info::Self;

        let addr = signer::address_of(diem_framework);
        let register_account = global<account::Account>(addr);

        aborts_if exists<voting::VotingForum<GovernanceProposal>>(addr);
        aborts_if !exists<account::Account>(addr);
        aborts_if register_account.guid_creation_num + 7 > MAX_U64;
        aborts_if register_account.guid_creation_num + 7 >= account::MAX_GUID_CREATION_NUM;
        aborts_if !type_info::spec_is_struct<GovernanceProposal>();

        include InitializeAbortIf;

        ensures exists<voting::VotingForum<governance_proposal::GovernanceProposal>>(addr);
        ensures exists<GovernanceConfig>(addr);
        ensures exists<GovernanceEvents>(addr);
        ensures exists<VotingRecords>(addr);
        ensures exists<ApprovedExecutionHashes>(addr);
    }

    spec schema InitializeAbortIf {
        diem_framework: &signer;
        min_voting_threshold: u128;
        // required_proposer_stake: u64;
        voting_duration_secs: u64;

        let addr = signer::address_of(diem_framework);
        let account = global<account::Account>(addr);
        aborts_if addr != @diem_framework;
        aborts_if exists<voting::VotingForum<governance_proposal::GovernanceProposal>>(addr);
        aborts_if exists<GovernanceConfig>(addr);
        aborts_if exists<GovernanceEvents>(addr);
        aborts_if exists<VotingRecords>(addr);
        aborts_if exists<ApprovedExecutionHashes>(addr);
        aborts_if !exists<account::Account>(addr);
    }

    /// Signer address must be @diem_framework.
    /// Address @diem_framework must exist GovernanceConfig and GovernanceEvents.
    spec update_governance_config(
        diem_framework: &signer,
        min_voting_threshold: u128,
        voting_duration_secs: u64,
    ) {
        let addr = signer::address_of(diem_framework);
        let governance_config = global<GovernanceConfig>(@diem_framework);

        let post new_governance_config = global<GovernanceConfig>(@diem_framework);
        aborts_if addr != @diem_framework;
        aborts_if !exists<GovernanceConfig>(@diem_framework);
        aborts_if !exists<GovernanceEvents>(@diem_framework);
        modifies global<GovernanceConfig>(addr);

        ensures new_governance_config.voting_duration_secs == voting_duration_secs;
        ensures new_governance_config.min_voting_threshold == min_voting_threshold;
        // ensures new_governance_config.required_proposer_stake == required_proposer_stake;
    }

    spec get_voting_duration_secs(): u64 {
        include AbortsIfNotGovernanceConfig;
    }

    spec get_min_voting_threshold(): u128 {
        include AbortsIfNotGovernanceConfig;
    }

    // spec get_required_proposer_stake(): u64 {
    //     include AbortsIfNotGovernanceConfig;
    // }

    spec schema AbortsIfNotGovernanceConfig {
        aborts_if !exists<GovernanceConfig>(@diem_framework);
    }

    spec create_proposal_v2(
        proposer: &signer,
        execution_hash: vector<u8>,
        metadata_location: vector<u8>,
        metadata_hash: vector<u8>,
        is_multi_step_proposal: bool,
    ) {
        use diem_framework::chain_status;
        // TODO: The variable `stake_balance` is the return value of the function `get_voting_power`.
        // `get_voting_power` has already stated that it cannot be fully verified,
        // so the value of `stake_balance` cannot be obtained in the spec,
        // and the `aborts_if` of `stake_balancede` cannot be written.
        pragma aborts_if_is_partial;
        requires chain_status::is_operating();
        include CreateProposalAbortsIf;
    }

    /// `stake_pool` must exist StakePool.
    /// The delegated voter under the resource StakePool of the stake_pool must be the proposer address.
    /// Address @diem_framework must exist GovernanceEvents.
    spec schema CreateProposalAbortsIf {
        // use diem_framework::stake;

        proposer: &signer;
        execution_hash: vector<u8>;
        metadata_location: vector<u8>;
        metadata_hash: vector<u8>;

        let proposer_address = signer::address_of(proposer);
        let governance_config = global<GovernanceConfig>(@diem_framework);

        include AbortsIfNotGovernanceConfig;
        let current_time = timestamp::now_seconds();
        let proposal_expiration = current_time + governance_config.voting_duration_secs;
        aborts_if !exists<GovernanceEvents>(@diem_framework);
    }

    /// stake_pool must exist StakePool.
    /// The delegated voter under the resource StakePool of the stake_pool must be the voter address.
    /// Address @diem_framework must exist VotingRecords and GovernanceProposal.
    spec vote (
        voter: &signer,
        proposal_id: u64,
        should_pass: bool,
    ) {
        // use diem_framework::stake;
        use diem_framework::chain_status;

        // TODO: The variable `voting_power` is the return value of the function `get_voting_power`.
        // `get_voting_power` has already stated that it cannot be completely verified,
        // so the value of `voting_power` cannot be obtained in the spec,
        // and the `aborts_if` of `voting_power` cannot be written.
        pragma aborts_if_is_partial;

        requires chain_status::is_operating();

        let voter_address = signer::address_of(voter);
        // let stake_pool_res = global<stake::StakePool>(stake_pool);
        // aborts_if !exists<stake::StakePool>(stake_pool);
        // aborts_if stake_pool_res.delegated_voter != voter_address;
        aborts_if !exists<VotingRecords>(@diem_framework);
        aborts_if !exists<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        // let allow_validator_set_change = global<staking_config::StakingConfig>(@diem_framework).allow_validator_set_change;
        // aborts_if !allow_validator_set_change && !exists<stake::ValidatorSet>(@diem_framework);
        let voting_forum = global<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        let proposal = table::spec_get(voting_forum.proposals, proposal_id);
        let proposal_expiration = proposal.expiration_secs;
        // let locked_until_secs = global<stake::StakePool>(stake_pool).locked_until_secs;
        // aborts_if proposal_expiration > locked_until_secs;
    }

    spec add_approved_script_hash(proposal_id: u64) {
        use diem_framework::chain_status;
        // TODO: The variable `proposal_state` is the return value of the function `voting::get_proposal_state`.
        // The calling level of `voting::get_proposal_state` is very deep,
        // so the value of `proposal_state` cannot be obtained in the spec,
        // and the `aborts_if` of `proposal_state` cannot be written.
        // Can't cover all aborts_if conditions
        pragma aborts_if_is_partial;

        requires chain_status::is_operating();
        aborts_if !exists<ApprovedExecutionHashes>(@diem_framework);
    }

    spec add_approved_script_hash_script(proposal_id: u64) {
        // TODO: Calls `add_approved_script_hash`.
        // Can't cover all aborts_if conditions
        pragma verify = false;
    }

    // /// Address @diem_framework must exist ApprovedExecutionHashes and GovernanceProposal and GovernanceResponsbility.
    // spec resolve(proposal_id: u64, signer_address: address): signer {
    //     use diem_framework::chain_status;
    //     // TODO: Executing the prove command gives an error that the target file is in `from_bcs::from_bytes`,
    //     // and the call level of the function `resolve` is too deep to obtain the parameter `bytes` of spec `from_bytes`,
    //     // so verification cannot be performed.
    //     // Can't cover all aborts_if conditions
    //     pragma aborts_if_is_partial;

    //     requires chain_status::is_operating();
    //     aborts_if !exists<voting::VotingForum<GovernanceProposal>>(@diem_framework);
    //     aborts_if !exists<ApprovedExecutionHashes>(@diem_framework);
    //     include GetSignerAbortsIf;
    // }

    /// Address @diem_framework must exist ApprovedExecutionHashes and GovernanceProposal.
    spec remove_approved_hash(proposal_id: u64) {
        aborts_if !exists<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        aborts_if !exists<ApprovedExecutionHashes>(@diem_framework);
        let voting_forum = global<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        aborts_if !table::spec_contains(voting_forum.proposals, proposal_id);
        aborts_if !exists<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        let proposal = table::spec_get(voting_forum.proposals, proposal_id);
        aborts_if !proposal.is_resolved;
    }

    spec reconfigure(diem_framework: &signer) {
        use diem_framework::chain_status;
        use diem_framework::coin::CoinInfo;
        use diem_framework::libra_coin::LibraCoin;
        use diem_framework::transaction_fee;
        // use diem_framework::staking_config;

        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved)

        aborts_if !system_addresses::is_diem_framework_address(signer::address_of(diem_framework));

        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockDiemSupply;
        // include staking_config::StakingRewardsConfigRequirement;
        requires chain_status::is_operating();
        requires timestamp::spec_now_microseconds() >= reconfiguration::last_reconfiguration_time();
        requires exists<stake::ValidatorFees>(@diem_framework);
        requires exists<CoinInfo<LibraCoin>>(@diem_framework);
    }

    // /// Signer address must be @core_resources.
    // /// signer must exist in MintCapStore.
    // /// Address @diem_framework must exist GovernanceResponsbility.
    // spec get_signer_testnet_only(core_resources: &signer, signer_address: address): signer {
    //     aborts_if signer::address_of(core_resources) != @core_resources;
    //     aborts_if !exists<libra_coin::MintCapStore>(signer::address_of(core_resources));
    //     include GetSignerAbortsIf;
    // }

    /// Address @diem_framework must exist StakingConfig.
    /// limit addition overflow.
    /// pool_address must exist in StakePool.
    // spec get_voting_power(pool_address: address): u64 {
    //     // TODO: `stake::get_current_epoch_voting_power` is called in the function,
    //     // the call level is very deep, and `stake::get_stake` has multiple return values,
    //     // and multiple return values cannot be obtained in the spec,
    //     // so the overflow aborts_if of active + pending_active + pending_inactive cannot be written.
    //     pragma aborts_if_is_partial;

    //     let staking_config = global<staking_config::StakingConfig>(@diem_framework);
    //     aborts_if !exists<staking_config::StakingConfig>(@diem_framework);
    //     let allow_validator_set_change = staking_config.allow_validator_set_change;
    //     let stake_pool = global<stake::StakePool>(pool_address);
    //     aborts_if allow_validator_set_change && (stake_pool.active.value + stake_pool.pending_active.value + stake_pool.pending_inactive.value) > MAX_U64;
    //     aborts_if !exists<stake::StakePool>(pool_address);
    //     aborts_if !allow_validator_set_change && !exists<stake::ValidatorSet>(@diem_framework);

    //     ensures allow_validator_set_change ==> result == stake_pool.active.value + stake_pool.pending_active.value + stake_pool.pending_inactive.value;
    // }

    spec get_signer(signer_address: address): signer {
        include GetSignerAbortsIf;
    }

    spec schema GetSignerAbortsIf {
        signer_address: address;

        aborts_if !exists<GovernanceResponsbility>(@diem_framework);
        let cap_map = global<GovernanceResponsbility>(@diem_framework).signer_caps;
        aborts_if !simple_map::spec_contains_key(cap_map, signer_address);
    }

    spec create_proposal_metadata(metadata_location: vector<u8>, metadata_hash: vector<u8>): SimpleMap<String, vector<u8>> {
        aborts_if string::length(utf8(metadata_location)) > 256;
        aborts_if string::length(utf8(metadata_hash)) > 256;
        aborts_if !string::spec_internal_check_utf8(metadata_location);
        aborts_if !string::spec_internal_check_utf8(metadata_hash);
        aborts_if !string::spec_internal_check_utf8(METADATA_LOCATION_KEY);
        aborts_if !string::spec_internal_check_utf8(METADATA_HASH_KEY);
    }

    /// verify_only
    spec initialize_for_verification(
        diem_framework: &signer,
        min_voting_threshold: u128,
        voting_duration_secs: u64,
    ) {
        pragma verify = false;
    }

    spec resolve_multi_step_proposal(proposal_id: u64, signer_address: address, next_execution_hash: vector<u8>): signer {
        use diem_framework::chain_status;

        // TODO: Executing the prove command gives an error that the target file is in `voting::is_proposal_resolvable`,
        // the level is too deep, it is difficult to obtain the value of `proposal_state`,
        // so it cannot be verified.
        // Can't cover all aborts_if conditions
        pragma aborts_if_is_partial;
        let voting_forum = borrow_global<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        let proposal = table::spec_get(voting_forum.proposals, proposal_id);
        requires chain_status::is_operating();
        aborts_if !exists<voting::VotingForum<GovernanceProposal>>(@diem_framework);
        aborts_if !exists<ApprovedExecutionHashes>(@diem_framework);
        aborts_if !table::spec_contains(voting_forum.proposals,proposal_id);
        aborts_if !string::spec_internal_check_utf8(b"IS_MULTI_STEP_PROPOSAL_IN_EXECUTION");
        aborts_if diem_framework::transaction_context::spec_get_script_hash() != proposal.execution_hash;
        include GetSignerAbortsIf;
    }
}
