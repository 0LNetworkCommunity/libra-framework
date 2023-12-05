
module diem_framework::stake {
    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
    use diem_std::bls12381;
    use diem_std::table::Table;
    use diem_std::comparator;
    use diem_framework::libra_coin::LibraCoin;
    use diem_framework::account;
    use diem_framework::coin::{Coin};
    use diem_framework::event::{Self, EventHandle};
    use diem_framework::system_addresses;
    use ol_framework::slow_wallet;
    use ol_framework::testnet;

    // use diem_std::debug::print;

    friend diem_framework::block;
    friend diem_framework::genesis;
    friend diem_framework::reconfiguration;
    friend diem_framework::transaction_fee;

    //////// 0L ///////
    friend diem_framework::epoch_boundary;
    friend ol_framework::rewards;

    /// Validator Config not published.
    const EVALIDATOR_CONFIG: u64 = 1;
    /// Account is already registered as a validator candidate.
    const EALREADY_REGISTERED: u64 = 8;
    /// Account does not have the right operator capability.
    const ENOT_OPERATOR: u64 = 9;
    /// Invalid consensus public key
    const EINVALID_PUBLIC_KEY: u64 = 11;
    /// Validator set exceeds the limit
    const EVALIDATOR_SET_TOO_LARGE: u64 = 12;
    /// Voting power increase has exceeded the limit for this current epoch.
    const EVOTING_POWER_INCREASE_EXCEEDS_LIMIT: u64 = 13;
    /// Stake pool does not exist at the provided pool address.
    const ESTAKE_POOL_DOES_NOT_EXIST: u64 = 14;
    /// Owner capability does not exist at the provided account.
    const EOWNER_CAP_NOT_FOUND: u64 = 15;
    // const EOWNER_CAP_ALREADY_EXISTS: u64 = 16;
    /// Validator is not defined in the ACL of entities allowed to be validators
    const EINELIGIBLE_VALIDATOR: u64 = 17;
    /// Table to store collected transaction fees for each validator already exists.
    const EFEES_TABLE_ALREADY_EXISTS: u64 = 19;


    /// ALL VALIDATORS HAVE UNIFORM VOTING POWER.
    const UNIFORM_VOTE_POWER: u64 = 1000000;

    /// Validator status enum. We can switch to proper enum later once Move supports it.
    // const VALIDATOR_STATUS_PENDING_ACTIVE: u64 = 1;
    const VALIDATOR_STATUS_ACTIVE: u64 = 2;
    // const VALIDATOR_STATUS_PENDING_INACTIVE: u64 = 3;
    const VALIDATOR_STATUS_INACTIVE: u64 = 4;

    /// Limit the maximum size to u16::max, it's the current limit of the bitvec
    /// https://github.com/diem-labs/diem-core/blob/main/crates/diem-bitvec/src/lib.rs#L20
    const MAX_VALIDATOR_SET_SIZE: u64 = 65536;

    /// Limit the maximum value of `rewards_rate` in order to avoid any arithmetic overflow.
    const MAX_REWARDS_RATE: u64 = 1000000;

    const MAX_U64: u128 = 18446744073709551615;

    /// Capability that represents ownership and can be used to control the validator and the associated stake pool.
    /// Having this be separate from the signer for the account that the validator resources are hosted at allows
    /// modules to have control over a validator.
    struct OwnerCapability has key, store {
        validator_address: address,
    }

    /// Each validator has a separate ValidatorState resource and can provide a stake.
    /// Changes in stake for an active validator:
    /// 1. If a validator calls add_stake, the newly added stake is moved to pending_active.
    /// 2. If validator calls unlock, their stake is moved to pending_inactive.
    /// 2. When the next epoch starts, any pending_inactive stake is moved to inactive and can be withdrawn.
    ///    Any pending_active stake is moved to active and adds to the validator's voting power.
    ///
    /// Changes in stake for an inactive validator:
    /// 1. If a validator calls add_stake, the newly added stake is moved directly to active.
    /// 2. If validator calls unlock, their stake is moved directly to inactive.
    /// 3. When the next epoch starts, the validator can be activated if their active stake is more than the minimum.
    struct ValidatorState has key {
        // Only the account holding OwnerCapability of the staking pool can update this.
        operator_address: address,

        // The events emitted for the entire ValidatorState's lifecycle.
        initialize_validator_events: EventHandle<RegisterValidatorCandidateEvent>,
        set_operator_events: EventHandle<SetOperatorEvent>,
        rotate_consensus_key_events: EventHandle<RotateConsensusKeyEvent>,
        update_network_and_fullnode_addresses_events: EventHandle<UpdateNetworkAndFullnodeAddressesEvent>,
        join_validator_set_events: EventHandle<JoinValidatorSetEvent>,
        distribute_rewards_events: EventHandle<DistributeRewardsEvent>,
        leave_validator_set_events: EventHandle<LeaveValidatorSetEvent>,
    }

    /// Validator info stored in validator address.
    struct ValidatorConfig has key, copy, store, drop {
        consensus_pubkey: vector<u8>,
        network_addresses: vector<u8>,
        // to make it compatible with previous definition, remove later
        fullnode_addresses: vector<u8>,
        // Index in the active set if the validator corresponding to this stake pool is active.
        validator_index: u64,
    }

    /// Consensus information per validator, stored in ValidatorSet.
    struct ValidatorInfo has copy, store, drop {
        addr: address,
        voting_power: u64,
        config: ValidatorConfig,
    }

    // NOTE: 0L things might break with diem-platform if this changes
    /// Full ValidatorSet, stored in @diem_framework.
    /// 1. join_validator_set adds to pending_active queue.
    /// 2. leave_validator_set moves from active to pending_inactive queue.
    /// 3. on_new_epoch processes two pending queues and refresh ValidatorInfo from the owner's address.
    struct ValidatorSet has key {
        consensus_scheme: u8,
        // Active validators for the current epoch.
        active_validators: vector<ValidatorInfo>,
        // Pending validators to leave in next epoch (still active).
        pending_inactive: vector<ValidatorInfo>,
        // Pending validators to join in next epoch.
        pending_active: vector<ValidatorInfo>,
        // Current total voting power.
        total_voting_power: u128,
        // Total voting power waiting to join in the next epoch.
        total_joining_power: u128,
    }

    struct IndividualValidatorPerformance has store, drop {
        successful_proposals: u64,
        failed_proposals: u64,
    }

    struct ValidatorPerformance has key {
        validators: vector<IndividualValidatorPerformance>,
    }

    struct RegisterValidatorCandidateEvent has drop, store {
        validator_address: address,
    }

    struct SetOperatorEvent has drop, store {
        validator_address: address,
        old_operator: address,
        new_operator: address,
    }

    struct RotateConsensusKeyEvent has drop, store {
        validator_address: address,
        old_consensus_pubkey: vector<u8>,
        new_consensus_pubkey: vector<u8>,
    }

    struct UpdateNetworkAndFullnodeAddressesEvent has drop, store {
        validator_address: address,
        old_network_addresses: vector<u8>,
        new_network_addresses: vector<u8>,
        old_fullnode_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    }

    struct JoinValidatorSetEvent has drop, store {
        validator_address: address,
    }

    struct DistributeRewardsEvent has drop, store {
        validator_address: address,
        rewards_amount: u64,
    }


    struct LeaveValidatorSetEvent has drop, store {
        validator_address: address,
    }

    /// Stores transaction fees assigned to validators. All fees are distributed to validators
    /// at the end of the epoch.
    struct ValidatorFees has key {
        fees_table: Table<address, Coin<LibraCoin>>,
    }

    #[view]
    /// @return: tuple
    /// - u64: number of proposals
    /// - address: the validator
    public fun get_highest_net_proposer(): (u64, address) acquires ValidatorSet,
    ValidatorPerformance, ValidatorConfig
    {
      let vals = get_current_validators();
      let highest_net_proposals = 0;
      let highest_addr = @0x0;
      vector::for_each(vals, |v| {
        let idx = get_validator_index(v);
        let (success, fail) = get_current_epoch_proposal_counts(idx);
        if (success > fail) {
          let net = success - fail;

          if (net > highest_net_proposals) {
            highest_net_proposals = net;
            highest_addr = v;
          }
        }
      });
      (highest_net_proposals, highest_addr)
    }


    #[view]
    /// Returns the list of active validators
    public fun get_current_validators(): vector<address> acquires ValidatorSet {
        let validator_set = borrow_global<ValidatorSet>(@diem_framework);
        let addresses = vector::empty<address>();
        let i = 0;
        while (i < vector::length(&validator_set.active_validators)) {
            let v_info = vector::borrow(&validator_set.active_validators, i);
            vector::push_back(&mut addresses, v_info.addr);
            i = i + 1;
        };
        addresses
    }

    #[view]
    /// helper to check if this address is a current one.
    public fun is_current_val(addr: address):bool acquires ValidatorSet {
      let vals = get_current_validators();
      vector::contains(&vals, &addr)
    }

    #[view]
    /// Returns the validator's state.
    public fun get_validator_state(validator_address: address): u64 acquires ValidatorSet {
        let validator_set = borrow_global<ValidatorSet>(@diem_framework);
        if (option::is_some(&find_validator(&validator_set.active_validators, validator_address))) {
            VALIDATOR_STATUS_ACTIVE
        } else {
            VALIDATOR_STATUS_INACTIVE
        }
    }


    //////// 0L ////////
    #[view]
    /// Returns the validator's state.
    public fun is_valid(addr: address): bool {
      exists<ValidatorConfig>(addr)
    }


    #[view]
    /// Return the operator of the validator at `validator_address`.
    public fun get_operator(validator_address: address): address acquires ValidatorState {
        assert_stake_pool_exists(validator_address);
        borrow_global<ValidatorState>(validator_address).operator_address
    }

    /// Return the pool address in `owner_cap`.
    public fun get_owned_pool_address(owner_cap: &OwnerCapability): address {
        owner_cap.validator_address
    }

    #[view]
    /// Return the validator index for `validator_address`.
    public fun get_validator_index(validator_address: address): u64 acquires ValidatorConfig {
        assert_stake_pool_exists(validator_address);
        borrow_global<ValidatorConfig>(validator_address).validator_index
    }

    #[view]
    /// @return: tuple
    /// - u64:  the number of successful
    /// - u64: and failed proposals for the proposal at the given validator index.
    public fun get_current_epoch_proposal_counts(validator_index: u64): (u64, u64) acquires ValidatorPerformance {
        let validator_performances = &borrow_global<ValidatorPerformance>(@diem_framework).validators;
        let validator_performance = vector::borrow(validator_performances, validator_index);
        (validator_performance.successful_proposals, validator_performance.failed_proposals)
    }

    #[view]
    /// Get net proposals from address
    /// @return:  u64:  the number of net proposals (success - fail)
    public fun get_val_net_proposals(val: address): u64 acquires
    ValidatorPerformance, ValidatorConfig {
        let idx = get_validator_index(val);
        let (proposed, failed) = get_current_epoch_proposal_counts(idx);
        if (proposed > failed) {
          proposed - failed
        } else {
          0
        }
    }

    #[view]
    /// Return the validator's config.
    public fun get_validator_config(validator_address: address): (vector<u8>, vector<u8>, vector<u8>) acquires ValidatorConfig {
        assert_stake_pool_exists(validator_address);
        let validator_config = borrow_global<ValidatorConfig>(validator_address);
        (validator_config.consensus_pubkey, validator_config.network_addresses, validator_config.fullnode_addresses)
    }

    #[view]
    public fun stake_pool_exists(addr: address): bool {
        exists<ValidatorState>(addr)
    }

    /// Initialize validator set to the core resource account.
    public(friend) fun initialize(diem_framework: &signer) {
        system_addresses::assert_diem_framework(diem_framework);

        move_to(diem_framework, ValidatorSet {
            consensus_scheme: 0,
            active_validators: vector::empty(),
            pending_active: vector::empty(),
            pending_inactive: vector::empty(),
            total_voting_power: 1,
            total_joining_power: 1,
        });

        move_to(diem_framework, ValidatorPerformance {
            validators: vector::empty(),
        });
    }

    /// Initialize the validator account and give ownership to the signing account
    /// except it leaves the ValidatorConfig to be set by another entity.
    /// Note: this triggers setting the operator and owner, set it to the account's address
    /// to set later.
    public entry fun initialize_stake_owner(
        owner: &signer,
        initial_stake_amount: u64,
        operator: address,
        _voter: address,
    ) acquires AllowedValidators, OwnerCapability, ValidatorState {
        initialize_owner(owner);
        move_to(owner, ValidatorConfig {
            consensus_pubkey: vector::empty(),
            network_addresses: vector::empty(),
            fullnode_addresses: vector::empty(),
            validator_index: 0,
        });

        if (initial_stake_amount > 0) {
            // add_stake(owner, initial_stake_amount);
        };

        let account_address = signer::address_of(owner);
        if (account_address != operator) {
            set_operator(owner, operator)
        };
    }

    /// Initialize the validator account and give ownership to the signing account.
    public entry fun initialize_validator(
        account: &signer,
        consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
        network_addresses: vector<u8>,
        fullnode_addresses: vector<u8>,
    ) acquires AllowedValidators {
        // Checks the public key has a valid proof-of-possession to prevent rogue-key attacks.
        let pubkey_from_pop = &mut bls12381::public_key_from_bytes_with_pop(
            consensus_pubkey,
            &bls12381::proof_of_possession_from_bytes(proof_of_possession)
        );
        assert!(option::is_some(pubkey_from_pop), error::invalid_argument(EINVALID_PUBLIC_KEY));

        initialize_owner(account);

        move_to(account, ValidatorConfig {
            consensus_pubkey,
            network_addresses,
            fullnode_addresses,
            validator_index: 0,
        });

        slow_wallet::set_slow(account);
    }

    fun initialize_owner(owner: &signer) acquires AllowedValidators {
        let owner_address = signer::address_of(owner);
        assert!(is_allowed(owner_address), error::not_found(EINELIGIBLE_VALIDATOR));
        assert!(!stake_pool_exists(owner_address), error::already_exists(EALREADY_REGISTERED));

        move_to(owner, ValidatorState {
            operator_address: owner_address,
            // Events.
            initialize_validator_events: account::new_event_handle<RegisterValidatorCandidateEvent>(owner),
            set_operator_events: account::new_event_handle<SetOperatorEvent>(owner),

            rotate_consensus_key_events: account::new_event_handle<RotateConsensusKeyEvent>(owner),
            update_network_and_fullnode_addresses_events: account::new_event_handle<UpdateNetworkAndFullnodeAddressesEvent>(owner),
            join_validator_set_events: account::new_event_handle<JoinValidatorSetEvent>(owner),
            distribute_rewards_events: account::new_event_handle<DistributeRewardsEvent>(owner),
            leave_validator_set_events: account::new_event_handle<LeaveValidatorSetEvent>(owner),
        });

    }

    /// Allows an owner to change the operator of the stake pool.
    public entry fun set_operator(owner: &signer, new_operator: address) acquires OwnerCapability, ValidatorState {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        set_operator_with_cap(ownership_cap, new_operator);
    }

    /// Allows an account with ownership capability to change the operator of the stake pool.
    public fun set_operator_with_cap(owner_cap: &OwnerCapability, new_operator: address) acquires ValidatorState {
        let validator_address = owner_cap.validator_address;
        assert_stake_pool_exists(validator_address);
        let stake_pool = borrow_global_mut<ValidatorState>(validator_address);
        let old_operator = stake_pool.operator_address;
        stake_pool.operator_address = new_operator;

        event::emit_event(
            &mut stake_pool.set_operator_events,
            SetOperatorEvent {
                validator_address,
                old_operator,
                new_operator,
            },
        );
    }


    /// Rotate the consensus key of the validator, it'll take effect in next epoch.
    public entry fun rotate_consensus_key(
        operator: &signer,
        validator_address: address,
        new_consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
    ) acquires ValidatorState, ValidatorConfig {
        assert_stake_pool_exists(validator_address);
        let stake_pool = borrow_global_mut<ValidatorState>(validator_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        assert!(exists<ValidatorConfig>(validator_address), error::not_found(EVALIDATOR_CONFIG));
        let validator_info = borrow_global_mut<ValidatorConfig>(validator_address);
        let old_consensus_pubkey = validator_info.consensus_pubkey;
        // Checks the public key has a valid proof-of-possession to prevent rogue-key attacks.
        let pubkey_from_pop = &mut bls12381::public_key_from_bytes_with_pop(
            new_consensus_pubkey,
            &bls12381::proof_of_possession_from_bytes(proof_of_possession)
        );
        assert!(option::is_some(pubkey_from_pop), error::invalid_argument(EINVALID_PUBLIC_KEY));
        validator_info.consensus_pubkey = new_consensus_pubkey;

        event::emit_event(
            &mut stake_pool.rotate_consensus_key_events,
            RotateConsensusKeyEvent {
                validator_address,
                old_consensus_pubkey,
                new_consensus_pubkey,
            },
        );
    }

    /// Update the network and full node addresses of the validator. This only takes effect in the next epoch.
    public entry fun update_network_and_fullnode_addresses(
        operator: &signer,
        validator_address: address,
        new_network_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    ) acquires ValidatorState, ValidatorConfig {
        assert_stake_pool_exists(validator_address);
        let stake_pool = borrow_global_mut<ValidatorState>(validator_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        assert!(exists<ValidatorConfig>(validator_address), error::not_found(EVALIDATOR_CONFIG));
        let validator_info = borrow_global_mut<ValidatorConfig>(validator_address);
        let old_network_addresses = validator_info.network_addresses;
        validator_info.network_addresses = new_network_addresses;
        let old_fullnode_addresses = validator_info.fullnode_addresses;
        validator_info.fullnode_addresses = new_fullnode_addresses;

        event::emit_event(
            &mut stake_pool.update_network_and_fullnode_addresses_events,
            UpdateNetworkAndFullnodeAddressesEvent {
                validator_address,
                old_network_addresses,
                new_network_addresses,
                old_fullnode_addresses,
                new_fullnode_addresses,
            },
        );
    }


    // #[test_only]
    /// Request to have `validator_address` join the validator set. Can only be called after calling `initialize_validator`.
    /// If the validator has the required stake (more than minimum and less than maximum allowed), they will be
    /// added to the pending_active queue. All validators in this queue will be added to the active set when the next
    /// epoch starts (eligibility will be rechecked).
    ///
    /// This internal version can only be called by the Genesis module during
    /// Genesis.
    public(friend) fun join_validator_set(
        operator: &signer,
        validator_address: address
    ) acquires ValidatorState, ValidatorConfig, ValidatorSet {

        assert_stake_pool_exists(validator_address);
        let stake_pool = borrow_global_mut<ValidatorState>(validator_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        // Add validator to pending_active, to be activated in the next epoch.
        let validator_config = borrow_global_mut<ValidatorConfig>(validator_address);
        assert!(!vector::is_empty(&validator_config.consensus_pubkey), error::invalid_argument(EINVALID_PUBLIC_KEY));

        // Validate the current validator set size has not exceeded the limit.
        let validator_set = borrow_global_mut<ValidatorSet>(@diem_framework);
        vector::push_back(&mut validator_set.active_validators, generate_validator_info(validator_address, *validator_config));

        let validator_set_size = vector::length(&validator_set.active_validators);
        assert!(validator_set_size <= MAX_VALIDATOR_SET_SIZE, error::invalid_argument(EVALIDATOR_SET_TOO_LARGE));

        event::emit_event(
            &mut stake_pool.join_validator_set_events,
            JoinValidatorSetEvent { validator_address },
        );
    }


    /// Returns true if the current validator can still vote in the current epoch.
    /// This includes validators that requested to leave but are still in the pending_inactive queue and will be removed
    /// when the epoch starts.
    public fun is_current_epoch_validator(addr: address): bool  acquires ValidatorSet{
      get_validator_state(addr) == VALIDATOR_STATUS_ACTIVE
    }

    /// Update the validator performance (proposal statistics). This is only called by block::prologue().
    /// This function cannot abort.
    public(friend) fun update_performance_statistics(proposer_index: Option<u64>, failed_proposer_indices: vector<u64>) acquires ValidatorPerformance {
        // Validator set cannot change until the end of the epoch, so the validator index in arguments should
        // match with those of the validators in ValidatorPerformance resource.
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@diem_framework);
        let validator_len = vector::length(&validator_perf.validators);

        // proposer_index is an option because it can be missing (for NilBlocks)
        if (option::is_some(&proposer_index)) {
            let cur_proposer_index = option::extract(&mut proposer_index);
            // Here, and in all other vector::borrow, skip any validator indices that are out of bounds,
            // this ensures that this function doesn't abort if there are out of bounds errors.
            if (cur_proposer_index < validator_len) {
                let validator = vector::borrow_mut(&mut validator_perf.validators, cur_proposer_index);
                spec {
                    assume validator.successful_proposals + 1 <= MAX_U64;
                };
                validator.successful_proposals = validator.successful_proposals + 1;
            };
        };

        let f = 0;
        let f_len = vector::length(&failed_proposer_indices);
        while ({
            spec {
                invariant len(validator_perf.validators) == validator_len;
            };
            f < f_len
        }) {
            let validator_index = *vector::borrow(&failed_proposer_indices, f);
            if (validator_index < validator_len) {
                let validator = vector::borrow_mut(&mut validator_perf.validators, validator_index);
                spec {
                    assume validator.failed_proposals + 1 <= MAX_U64;
                };
                validator.failed_proposals = validator.failed_proposals + 1;
            };
            f = f + 1;
        };
    }

    /// Triggers at epoch boundary. This function shouldn't abort.
    /// NOTE: THIS ONLY EXISTS FOR VENDOR TESTS
    public(friend) fun on_new_epoch() acquires ValidatorConfig, ValidatorPerformance, ValidatorSet {
        let validator_set = borrow_global_mut<ValidatorSet>(@diem_framework);
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@diem_framework);


        // Update active validator set so that network address/public key change takes effect.
        // Moreover, recalculate the total voting power, and deactivate the validator whose
        // voting power is less than the minimum required stake.
        let next_epoch_validators = vector::empty();
        let minimum_stake = 0;

        let vlen = vector::length(&validator_set.active_validators);
        let total_voting_power = 0;
        let i = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(next_epoch_validators);
            };
            i < vlen
        }) {
            let old_validator_info = vector::borrow_mut(&mut validator_set.active_validators, i);
            let validator_address = old_validator_info.addr;
            let validator_config = borrow_global_mut<ValidatorConfig>(validator_address);

            let new_validator_info = generate_validator_info(validator_address, *validator_config);

            // A validator needs at least the min stake required to join the validator set.
            if (new_validator_info.voting_power >= minimum_stake) {
                spec {
                    assume total_voting_power + new_validator_info.voting_power <= MAX_U128;
                };
                total_voting_power = total_voting_power + (new_validator_info.voting_power as u128);
                vector::push_back(&mut next_epoch_validators, new_validator_info);
            };
            i = i + 1;
        };


        validator_set.active_validators = next_epoch_validators;
        validator_set.total_voting_power = total_voting_power;


        // Update validator indices, reset performance scores, and renew lockups.
        validator_perf.validators = vector::empty();

        let vlen = vector::length(&validator_set.active_validators);
        let validator_index = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(validator_set.active_validators);
                // invariant len(validator_set.pending_active) == 0;
                // invariant len(validator_set.pending_inactive) == 0;
                invariant 0 <= validator_index && validator_index <= vlen;
                invariant vlen == len(validator_set.active_validators);
                invariant forall i in 0..validator_index:
                    global<ValidatorConfig>(validator_set.active_validators[i].addr).validator_index < validator_index;
                invariant len(validator_perf.validators) == validator_index;
            };
            validator_index < vlen
        }) {

            // Update validator index.
            let validator_info = vector::borrow_mut(&mut validator_set.active_validators, validator_index);
            validator_info.config.validator_index = validator_index;
            let validator_config = borrow_global_mut<ValidatorConfig>(validator_info.addr);
            validator_config.validator_index = validator_index;

            // reset performance scores.
            vector::push_back(&mut validator_perf.validators, IndividualValidatorPerformance {
                successful_proposals: 0,
                failed_proposals: 0,
            });


            validator_index = validator_index + 1;
        };

    }

    /// DANGER: belt and suspenders critical mutations
    /// Called on epoch boundary to reconfigure
    /// No change may happen due to failover rules.
    /// Returns instrumentation for audits: if what the validator set was, which validators qualified after failover rules, a list of validators which had missing configs and were excluded, if the new list sucessfully matches the actual validators after reconfiguration(actual_validator_set, qualified_on_failover, missing_configs, success)
    public(friend) fun maybe_reconfigure(root: &signer, proposed_validators:
    vector<address>): (vector<address>, vector<address>, bool) acquires
    ValidatorConfig, ValidatorPerformance, ValidatorSet {


        // NOTE: ol does not use the pending, and pending inactive lists.
        // DANGER: critical mutation: belt and suspenders

        // we attempt to change the validator set
        // it is no guaranteed that it will change
        // we check our "failover rules" here
        // which establishes minimum amount of vals for a viable network

        //////// RECONFIGURE ////////

        let (list_info, _voting_power, missing_configs) = make_validator_set_config(&proposed_validators);
        // mutations happen in private function
        bulk_set_next_validators(root, list_info);

        let validator_set = borrow_global_mut<ValidatorSet>(@diem_framework);
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@diem_framework);

        let validator_set_sanity = vector::empty<address>();


        ///// UPDATE VALIDATOR PERFORMANCE

        validator_perf.validators = vector::empty();


        let vlen = vector::length(&proposed_validators);
        let validator_index = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(validator_set.active_validators);
                // invariant len(validator_set.pending_active) == 0;
                // invariant len(validator_set.pending_inactive) == 0;
                invariant 0 <= validator_index && validator_index <= vlen;
                invariant vlen == len(validator_set.active_validators);
                invariant forall i in 0..validator_index:
                    global<ValidatorConfig>(validator_set.active_validators[i].addr).validator_index < validator_index;
                invariant len(validator_perf.validators) == validator_index;
            };
            validator_index < vlen
        }) {

            // Update validator index.
            let validator_info = vector::borrow_mut(&mut validator_set.active_validators, validator_index);
            validator_info.config.validator_index = validator_index;
            let validator_config = borrow_global_mut<ValidatorConfig>(validator_info.addr);

            vector::push_back(&mut validator_set_sanity, validator_info.addr);
            validator_config.validator_index = validator_index;

            // reset performance scores.
            vector::push_back(&mut validator_perf.validators, IndividualValidatorPerformance {
                successful_proposals: 0,
                failed_proposals: 0,
            });

            validator_index = validator_index + 1;
        };

      let finally_the_validators = get_current_validators();
      let success_sanity = comparator::is_equal(
          &comparator::compare(&proposed_validators, &validator_set_sanity)
      );
      let success_current = comparator::is_equal(
        &comparator::compare(&proposed_validators, &finally_the_validators)
      );

      (finally_the_validators, missing_configs, success_sanity && success_current)
    }

    #[test_only]
    public fun test_make_val_cfg(list: &vector<address>): (vector<ValidatorInfo>, u128, vector<address>) acquires ValidatorConfig {
      make_validator_set_config(list)
    }

    /// Make the active validators list
    /// returns: the list of validators, and the total voting power (1 per validator), and also the list of validators that did not have valid configs
    fun make_validator_set_config(list: &vector<address>): (vector<ValidatorInfo>, u128, vector<address>) acquires ValidatorConfig  {

      let next_epoch_validators = vector::empty();
      let vlen = vector::length(list);

      let missing_configs = vector::empty();

      let total_voting_power = 0;
      let i = 0;
      while ({
          spec {
              invariant spec_validators_are_initialized(next_epoch_validators);
          };
          i < vlen
      }) {
          // let old_validator_info = vector::borrow_mut(&mut validator_set.active_validators, i);

          let validator_address = *vector::borrow(list, i);


          if (!exists<ValidatorConfig>(validator_address)) {
            vector::push_back(&mut missing_configs, validator_address);
            i = i + 1;
            continue
          }; // belt and suspenders

          let validator_config = borrow_global_mut<ValidatorConfig>(validator_address);

          if (!exists<ValidatorState>(validator_address)) {
            vector::push_back(&mut missing_configs, validator_address);
            i = i + 1;
            continue
          }; // belt and suspenders


          let new_validator_info = generate_validator_info(validator_address, *validator_config);

          // all validators have same weight
          total_voting_power = total_voting_power + 1;

          vector::push_back(&mut next_epoch_validators, new_validator_info);
          // A validator needs at least the min stake required to join the validator set.

          // TODO: check this
          // spec {
          //     assume total_voting_power + 1<= MAX_U128;
          // };

          i = i + 1;
      };

      (next_epoch_validators, total_voting_power, missing_configs)
    }

    #[test_only]
    public fun test_set_next_vals(root: &signer, list: vector<ValidatorInfo>) acquires ValidatorSet {
      system_addresses::assert_ol(root);
      bulk_set_next_validators(root, list);
    }

    /// sets the global state for the next epoch validators
    /// belt and suspenders, private function is authorized. Test functions also authorized.
    fun bulk_set_next_validators(root: &signer, list: vector<ValidatorInfo>) acquires ValidatorSet {
      system_addresses::assert_ol(root);

      let validator_set = borrow_global_mut<ValidatorSet>(@diem_framework);
      validator_set.active_validators = list;
      validator_set.total_voting_power = (vector::length(&list) as u128);
    }

    //////// Failover Rules ////////
    // If the cardinality of proposed validators set (as chosen by Musical
    // Chairs and Proof of Fee) in the upcoming is less than a
    // healthy threshold (9) we should proactively increase the validator set.
    // The best and the worse case are fairly straightforward to address.
    // A) The Happy Case. simply returns the proposed set, since it is above
    // the healthy threshold.
    // B) Sick Patient. The proposed qualifying set is below the threshold (9).
    // We should expand the set. IF the algorithm prior is returning a
    // list of the size (or, unlikely, larger) than the Musical Chairs choice,
    // use all of the proposed validtors.
    // C) Defibrillator. The proposed qualifying set is both below healthy, and
    // belowr what MusicalChairs thought was appropriate. We should expand the
    // set conservatively since adding any validator who did not qualify or win
    // the auction can be in an unknown state (most likely unresponsive, rather
    // than malicious). Arguably adding unreponsive nodes is better than adding
    // none at this stage. But we can minimize those chances by taking the
    // highest performing validators (by net approved proposals), and filling it
    // only as much as needed to either get to the MusicalChairs number or healthy threshold.
    // After both of the attempts (B, C) above, in the case of certain death,
    // where the proposed set is still less than 4 (D: Hail Mary), we should
    // not change the validator set.

    public fun check_failover_rules(proposed: vector<address>, seats_offered: u64): vector<address> acquires ValidatorSet,
   ValidatorConfig, ValidatorPerformance {
        let healthy_threshold = 9;
        let minimum_seats = 4;

        if (seats_offered < minimum_seats) { seats_offered = minimum_seats };
        // let min_f = 3;
        let current_vals = get_current_validators();
        // check if this is not test. Failover only when there are no validators proposed
        if (testnet::is_testnet()) {
          if (vector::length(&proposed) == 0) {
            return current_vals
          };
          return proposed
        };

        let count_proposed = vector::length(&proposed);
        // did we get at lest 9 qualified validators to join set?
        let enough_qualified = count_proposed > healthy_threshold;

        // expand the least amount, otherwise we could halt again
        // with unprepared validtors.
        let new_target = if (seats_offered > healthy_threshold)
        healthy_threshold else seats_offered;
        // // do we have 10 or more performant validators (whether or not they
        // // qualified for next epoch)?
        // let sufficient_performance = vector::length(&performant) > 9;

        // Scenario A) Happy Case
        // not near failure
        if (enough_qualified) {
          return proposed
        } else if (new_target >= 4) {
          // we won't dig out of the hole if we target less than 4
          // Scenario B) Sick Patient
          // always fill seats with proposed/qualified if possible.
          // if not, go to hail mary
          if (count_proposed >= new_target) { // should not be higher, most
          // likely equal
            return proposed
          } else {
          // Scenario C) Defibrillator
          // we are below threshold, and neither can get to the number
          // of seats offered. If we don't expand the set, we will likely fail
          // lets expand the set even with people that did not win the auction
          // or otherwise qualify, but may have been performant in the last epoch


            // fill remaining seats with
            // take the most performant validators from previous epoch.
            let remainder = new_target - count_proposed;
            let ranked = get_sorted_vals_by_net_props();
            let i = 0;
            while (i < remainder) {
              let best_val = vector::borrow(&ranked, i);
              if (!vector::contains(&proposed, best_val)) {
                vector::push_back(&mut proposed, *best_val)
              };
              i = i + 1;
            }
          }
        };

        // Scenario D) Hail Mary
        // if at the end of all this we still don't have 4 validators
        // then just return the same validator set.
        if (vector::length(&proposed) > 3) {
          return proposed
        };
        // this is unreachable but as a backstop for dev finge
        current_vals
    }

    /// Bubble sort the validators by their proposal counts.
    public fun get_sorted_vals_by_net_props(): vector<address> acquires ValidatorSet, ValidatorConfig, ValidatorPerformance {
      let eligible_validators = get_current_validators();
      let length = vector::length<address>(&eligible_validators);

      // vector to store each address's node_weight
      let weights = vector::empty<u64>();
      let filtered_vals = vector::empty<address>();
      let k = 0;
      while (k < length) {
        // TODO: Ensure that this address is an active validator

        let cur_address = *vector::borrow<address>(&eligible_validators, k);

        let idx = get_validator_index(cur_address);
        let (proposed, failed) = get_current_epoch_proposal_counts(idx);
        let delta = 0;
        if (proposed > failed) {
          delta = proposed - failed;
        };

        vector::push_back<u64>(&mut weights, delta);
        vector::push_back<address>(&mut filtered_vals, cur_address);
        k = k + 1;
      };

      // Sorting the accounts vector based on value (weights).
      // Bubble sort algorithm
      let len_filtered = vector::length<address>(&filtered_vals);
      if (len_filtered < 2) return filtered_vals;
      let i = 0;
      while (i < len_filtered){
        let j = 0;
        while(j < len_filtered-i-1){

          let value_j = *(vector::borrow<u64>(&weights, j));

          let value_jp1 = *(vector::borrow<u64>(&weights, j+1));
          if(value_j > value_jp1){

            vector::swap<u64>(&mut weights, j, j+1);

            vector::swap<address>(&mut filtered_vals, j, j+1);
          };
          j = j + 1;

        };
        i = i + 1;

      };


      // Reverse to have sorted order - high to low.
      vector::reverse<address>(&mut filtered_vals);

      // // Return the top n validators
      // let final = vector::empty<address>();
      // let m = 0;
      // while ( m < n) {
      //    vector::push_back(&mut final, *vector::borrow(&filtered_vals, m));
      //    m = m + 1;
      // };

      return filtered_vals

    }

    //////// 0L ////////
    // since rewards are handled externally to stake.move we need an api to emit the event
    public(friend) fun emit_distribute_reward(root: &signer, validator_address: address, rewards_amount: u64) acquires ValidatorState {
        system_addresses::assert_ol(root);
        let stake_pool = borrow_global_mut<ValidatorState>(validator_address);
        event::emit_event(
          &mut stake_pool.distribute_rewards_events,
          DistributeRewardsEvent {
              validator_address,
              rewards_amount,
          },
      );
    }

    fun find_validator(v: &vector<ValidatorInfo>, addr: address): Option<u64> {
        let i = 0;
        let len = vector::length(v);
        while ({
            spec {
                invariant !(exists j in 0..i: v[j].addr == addr);
            };
            i < len
        }) {
            if (vector::borrow(v, i).addr == addr) {
                return option::some(i)
            };
            i = i + 1;
        };
        option::none()
    }

    fun generate_validator_info(addr: address, config: ValidatorConfig): ValidatorInfo {
        ValidatorInfo {
            addr,
            voting_power: UNIFORM_VOTE_POWER,
            config,
        }
    }


    fun assert_stake_pool_exists(validator_address: address) {
        assert!(stake_pool_exists(validator_address), error::invalid_argument(ESTAKE_POOL_DOES_NOT_EXIST));
    }

    //////////// TESTNET ////////////

    /// This provides an ACL for Testnet purposes. In testnet, everyone is a whale, a whale can be a validator.
    /// This allows a testnet to bring additional entities into the validator set without compromising the
    /// security of the testnet. This will NOT be enabled in Mainnet.
    struct AllowedValidators has key {
        accounts: vector<address>,
    }

    public fun configure_allowed_validators(diem_framework: &signer, accounts: vector<address>) acquires AllowedValidators {
        let diem_framework_address = signer::address_of(diem_framework);
        system_addresses::assert_diem_framework(diem_framework);
        if (!exists<AllowedValidators>(diem_framework_address)) {
            move_to(diem_framework, AllowedValidators { accounts });
        } else {
            let allowed = borrow_global_mut<AllowedValidators>(diem_framework_address);
            allowed.accounts = accounts;
        }
    }

    fun is_allowed(account: address): bool acquires AllowedValidators {
        if (!exists<AllowedValidators>(@diem_framework)) {
            true
        } else {
            let allowed = borrow_global<AllowedValidators>(@diem_framework);
            vector::contains(&allowed.accounts, &account)
        }
    }
    // TODO: v7 - remove this

    fun assert_owner_cap_exists(owner: address) {
        assert!(exists<OwnerCapability>(owner), error::not_found(EOWNER_CAP_NOT_FOUND));
    }


    #[test_only]
    const EPOCH_DURATION: u64 = 60;

    #[test_only]
    public fun mock_performance(vm: &signer, addr: address, success: u64, fail: u64) acquires ValidatorConfig, ValidatorPerformance  {
      system_addresses::assert_ol(vm);

      let idx = get_validator_index(addr);

      let perf = borrow_global_mut<ValidatorPerformance>(@diem_framework);

      let p = vector::borrow_mut(&mut perf.validators, idx);
      p.successful_proposals = success;
      p.failed_proposals = fail;
    }

    #[test_only]
    public fun generate_identity(): (bls12381::SecretKey, bls12381::PublicKey, bls12381::ProofOfPossession) {
        let (sk, pkpop) = bls12381::generate_keys();
        let pop = bls12381::generate_proof_of_possession(&sk);
        let unvalidated_pk = bls12381::public_key_with_pop_to_normal(&pkpop);
        (sk, unvalidated_pk, pop)
    }

    #[test_only]
    public fun end_epoch() acquires ValidatorPerformance, ValidatorConfig, ValidatorSet {
        // Set the number of blocks to 1, to give out rewards to non-failing validators.
        set_validator_perf_at_least_one_block();
        timestamp::fast_forward_seconds(EPOCH_DURATION);
        on_new_epoch();
    }



    #[test_only]
    public fun test_reconfigure(root: &signer, vals: vector<address>) acquires
    ValidatorConfig, ValidatorPerformance, ValidatorSet {
      maybe_reconfigure(root, vals);
    }

    #[test_only]
    public fun set_validator_perf_at_least_one_block() acquires ValidatorPerformance {
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@diem_framework);
        let len = vector::length(&validator_perf.validators);
        let i = 0;
        while (i < len) {
            let validator = vector::borrow_mut(&mut validator_perf.validators, i);
            if (validator.successful_proposals + validator.failed_proposals < 1) {
                validator.successful_proposals = 1;
            };
            i = i + 1;
        };
    }

    #[test_only]
    public fun initialize_test_validator(
        root: &signer,
        public_key: &bls12381::PublicKey,
        proof_of_possession: &bls12381::ProofOfPossession,
        validator: &signer,
        _amount: u64,
        should_join_validator_set: bool,
        should_end_epoch: bool,
    ) acquires AllowedValidators, ValidatorState, ValidatorConfig, ValidatorSet,
   ValidatorPerformance {
        use ol_framework::ol_account;
        system_addresses::assert_ol(root);
        let validator_address = signer::address_of(validator);
        if (!account::exists_at(signer::address_of(validator))) {
            ol_account::create_account(root, validator_address);
        };

        let pk_bytes = bls12381::public_key_to_bytes(public_key);
        let pop_bytes = bls12381::proof_of_possession_to_bytes(proof_of_possession);
        initialize_validator(validator, pk_bytes, pop_bytes, vector::empty(), vector::empty());

        // if (amount > 0) {
        //     mint_and_add_stake(validator, amount);
        // };

        if (should_join_validator_set) {
            join_validator_set(validator, validator_address);
        };
        if (should_end_epoch) {
            end_epoch();
        };
    }

    #[test_only]
    /// One step setup for tests
    public fun quick_init(root: &signer, val_sig: &signer) acquires
    ValidatorSet, ValidatorState, ValidatorConfig, AllowedValidators,
    ValidatorPerformance {
      system_addresses::assert_ol(root);
      let (_sk, pk, pop) = generate_identity();
      initialize_test_validator(root, &pk, &pop, val_sig, 100, true, true);
    }

    #[test_only]
    public fun get_reward_event_guid(val: address): u64 acquires ValidatorState{
      let sp = borrow_global<ValidatorState>(val);
      event::counter(&sp.distribute_rewards_events)
    }
}
