/**
 * Validator lifecycle:
 * 1. Prepare a validator node set up and call stake::initialize_validator
 * 2. Once ready to deposit stake (or have funds assigned by a staking service in exchange for ownership capability),
 * call stake::add_stake (or *_with_cap versions if called from the staking service)
 * 3. Call stake::join_validator_set (or _with_cap version) to join the active validator set. Changes are effective in
 * the next epoch.
 * 4. Validate and gain rewards. The stake will automatically be locked up for a fixed duration (set by governance) and
 * automatically renewed at expiration.
 * 5. At any point, if the validator operator wants to update the consensus key or network/fullnode addresses, they can
 * call stake::rotate_consensus_key and stake::update_network_and_fullnode_addresses. Similar to changes to stake, the
 * changes to consensus key/network/fullnode addresses are only effective in the next epoch.
 * 6. Validator can request to unlock their stake at any time. However, their stake will only become withdrawable when
 * their current lockup expires. This can be at most as long as the fixed lockup duration.
 * 7. After exiting, the validator can either explicitly leave the validator set by calling stake::leave_validator_set
 * or if their stake drops below the min required, they would get removed at the end of the epoch.
 * 8. Validator can always rejoin the validator set by going through steps 2-3 again.
 * 9. An owner can always switch operators by calling stake::set_operator.
 * 10. An owner can always switch designated voter by calling stake::set_designated_voter.
*/
module aptos_framework::stake {
    use std::error;
    use std::features;
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
    use aptos_std::bls12381;
    // use aptos_std::math64::min;
    use aptos_std::table::{Self, Table};
    // use aptos_std::debug::print;

    use ol_framework::slow_wallet;
    
    use ol_framework::testnet;
    
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::account;
    use aptos_framework::coin::{Self, Coin, MintCapability};
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_framework::system_addresses;
    // use aptos_framework::staking_config::{Self, StakingConfig, StakingRewardsConfig};
    use aptos_framework::chain_status;

    // use aptos_framework::validator_universe;

    friend aptos_framework::block;
    friend aptos_framework::genesis;
    friend aptos_framework::reconfiguration;
    friend aptos_framework::transaction_fee;

    //////// 0L ///////
    friend aptos_framework::epoch_boundary;


    /// Validator Config not published.
    const EVALIDATOR_CONFIG: u64 = 1;
    /// Not enough stake to join validator set.
    const ESTAKE_TOO_LOW: u64 = 2;
    /// Too much stake to join validator set.
    const ESTAKE_TOO_HIGH: u64 = 3;
    /// Account is already a validator or pending validator.
    const EALREADY_ACTIVE_VALIDATOR: u64 = 4;
    /// Account is not a validator.
    const ENOT_VALIDATOR: u64 = 5;
    /// Can't remove last validator.
    const ELAST_VALIDATOR: u64 = 6;
    /// Total stake exceeds maximum allowed.
    const ESTAKE_EXCEEDS_MAX: u64 = 7;
    /// Account is already registered as a validator candidate.
    const EALREADY_REGISTERED: u64 = 8;
    /// Account does not have the right operator capability.
    const ENOT_OPERATOR: u64 = 9;
    /// Validators cannot join or leave post genesis on this test network.
    const ENO_POST_GENESIS_VALIDATOR_SET_CHANGE_ALLOWED: u64 = 10;
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
    /// An account cannot own more than one owner capability.
    const EOWNER_CAP_ALREADY_EXISTS: u64 = 16;
    /// Validator is not defined in the ACL of entities allowed to be validators
    const EINELIGIBLE_VALIDATOR: u64 = 17;
    /// Cannot update stake pool's lockup to earlier than current lockup.
    const EINVALID_LOCKUP: u64 = 18;
    /// Table to store collected transaction fees for each validator already exists.
    const EFEES_TABLE_ALREADY_EXISTS: u64 = 19;

    /// Validator status enum. We can switch to proper enum later once Move supports it.
    const VALIDATOR_STATUS_PENDING_ACTIVE: u64 = 1;
    const VALIDATOR_STATUS_ACTIVE: u64 = 2;
    const VALIDATOR_STATUS_PENDING_INACTIVE: u64 = 3;
    const VALIDATOR_STATUS_INACTIVE: u64 = 4;

    /// Limit the maximum size to u16::max, it's the current limit of the bitvec
    /// https://github.com/aptos-labs/aptos-core/blob/main/crates/aptos-bitvec/src/lib.rs#L20
    const MAX_VALIDATOR_SET_SIZE: u64 = 65536;

    /// Limit the maximum value of `rewards_rate` in order to avoid any arithmetic overflow.
    const MAX_REWARDS_RATE: u64 = 1000000;

    const MAX_U64: u128 = 18446744073709551615;

    /// Capability that represents ownership and can be used to control the validator and the associated stake pool.
    /// Having this be separate from the signer for the account that the validator resources are hosted at allows
    /// modules to have control over a validator.
    struct OwnerCapability has key, store {
        pool_address: address,
    }

    /// Each validator has a separate StakePool resource and can provide a stake.
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
    struct StakePool has key {
        // active stake
        active: Coin<AptosCoin>,
        // inactive stake, can be withdrawn
        inactive: Coin<AptosCoin>,
        // pending activation for next epoch
        pending_active: Coin<AptosCoin>,
        // pending deactivation for next epoch
        pending_inactive: Coin<AptosCoin>,
        locked_until_secs: u64,
        // Track the current operator of the validator node.
        // This allows the operator to be different from the original account and allow for separation of
        // the validator operations and ownership.
        // Only the account holding OwnerCapability of the staking pool can update this.
        operator_address: address,

        // Track the current vote delegator of the staking pool.
        // Only the account holding OwnerCapability of the staking pool can update this.
        delegated_voter: address,

        // The events emitted for the entire StakePool's lifecycle.
        initialize_validator_events: EventHandle<RegisterValidatorCandidateEvent>,
        set_operator_events: EventHandle<SetOperatorEvent>,
        add_stake_events: EventHandle<AddStakeEvent>,
        reactivate_stake_events: EventHandle<ReactivateStakeEvent>,
        rotate_consensus_key_events: EventHandle<RotateConsensusKeyEvent>,
        update_network_and_fullnode_addresses_events: EventHandle<UpdateNetworkAndFullnodeAddressesEvent>,
        increase_lockup_events: EventHandle<IncreaseLockupEvent>,
        join_validator_set_events: EventHandle<JoinValidatorSetEvent>,
        distribute_rewards_events: EventHandle<DistributeRewardsEvent>,
        unlock_stake_events: EventHandle<UnlockStakeEvent>,
        withdraw_stake_events: EventHandle<WithdrawStakeEvent>,
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

    /// Full ValidatorSet, stored in @aptos_framework.
    /// 1. join_validator_set adds to pending_active queue.
    /// 2. leave_valdiator_set moves from active to pending_inactive queue.
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

    /// AptosCoin capabilities, set during genesis and stored in @CoreResource account.
    /// This allows the Stake module to mint rewards to stakers.
    struct AptosCoinCapabilities has key {
        mint_cap: MintCapability<AptosCoin>,
    }

    struct IndividualValidatorPerformance has store, drop {
        successful_proposals: u64,
        failed_proposals: u64,
    }

    struct ValidatorPerformance has key {
        validators: vector<IndividualValidatorPerformance>,
    }

    struct RegisterValidatorCandidateEvent has drop, store {
        pool_address: address,
    }

    struct SetOperatorEvent has drop, store {
        pool_address: address,
        old_operator: address,
        new_operator: address,
    }

    struct AddStakeEvent has drop, store {
        pool_address: address,
        amount_added: u64,
    }

    struct ReactivateStakeEvent has drop, store {
        pool_address: address,
        amount: u64,
    }

    struct RotateConsensusKeyEvent has drop, store {
        pool_address: address,
        old_consensus_pubkey: vector<u8>,
        new_consensus_pubkey: vector<u8>,
    }

    struct UpdateNetworkAndFullnodeAddressesEvent has drop, store {
        pool_address: address,
        old_network_addresses: vector<u8>,
        new_network_addresses: vector<u8>,
        old_fullnode_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    }

    struct IncreaseLockupEvent has drop, store {
        pool_address: address,
        old_locked_until_secs: u64,
        new_locked_until_secs: u64,
    }

    struct JoinValidatorSetEvent has drop, store {
        pool_address: address,
    }

    struct DistributeRewardsEvent has drop, store {
        pool_address: address,
        rewards_amount: u64,
    }

    struct UnlockStakeEvent has drop, store {
        pool_address: address,
        amount_unlocked: u64,
    }

    struct WithdrawStakeEvent has drop, store {
        pool_address: address,
        amount_withdrawn: u64,
    }

    struct LeaveValidatorSetEvent has drop, store {
        pool_address: address,
    }

    /// Stores transaction fees assigned to validators. All fees are distributed to validators
    /// at the end of the epoch.
    struct ValidatorFees has key {
        fees_table: Table<address, Coin<AptosCoin>>,
    }

    // /// Initializes the resource storing information about collected transaction fees per validator.
    // /// Used by `transaction_fee.move` to initialize fee collection and distribution.
    // public(friend) fun initialize_validator_fees(aptos_framework: &signer) {
    //     system_addresses::assert_aptos_framework(aptos_framework);
    //     assert!(
    //         !exists<ValidatorFees>(@aptos_framework),
    //         error::already_exists(EFEES_TABLE_ALREADY_EXISTS)
    //     );
    //     move_to(aptos_framework, ValidatorFees { fees_table: table::new() });
    // }

    // /// Stores the transaction fee collected to the specified validator address.
    // public(friend) fun add_transaction_fee(validator_addr: address, fee: Coin<AptosCoin>) acquires ValidatorFees {
    //     let fees_table = &mut borrow_global_mut<ValidatorFees>(@aptos_framework).fees_table;
    //     if (table::contains(fees_table, validator_addr)) {
    //         let collected_fee = table::borrow_mut(fees_table, validator_addr);
    //         coin::merge(collected_fee, fee);
    //     } else {
    //         table::add(fees_table, validator_addr, fee);
    //     }
    // }

    // #[view]
    // /// Return the lockup expiration of the stake pool at `pool_address`.
    // /// This will throw an error if there's no stake pool at `pool_address`.
    // public fun get_lockup_secs(pool_address: address): u64 acquires StakePool {
    //     assert_stake_pool_exists(pool_address);
    //     borrow_global<StakePool>(pool_address).locked_until_secs
    // }

    // #[view]
    // /// Return the remaining lockup of the stake pool at `pool_address`.
    // /// This will throw an error if there's no stake pool at `pool_address`.
    // public fun get_remaining_lockup_secs(pool_address: address): u64 acquires StakePool {
    //     assert_stake_pool_exists(pool_address);
    //     let lockup_time = borrow_global<StakePool>(pool_address).locked_until_secs;
    //     if (lockup_time <= timestamp::now_seconds()) {
    //         0
    //     } else {
    //         lockup_time - timestamp::now_seconds()
    //     }
    // }

    // #[view]
    // /// Return the different stake amounts for `pool_address` (whether the validator is active or not).
    // /// The returned amounts are for (active, inactive, pending_active, pending_inactive) stake respectively.
    // public fun get_stake(pool_address: address): (u64, u64, u64, u64) acquires StakePool {
    //     assert_stake_pool_exists(pool_address);
    //     let stake_pool = borrow_global<StakePool>(pool_address);
    //     (
    //         coin::value(&stake_pool.active),
    //         coin::value(&stake_pool.inactive),
    //         coin::value(&stake_pool.pending_active),
    //         coin::value(&stake_pool.pending_inactive),
    //     )
    // }

    #[view]
    /// Returns the list of active validators
    public fun get_current_validators(): vector<address> acquires ValidatorSet {
        let validator_set = borrow_global<ValidatorSet>(@aptos_framework);
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
    /// Returns the validator's state.
    public fun get_validator_state(pool_address: address): u64 acquires ValidatorSet {
        let validator_set = borrow_global<ValidatorSet>(@aptos_framework);
        if (option::is_some(&find_validator(&validator_set.pending_active, pool_address))) {
            VALIDATOR_STATUS_PENDING_ACTIVE
        } else if (option::is_some(&find_validator(&validator_set.active_validators, pool_address))) {
            VALIDATOR_STATUS_ACTIVE
        } else if (option::is_some(&find_validator(&validator_set.pending_inactive, pool_address))) {
            VALIDATOR_STATUS_PENDING_INACTIVE
        } else {
            VALIDATOR_STATUS_INACTIVE
        }
    }


    //////// 0L ////////
    // TODO: v7
    #[view]
    /// Returns the validator's state.
    public fun is_valid(pool_address: address): bool acquires ValidatorSet {
        get_validator_state(pool_address) < VALIDATOR_STATUS_INACTIVE
    }


    #[view]
    /// Return the voting power of the validator in the current epoch.
    /// This is the same as the validator's total active and pending_inactive stake.
    public fun get_current_epoch_voting_power(pool_address: address): u64 acquires StakePool, ValidatorSet {
        assert_stake_pool_exists(pool_address);
        let validator_state = get_validator_state(pool_address);
        // Both active and pending inactive validators can still vote in the current epoch.
        if (validator_state == VALIDATOR_STATUS_ACTIVE || validator_state == VALIDATOR_STATUS_PENDING_INACTIVE) {
            let active_stake = coin::value(&borrow_global<StakePool>(pool_address).active);
            let pending_inactive_stake = coin::value(&borrow_global<StakePool>(pool_address).pending_inactive);
            active_stake + pending_inactive_stake
        } else {
            0
        }
    }

    // #[view]
    // /// Return the delegated voter of the validator at `pool_address`.
    // public fun get_delegated_voter(pool_address: address): address acquires StakePool {
    //     assert_stake_pool_exists(pool_address);
    //     borrow_global<StakePool>(pool_address).delegated_voter
    // }

    #[view]
    /// Return the operator of the validator at `pool_address`.
    public fun get_operator(pool_address: address): address acquires StakePool {
        assert_stake_pool_exists(pool_address);
        borrow_global<StakePool>(pool_address).operator_address
    }

    /// Return the pool address in `owner_cap`.
    public fun get_owned_pool_address(owner_cap: &OwnerCapability): address {
        owner_cap.pool_address
    }

    #[view]
    /// Return the validator index for `pool_address`.
    public fun get_validator_index(pool_address: address): u64 acquires ValidatorConfig {
        assert_stake_pool_exists(pool_address);
        borrow_global<ValidatorConfig>(pool_address).validator_index
    }

    #[view]
    /// Return the number of successful and failed proposals for the proposal at the given validator index.
    public fun get_current_epoch_proposal_counts(validator_index: u64): (u64, u64) acquires ValidatorPerformance {
        let validator_performances = &borrow_global<ValidatorPerformance>(@aptos_framework).validators;
        let validator_performance = vector::borrow(validator_performances, validator_index);
        (validator_performance.successful_proposals, validator_performance.failed_proposals)
    }

    #[view]
    /// Return the validator's config.
    public fun get_validator_config(pool_address: address): (vector<u8>, vector<u8>, vector<u8>) acquires ValidatorConfig {
        assert_stake_pool_exists(pool_address);
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        (validator_config.consensus_pubkey, validator_config.network_addresses, validator_config.fullnode_addresses)
    }

    #[view]
    public fun stake_pool_exists(addr: address): bool {
        exists<StakePool>(addr)
    }

    /// Initialize validator set to the core resource account.
    public(friend) fun initialize(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);

        move_to(aptos_framework, ValidatorSet {
            consensus_scheme: 0,
            active_validators: vector::empty(),
            pending_active: vector::empty(),
            pending_inactive: vector::empty(),
            total_voting_power: 0,
            total_joining_power: 0,
        });

        move_to(aptos_framework, ValidatorPerformance {
            validators: vector::empty(),
        });
    }

    /// This is only called during Genesis, which is where MintCapability<AptosCoin> can be created.
    /// Beyond genesis, no one can create AptosCoin mint/burn capabilities.

    // TODO: v7 - remove this
    // public(friend) fun store_aptos_coin_mint_cap(aptos_framework: &signer, mint_cap: MintCapability<AptosCoin>) {
    //     system_addresses::assert_aptos_framework(aptos_framework);
    //     move_to(aptos_framework, AptosCoinCapabilities { mint_cap })
    // }

    /// Allow on chain governance to remove validators from the validator set.
    // TODO: v7 - remove this

    public fun remove_validators(
        aptos_framework: &signer,
        validators: &vector<address>,
    ) acquires ValidatorSet {
        system_addresses::assert_aptos_framework(aptos_framework);

        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        let active_validators = &mut validator_set.active_validators;
        let pending_inactive = &mut validator_set.pending_inactive;
        let len = vector::length(validators);
        let i = 0;
        // Remove each validator from the validator set.
        while ({
            spec {
                invariant spec_validators_are_initialized(active_validators);
                invariant spec_validator_indices_are_valid(active_validators);
                invariant spec_validators_are_initialized(pending_inactive);
                invariant spec_validator_indices_are_valid(pending_inactive);
            };
            i < len
        }) {
            let validator = *vector::borrow(validators, i);
            let validator_index = find_validator(active_validators, validator);
            if (option::is_some(&validator_index)) {
                let validator_info = vector::swap_remove(active_validators, *option::borrow(&validator_index));
                vector::push_back(pending_inactive, validator_info);
            };
            i = i + 1;
        };
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
    ) acquires AllowedValidators, OwnerCapability, StakePool {
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
        // if (account_address != voter) {
        //     set_delegated_voter(owner, voter)
        // };
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

        move_to(owner, StakePool {
            active: coin::zero<AptosCoin>(),
            pending_active: coin::zero<AptosCoin>(),
            pending_inactive: coin::zero<AptosCoin>(),
            inactive: coin::zero<AptosCoin>(),
            locked_until_secs: 0,
            operator_address: owner_address,
            delegated_voter: owner_address,
            // Events.
            initialize_validator_events: account::new_event_handle<RegisterValidatorCandidateEvent>(owner),
            set_operator_events: account::new_event_handle<SetOperatorEvent>(owner),
            add_stake_events: account::new_event_handle<AddStakeEvent>(owner),
            reactivate_stake_events: account::new_event_handle<ReactivateStakeEvent>(owner),
            rotate_consensus_key_events: account::new_event_handle<RotateConsensusKeyEvent>(owner),
            update_network_and_fullnode_addresses_events: account::new_event_handle<UpdateNetworkAndFullnodeAddressesEvent>(owner),
            increase_lockup_events: account::new_event_handle<IncreaseLockupEvent>(owner),
            join_validator_set_events: account::new_event_handle<JoinValidatorSetEvent>(owner),
            distribute_rewards_events: account::new_event_handle<DistributeRewardsEvent>(owner),
            unlock_stake_events: account::new_event_handle<UnlockStakeEvent>(owner),
            withdraw_stake_events: account::new_event_handle<WithdrawStakeEvent>(owner),
            leave_validator_set_events: account::new_event_handle<LeaveValidatorSetEvent>(owner),
        });

        // move_to(owner, OwnerCapability { pool_address: owner_address });
    }

    // /// Extract and return owner capability from the signing account.
    // public fun extract_owner_cap(owner: &signer): OwnerCapability acquires OwnerCapability {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     move_from<OwnerCapability>(owner_address)
    // }

    // /// Deposit `owner_cap` into `account`. This requires `account` to not already have owernship of another
    // /// staking pool.
    // public fun deposit_owner_cap(owner: &signer, owner_cap: OwnerCapability) {
    //     assert!(!exists<OwnerCapability>(signer::address_of(owner)), error::not_found(EOWNER_CAP_ALREADY_EXISTS));
    //     move_to(owner, owner_cap);
    // }

    // /// Destroy `owner_cap`.
    // public fun destroy_owner_cap(owner_cap: OwnerCapability) {
    //     let OwnerCapability { pool_address: _ } = owner_cap;
    // }

    /// Allows an owner to change the operator of the stake pool.
    public entry fun set_operator(owner: &signer, new_operator: address) acquires OwnerCapability, StakePool {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        set_operator_with_cap(ownership_cap, new_operator);
    }

    /// Allows an account with ownership capability to change the operator of the stake pool.
    public fun set_operator_with_cap(owner_cap: &OwnerCapability, new_operator: address) acquires StakePool {
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        let old_operator = stake_pool.operator_address;
        stake_pool.operator_address = new_operator;

        event::emit_event(
            &mut stake_pool.set_operator_events,
            SetOperatorEvent {
                pool_address,
                old_operator,
                new_operator,
            },
        );
    }

    // /// Allows an owner to change the delegated voter of the stake pool.
    // public entry fun set_delegated_voter(owner: &signer, new_voter: address) acquires OwnerCapability, StakePool {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     let ownership_cap = borrow_global<OwnerCapability>(owner_address);
    //     set_delegated_voter_with_cap(ownership_cap, new_voter);
    // }

    // /// Allows an owner to change the delegated voter of the stake pool.
    // public fun set_delegated_voter_with_cap(owner_cap: &OwnerCapability, new_voter: address) acquires StakePool {
    //     let pool_address = owner_cap.pool_address;
    //     assert_stake_pool_exists(pool_address);
    //     let stake_pool = borrow_global_mut<StakePool>(pool_address);
    //     stake_pool.delegated_voter = new_voter;
    // }

    /// Add `amount` of coins from the `account` owning the StakePool.

    // TODO: v7 - remove this
    // public entry fun add_stake(owner: &signer, amount: u64) acquires OwnerCapability, StakePool, ValidatorSet {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     let ownership_cap = borrow_global<OwnerCapability>(owner_address);
    //     add_stake_with_cap(ownership_cap, coin::withdraw<AptosCoin>(owner, amount));
    // }

    // TODO: v7 - remove this

    // /// Add `coins` into `pool_address`. this requires the corresponding `owner_cap` to be passed in.
    // public fun add_stake_with_cap(owner_cap: &OwnerCapability, coins: Coin<AptosCoin>) acquires StakePool, ValidatorSet {
    //     let pool_address = owner_cap.pool_address;
    //     assert_stake_pool_exists(pool_address);

    //     let amount = coin::value(&coins);
    //     if (amount == 0) {
    //         coin::destroy_zero(coins);
    //         return
    //     };

    //     // Only track and validate voting power increase for active and pending_active validator.
    //     // Pending_inactive validator will be removed from the validator set in the next epoch.
    //     // Inactive validator's total stake will be tracked when they join the validator set.
    //     let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
    //     // Search directly rather using get_validator_state to save on unnecessary loops.
    //     if (option::is_some(&find_validator(&validator_set.active_validators, pool_address)) ||
    //         option::is_some(&find_validator(&validator_set.pending_active, pool_address))) {
    //         update_voting_power_increase(amount);
    //     };

    //     // Add to pending_active if it's a current validator because the stake is not counted until the next epoch.
    //     // Otherwise, the delegation can be added to active directly as the validator is also activated in the epoch.
    //     let stake_pool = borrow_global_mut<StakePool>(pool_address);
    //     if (is_current_epoch_validator(pool_address)) {
    //         coin::merge<AptosCoin>(&mut stake_pool.pending_active, coins);
    //     } else {
    //         coin::merge<AptosCoin>(&mut stake_pool.active, coins);
    //     };

    //     // let (_, maximum_stake) = staking_config::get_required_stake(&staking_config::get());
    //     let maximum_stake = 1000;

    //     let voting_power = get_next_epoch_voting_power(stake_pool);

    //     assert!(voting_power <= maximum_stake, error::invalid_argument(ESTAKE_EXCEEDS_MAX));

    //     event::emit_event(
    //         &mut stake_pool.add_stake_events,
    //         AddStakeEvent {
    //             pool_address,
    //             amount_added: amount,
    //         },
    //     );
    // }

    // /// Move `amount` of coins from pending_inactive to active.
    // public entry fun reactivate_stake(owner: &signer, amount: u64) acquires OwnerCapability, StakePool {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     let ownership_cap = borrow_global<OwnerCapability>(owner_address);
    //     reactivate_stake_with_cap(ownership_cap, amount);
    // }

    // public fun reactivate_stake_with_cap(owner_cap: &OwnerCapability, amount: u64) acquires StakePool {
    //     let pool_address = owner_cap.pool_address;
    //     assert_stake_pool_exists(pool_address);

    //     // Cap the amount to reactivate by the amount in pending_inactive.
    //     let stake_pool = borrow_global_mut<StakePool>(pool_address);
    //     let total_pending_inactive = coin::value(&stake_pool.pending_inactive);
    //     amount = min(amount, total_pending_inactive);

    //     // Since this does not count as a voting power change (pending inactive still counts as voting power in the
    //     // current epoch), stake can be immediately moved from pending inactive to active.
    //     // We also don't need to check voting power increase as there's none.
    //     let reactivated_coins = coin::extract(&mut stake_pool.pending_inactive, amount);
    //     coin::merge(&mut stake_pool.active, reactivated_coins);

    //     event::emit_event(
    //         &mut stake_pool.reactivate_stake_events,
    //         ReactivateStakeEvent {
    //             pool_address,
    //             amount,
    //         },
    //     );
    // }

    /// Rotate the consensus key of the validator, it'll take effect in next epoch.
    public entry fun rotate_consensus_key(
        operator: &signer,
        pool_address: address,
        new_consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
    ) acquires StakePool, ValidatorConfig {
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        assert!(exists<ValidatorConfig>(pool_address), error::not_found(EVALIDATOR_CONFIG));
        let validator_info = borrow_global_mut<ValidatorConfig>(pool_address);
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
                pool_address,
                old_consensus_pubkey,
                new_consensus_pubkey,
            },
        );
    }

    /// Update the network and full node addresses of the validator. This only takes effect in the next epoch.
    public entry fun update_network_and_fullnode_addresses(
        operator: &signer,
        pool_address: address,
        new_network_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    ) acquires StakePool, ValidatorConfig {
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        assert!(exists<ValidatorConfig>(pool_address), error::not_found(EVALIDATOR_CONFIG));
        let validator_info = borrow_global_mut<ValidatorConfig>(pool_address);
        let old_network_addresses = validator_info.network_addresses;
        validator_info.network_addresses = new_network_addresses;
        let old_fullnode_addresses = validator_info.fullnode_addresses;
        validator_info.fullnode_addresses = new_fullnode_addresses;

        event::emit_event(
            &mut stake_pool.update_network_and_fullnode_addresses_events,
            UpdateNetworkAndFullnodeAddressesEvent {
                pool_address,
                old_network_addresses,
                new_network_addresses,
                old_fullnode_addresses,
                new_fullnode_addresses,
            },
        );
    }

    // /// Similar to increase_lockup_with_cap but will use ownership capability from the signing account.
    // public entry fun increase_lockup(owner: &signer) acquires OwnerCapability,  {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     let _ownership_cap = borrow_global<OwnerCapability>(owner_address);
    //     // increase_lockup_with_cap(ownership_cap);
    // }

    /// Unlock from active delegation, it's moved to pending_inactive if locked_until_secs < current_time or
    /// directly inactive if it's not from an active validator.
    // public fun increase_lockup_with_cap(owner_cap: &OwnerCapability) acquires StakePool {
    //     let pool_address = owner_cap.pool_address;
    //     assert_stake_pool_exists(pool_address);
    //     let config = staking_config::get();

    //     let stake_pool = borrow_global_mut<StakePool>(pool_address);
    //     let old_locked_until_secs = stake_pool.locked_until_secs;
    //     let new_locked_until_secs = timestamp::now_seconds() + staking_config::get_recurring_lockup_duration(&config);
    //     assert!(old_locked_until_secs < new_locked_until_secs, error::invalid_argument(EINVALID_LOCKUP));
    //     stake_pool.locked_until_secs = new_locked_until_secs;

    //     event::emit_event(
    //         &mut stake_pool.increase_lockup_events,
    //         IncreaseLockupEvent {
    //             pool_address,
    //             old_locked_until_secs,
    //             new_locked_until_secs,
    //         },
    //     );
    // }

    /// This can only called by the operator of the validator/staking pool.
    public entry fun join_validator_set(
        operator: &signer,
        pool_address: address
    ) acquires StakePool, ValidatorConfig, ValidatorSet {
        // assert!(
        //     staking_config::get_allow_validator_set_change(&staking_config::get()),
        //     error::invalid_argument(ENO_POST_GENESIS_VALIDATOR_SET_CHANGE_ALLOWED),
        // );

        join_validator_set_internal(operator, pool_address);
    }

    /// Request to have `pool_address` join the validator set. Can only be called after calling `initialize_validator`.
    /// If the validator has the required stake (more than minimum and less than maximum allowed), they will be
    /// added to the pending_active queue. All validators in this queue will be added to the active set when the next
    /// epoch starts (eligibility will be rechecked).
    ///
    /// This internal version can only be called by the Genesis module during Genesis.
    public(friend) fun join_validator_set_internal(
        operator: &signer,
        pool_address: address
    ) acquires StakePool, ValidatorConfig, ValidatorSet {

        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        assert!(
            get_validator_state(pool_address) == VALIDATOR_STATUS_INACTIVE,
            error::invalid_state(EALREADY_ACTIVE_VALIDATOR),
        );

        // let config = staking_config::get();
        // let (minimum_stake, maximum_stake) = staking_config::get_required_stake(&config);
        let minimum_stake = 1;
        let maximum_stake = 100;
        let voting_power = 1; // voting power is always 1 in Libra
        assert!(voting_power >= minimum_stake, error::invalid_argument(ESTAKE_TOO_LOW));
        assert!(voting_power <= maximum_stake, error::invalid_argument(ESTAKE_TOO_HIGH));
        // Track and validate voting power increase.
        update_voting_power_increase(voting_power);


        // Add validator to pending_active, to be activated in the next epoch.
        let validator_config = borrow_global_mut<ValidatorConfig>(pool_address);
        assert!(!vector::is_empty(&validator_config.consensus_pubkey), error::invalid_argument(EINVALID_PUBLIC_KEY));

        // Validate the current validator set size has not exceeded the limit.
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        vector::push_back(&mut validator_set.pending_active, generate_validator_info(pool_address, stake_pool, *validator_config));

        let validator_set_size = vector::length(&validator_set.active_validators) + vector::length(&validator_set.pending_active);
        assert!(validator_set_size <= MAX_VALIDATOR_SET_SIZE, error::invalid_argument(EVALIDATOR_SET_TOO_LARGE));

        event::emit_event(
            &mut stake_pool.join_validator_set_events,
            JoinValidatorSetEvent { pool_address },
        );
    }

    /// Similar to unlock_with_cap but will use ownership capability from the signing account.
    // public entry fun unlock(owner: &signer, amount: u64) acquires OwnerCapability, StakePool {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     let ownership_cap = borrow_global<OwnerCapability>(owner_address);
    //     unlock_with_cap(amount, ownership_cap);
    // }

    /// Unlock `amount` from the active stake. Only possible if the lockup has expired.
    // public fun unlock_with_cap(amount: u64, owner_cap: &OwnerCapability) acquires StakePool {
    //     // Short-circuit if amount to unlock is 0 so we don't emit events.
    //     if (amount == 0) {
    //         return
    //     };

    //     // Unlocked coins are moved to pending_inactive. When the current lockup cycle expires, they will be moved into
    //     // inactive in the earliest possible epoch transition.
    //     let pool_address = owner_cap.pool_address;
    //     assert_stake_pool_exists(pool_address);
    //     let stake_pool = borrow_global_mut<StakePool>(pool_address);
    //     // Cap amount to unlock by maximum active stake.
    //     let amount = min(amount, coin::value(&stake_pool.active));
    //     let unlocked_stake = coin::extract(&mut stake_pool.active, amount);
    //     coin::merge<AptosCoin>(&mut stake_pool.pending_inactive, unlocked_stake);

    //     event::emit_event(
    //         &mut stake_pool.unlock_stake_events,
    //         UnlockStakeEvent {
    //             pool_address,
    //             amount_unlocked: amount,
    //         },
    //     );
    // }

    // /// Withdraw from `account`'s inactive stake.
    // public entry fun withdraw(
    //     owner: &signer,
    //     withdraw_amount: u64
    // ) acquires OwnerCapability, StakePool, ValidatorSet {
    //     let owner_address = signer::address_of(owner);
    //     assert_owner_cap_exists(owner_address);
    //     let ownership_cap = borrow_global<OwnerCapability>(owner_address);
    //     let coins = withdraw_with_cap(ownership_cap, withdraw_amount);
    //     coin::deposit<AptosCoin>(owner_address, coins);
    // }

    // /// Withdraw from `pool_address`'s inactive stake with the corresponding `owner_cap`.
    // public fun withdraw_with_cap(
    //     owner_cap: &OwnerCapability,
    //     withdraw_amount: u64
    // ): Coin<AptosCoin> acquires StakePool, ValidatorSet {
    //     let pool_address = owner_cap.pool_address;
    //     assert_stake_pool_exists(pool_address);
    //     let stake_pool = borrow_global_mut<StakePool>(pool_address);
    //     // There's an edge case where a validator unlocks their stake and leaves the validator set before
    //     // the stake is fully unlocked (the current lockup cycle has not expired yet).
    //     // This can leave their stake stuck in pending_inactive even after the current lockup cycle expires.
    //     if (get_validator_state(pool_address) == VALIDATOR_STATUS_INACTIVE &&
    //         timestamp::now_seconds() >= stake_pool.locked_until_secs) {
    //         let pending_inactive_stake = coin::extract_all(&mut stake_pool.pending_inactive);
    //         coin::merge(&mut stake_pool.inactive, pending_inactive_stake);
    //     };

    //     // Cap withdraw amount by total ianctive coins.
    //     withdraw_amount = min(withdraw_amount, coin::value(&stake_pool.inactive));
    //     if (withdraw_amount == 0) return coin::zero<AptosCoin>();

    //     event::emit_event(
    //         &mut stake_pool.withdraw_stake_events,
    //         WithdrawStakeEvent {
    //             pool_address,
    //             amount_withdrawn: withdraw_amount,
    //         },
    //     );

    //     coin::extract(&mut stake_pool.inactive, withdraw_amount)
    // }

    /// Request to have `pool_address` leave the validator set. The validator is only actually removed from the set when
    /// the next epoch starts.
    /// The last validator in the set cannot leave. This is an edge case that should never happen as long as the network
    /// is still operational.
    ///
    /// Can only be called by the operator of the validator/staking pool.

    // TODO: V7 change this
    public entry fun leave_validator_set(
        operator: &signer,
        pool_address: address
    ) acquires StakePool, ValidatorSet {
        // let config = staking_config::get();
        // assert!(
        //     staking_config::get_allow_validator_set_change(&config),
        //     error::invalid_argument(ENO_POST_GENESIS_VALIDATOR_SET_CHANGE_ALLOWED),
        // );

        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        // Account has to be the operator.
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        // If the validator is still pending_active, directly kick the validator out.
        let maybe_pending_active_index = find_validator(&validator_set.pending_active, pool_address);
        if (option::is_some(&maybe_pending_active_index)) {
            vector::swap_remove(
                &mut validator_set.pending_active, option::extract(&mut maybe_pending_active_index));

            // Decrease the voting power increase as the pending validator's voting power was added when they requested
            // to join. Now that they changed their mind, their voting power should not affect the joining limit of this
            // epoch.
            let validator_stake = 1; // NOTE: voting power is always 1 in Libra
            // total_joining_power should be larger than validator_stake but just in case there has been a small
            // rounding error somewhere that can lead to an underflow, we still want to allow this transaction to
            // succeed.
            if (validator_set.total_joining_power > validator_stake) {
                validator_set.total_joining_power = validator_set.total_joining_power - validator_stake;
            } else {
                validator_set.total_joining_power = 0;
            };
        } else {
            // Validate that the validator is already part of the validator set.
            let maybe_active_index = find_validator(&validator_set.active_validators, pool_address);
            assert!(option::is_some(&maybe_active_index), error::invalid_state(ENOT_VALIDATOR));
            let validator_info = vector::swap_remove(
                &mut validator_set.active_validators, option::extract(&mut maybe_active_index));
            assert!(vector::length(&validator_set.active_validators) > 0, error::invalid_state(ELAST_VALIDATOR));
            vector::push_back(&mut validator_set.pending_inactive, validator_info);

            event::emit_event(
                &mut stake_pool.leave_validator_set_events,
                LeaveValidatorSetEvent {
                    pool_address,
                },
            );
        };
    }

    /// Returns true if the current validator can still vote in the current epoch.
    /// This includes validators that requested to leave but are still in the pending_inactive queue and will be removed
    /// when the epoch starts.
    public fun is_current_epoch_validator(pool_address: address): bool acquires ValidatorSet {
        assert_stake_pool_exists(pool_address);
        let validator_state = get_validator_state(pool_address);
        validator_state == VALIDATOR_STATUS_ACTIVE || validator_state == VALIDATOR_STATUS_PENDING_INACTIVE
    }

    /// Update the validator performance (proposal statistics). This is only called by block::prologue().
    /// This function cannot abort.
    public(friend) fun update_performance_statistics(proposer_index: Option<u64>, failed_proposer_indices: vector<u64>) acquires ValidatorPerformance {
        // Validator set cannot change until the end of the epoch, so the validator index in arguments should
        // match with those of the validators in ValidatorPerformance resource.
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);
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
    ///
    /// 1. Distribute transaction fees and rewards to stake pools of active and pending inactive validators (requested
    /// to leave but not yet removed).
    /// 2. Officially move pending active stake to active and move pending inactive stake to inactive.
    /// The staking pool's voting power in this new epoch will be updated to the total active stake.
    /// 3. Add pending active validators to the active set if they satisfy requirements so they can vote and remove
    /// pending inactive validators so they no longer can vote.
    /// 4. The validator's voting power in the validator set is updated to be the corresponding staking pool's voting
    /// power.
    public(friend) fun on_new_epoch() acquires StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet {
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        // let config = staking_config::get();
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);

        // Process pending stake and distribute transaction fees and rewards for each currently active validator.
        // let i = 0;
        // let len = vector::length(&validator_set.active_validators);
        // while (i < len) {
        //     let validator = vector::borrow(&validator_set.active_validators, i);
        //     update_stake_pool(validator_perf, validator.addr, &config);
        //     i = i + 1;
        // };

        // Process pending stake and distribute transaction fees and rewards for each currently pending_inactive validator
        // (requested to leave but not removed yet).
        // let i = 0;
        // let len = vector::length(&validator_set.pending_inactive);
        // while (i < len) {
        //     let validator = vector::borrow(&validator_set.pending_inactive, i);
        //     update_stake_pool(validator_perf, validator.addr, &config);
        //     i = i + 1;
        // };

        // Activate currently pending_active validators.
        append(&mut validator_set.active_validators, &mut validator_set.pending_active);

        // Officially deactivate all pending_inactive validators. They will now no longer receive rewards.
        validator_set.pending_inactive = vector::empty();

        // Update active validator set so that network address/public key change takes effect.
        // Moreover, recalculate the total voting power, and deactivate the validator whose
        // voting power is less than the minimum required stake.
        let next_epoch_validators = vector::empty();
        // let (minimum_stake, _) = staking_config::get_required_stake(&config);
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
            let pool_address = old_validator_info.addr;
            let validator_config = borrow_global_mut<ValidatorConfig>(pool_address);
            let stake_pool = borrow_global_mut<StakePool>(pool_address);
            let new_validator_info = generate_validator_info(pool_address, stake_pool, *validator_config);

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


        validator_set.total_joining_power = 0;

        // Update validator indices, reset performance scores, and renew lockups.
        validator_perf.validators = vector::empty();
        // let recurring_lockup_duration_secs = 0;
        //staking_config::get_recurring_lockup_duration(&config);


        let vlen = vector::length(&validator_set.active_validators);
        let validator_index = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(validator_set.active_validators);
                invariant len(validator_set.pending_active) == 0;
                invariant len(validator_set.pending_inactive) == 0;
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

            // // Automatically renew a validator's lockup for validators that will still be in the validator set in the
            // // next epoch.
            // let stake_pool = borrow_global_mut<StakePool>(validator_info.addr);
            // if (stake_pool.locked_until_secs <= timestamp::now_seconds()) {
            //     spec {
            //         assume timestamp::spec_now_seconds() + recurring_lockup_duration_secs <= MAX_U64;
            //     };
            //     stake_pool.locked_until_secs =
            //         timestamp::now_seconds() + recurring_lockup_duration_secs;
            // };

            validator_index = validator_index + 1;
        };

        // if (features::periodical_reward_rate_decrease_enabled()) {
        //     // Update rewards rate after reward distribution.
        //     staking_config::calculate_and_save_latest_epoch_rewards_rate();
        // };
    }

    public(friend) fun ol_on_new_epoch(root: &signer, list: &vector<address>) acquires StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet {


        // will update the stake of active validators.
        // NOTE: ol does not use the pending, and pending inactive lists.
        // critical mutation, belt and suspenders
        try_bulk_update(root, list);

        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);


        // Officially deactivate all pending_inactive validators. They will now no longer receive rewards.
        // validator_set.pending_inactive = vector::empty();
        // validator_set.total_joining_power = 0;


        // Update validator indices, reset performance scores, and renew lockups.
        validator_perf.validators = vector::empty();


        let vlen = vector::length(&validator_set.active_validators);
        let validator_index = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(validator_set.active_validators);
                invariant len(validator_set.pending_active) == 0;
                invariant len(validator_set.pending_inactive) == 0;
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

            // // Automatically renew a validator's lockup for validators that will still be in the validator set in the
            // // next epoch.
            // let stake_pool = borrow_global_mut<StakePool>(validator_info.addr);
            // if (stake_pool.locked_until_secs <= timestamp::now_seconds()) {
            //     spec {
            //         assume timestamp::spec_now_seconds() + recurring_lockup_duration_secs <= MAX_U64;
            //     };
            //     stake_pool.locked_until_secs =
            //         timestamp::now_seconds() + recurring_lockup_duration_secs;
            // };

            validator_index = validator_index + 1;
        };

        // if (features::periodical_reward_rate_decrease_enabled()) {
        //     // Update rewards rate after reward distribution.
        //     staking_config::calculate_and_save_latest_epoch_rewards_rate();
        // };
    }

    /////// 0L ///////
    // Critical mutation. Using belt and suspenders
    fun try_bulk_update(root: &signer, list: &vector<address>) acquires StakePool, ValidatorConfig, ValidatorSet {
      if (signer::address_of(root) != @ol_framework) return;

      let (list_info, _voting_power) = make_validator_set_config(list);

      // mutations happen in private function
      maybe_set_next_validators(root, list_info);
    }

    #[test_only] 
    public fun test_make_val_cfg(list: &vector<address>): (vector<ValidatorInfo>, u128) acquires StakePool, ValidatorConfig {
      make_validator_set_config(list)
    }

    fun make_validator_set_config(list: &vector<address>): (vector<ValidatorInfo>, u128) acquires StakePool, ValidatorConfig  {

      let next_epoch_validators = vector::empty();
      let vlen = vector::length(list);

      let total_voting_power = 0;
      let i = 0;
      while ({
          spec {
              invariant spec_validators_are_initialized(next_epoch_validators);
          };
          i < vlen
      }) {
          // let old_validator_info = vector::borrow_mut(&mut validator_set.active_validators, i);

          let pool_address = *vector::borrow(list, i);


          if (!exists<ValidatorConfig>(pool_address)) {
            i = i + 1;
            continue
          }; // belt and suspenders

          let validator_config = borrow_global_mut<ValidatorConfig>(pool_address);

          if (!exists<StakePool>(pool_address)) {
            i = i + 1;
            continue
          }; // belt and suspenders
          let stake_pool = borrow_global_mut<StakePool>(pool_address);


          let new_validator_info = generate_validator_info(pool_address, stake_pool, *validator_config);

          // all validators have same weight
          total_voting_power = total_voting_power + 1;

          vector::push_back(&mut next_epoch_validators, new_validator_info);
          // A validator needs at least the min stake required to join the validator set.

          // spec {
          //     assume total_voting_power + 1<= MAX_U128;
          // };
              
          i = i + 1;
      };

      (next_epoch_validators, total_voting_power)
    }

    #[test_only] 
    public fun test_set_next_vals(root: &signer, list: vector<ValidatorInfo>) acquires ValidatorSet {
      system_addresses::assert_ol(root);
      maybe_set_next_validators(root, list);
    }

    /// sets the global state for the next epoch validators
    /// belt and suspenders, private function is authorized. Test functions also authorized.
    fun maybe_set_next_validators(root: &signer, list: vector<ValidatorInfo>) acquires ValidatorSet {
      if (signer::address_of(root) != @ol_framework) return;
      // check if this is not test
      if (!testnet::is_testnet() && vector::length(&list) < 5) {
        return
      };
      
      let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
      validator_set.active_validators = list;
      validator_set.total_voting_power = (vector::length(&list) as u128);
    }

    /// Update individual validator's stake pool
    /// 1. distribute transaction fees to active/pending_inactive delegations
    /// 2. distribute rewards to active/pending_inactive delegations
    /// 3. process pending_active, pending_inactive correspondingly
    /// This function shouldn't abort.
    fun update_stake_pool(
        validator_perf: &ValidatorPerformance,
        pool_address: address,
        // _staking_config: &StakingConfig,
    ) acquires StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorFees {
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        let cur_validator_perf = vector::borrow(&validator_perf.validators, validator_config.validator_index);
        let num_successful_proposals = cur_validator_perf.successful_proposals;
        spec {
            // The following addition should not overflow because `num_total_proposals` cannot be larger than 86400,
            // the maximum number of proposals in a day (1 proposal per second).
            assume cur_validator_perf.successful_proposals + cur_validator_perf.failed_proposals <= MAX_U64;
        };
        let num_total_proposals = cur_validator_perf.successful_proposals + cur_validator_perf.failed_proposals;

        // let (rewards_rate, rewards_rate_denominator) = staking_config::get_reward_rate(staking_config);


        let rewards_rate = 1;
        let rewards_rate_denominator = 1;

        let rewards_active = distribute_rewards(
            &mut stake_pool.active,
            num_successful_proposals,
            num_total_proposals,
            rewards_rate,
            rewards_rate_denominator
        );
        let rewards_pending_inactive = distribute_rewards(
            &mut stake_pool.pending_inactive,
            num_successful_proposals,
            num_total_proposals,
            rewards_rate,
            rewards_rate_denominator
        );
        spec {
            assume rewards_active + rewards_pending_inactive <= MAX_U64;
        };
        let rewards_amount = rewards_active + rewards_pending_inactive;
        // Pending active stake can now be active.
        coin::merge(&mut stake_pool.active, coin::extract_all(&mut stake_pool.pending_active));

        // Additionally, distribute transaction fees.
        if (features::collect_and_distribute_gas_fees()) {
            let fees_table = &mut borrow_global_mut<ValidatorFees>(@aptos_framework).fees_table;
            if (table::contains(fees_table, pool_address)) {
                let coin = table::remove(fees_table, pool_address);
                coin::merge(&mut stake_pool.active, coin);
            };
        };

        // Pending inactive stake is only fully unlocked and moved into inactive if the current lockup cycle has expired
        let current_lockup_expiration = stake_pool.locked_until_secs;
        if (timestamp::now_seconds() >= current_lockup_expiration) {
            coin::merge(
                &mut stake_pool.inactive,
                coin::extract_all(&mut stake_pool.pending_inactive),
            );
        };

        event::emit_event(
            &mut stake_pool.distribute_rewards_events,
            DistributeRewardsEvent {
                pool_address,
                rewards_amount,
            },
        );
    }

    // /// Calculate the rewards amount.
    // // TODO: v7 - remove this
    // fun calculate_rewards_amount(
    //     stake_amount: u64,
    //     num_successful_proposals: u64,
    //     num_total_proposals: u64,
    //     rewards_rate: u64,
    //     rewards_rate_denominator: u64,
    // ): u64 {

    //     0
    // }

    /// Mint rewards corresponding to current epoch's `stake` and `num_successful_votes`.

    // TODO: v7 - change this
    fun distribute_rewards(
        stake: &mut Coin<AptosCoin>,
        _num_successful_proposals: u64,
        _num_total_proposals: u64,
        _rewards_rate: u64,
        _rewards_rate_denominator: u64,
    ): u64 acquires AptosCoinCapabilities {
        // let _stake_amount = coin::value(stake);
        let rewards_amount = 0;
        if (rewards_amount > 0) {
            let mint_cap = &borrow_global<AptosCoinCapabilities>(@aptos_framework).mint_cap;
            let rewards = coin::mint(rewards_amount, mint_cap);
            coin::merge(stake, rewards);
        };
        rewards_amount
    }

    fun append<T>(v1: &mut vector<T>, v2: &mut vector<T>) {
        while (!vector::is_empty(v2)) {
            vector::push_back(v1, vector::pop_back(v2));
        }
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

    fun generate_validator_info(addr: address, stake_pool: &StakePool, config: ValidatorConfig): ValidatorInfo {
        let voting_power = get_next_epoch_voting_power(stake_pool);
        ValidatorInfo {
            addr,
            voting_power,
            config,
        }
    }

    /// Returns validator's next epoch voting power, including pending_active, active, and pending_inactive stake.

    // TODO: v7 - remove this

    fun get_next_epoch_voting_power(_stake_pool: &StakePool): u64 {
        // let value_pending_active = coin::value(&stake_pool.pending_active);
        // let value_active = coin::value(&stake_pool.active);
        // let value_pending_inactive = coin::value(&stake_pool.pending_inactive);
        // spec {
        //     assume value_pending_active + value_active + value_pending_inactive <= MAX_U64;
        // };
        // // value_pending_active + value_active + value_pending_inactive;
        1
    }

    fun update_voting_power_increase(increase_amount: u64) acquires ValidatorSet {
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        // let voting_power_increase_limit =
        //     (staking_config::get_voting_power_increase_limit(&staking_config::get()) as u128);
        let voting_power_increase_limit = 1000;
        validator_set.total_joining_power = validator_set.total_joining_power + (increase_amount as u128);

        // Only validator voting power increase if the current validator set's voting power > 0.
        if (validator_set.total_voting_power > 0) {
            assert!(
                validator_set.total_joining_power <= validator_set.total_voting_power * voting_power_increase_limit / 100,
                error::invalid_argument(EVOTING_POWER_INCREASE_EXCEEDS_LIMIT),
            );
        }
    }

    fun assert_stake_pool_exists(pool_address: address) {
        assert!(stake_pool_exists(pool_address), error::invalid_argument(ESTAKE_POOL_DOES_NOT_EXIST));
    }

    //////////// TESTNET ////////////

    /// This provides an ACL for Testnet purposes. In testnet, everyone is a whale, a whale can be a validator.
    /// This allows a testnet to bring additional entities into the validator set without compromising the
    /// security of the testnet. This will NOT be enabled in Mainnet.
    struct AllowedValidators has key {
        accounts: vector<address>,
    }

    public fun configure_allowed_validators(aptos_framework: &signer, accounts: vector<address>) acquires AllowedValidators {
        let aptos_framework_address = signer::address_of(aptos_framework);
        system_addresses::assert_aptos_framework(aptos_framework);
        if (!exists<AllowedValidators>(aptos_framework_address)) {
            move_to(aptos_framework, AllowedValidators { accounts });
        } else {
            let allowed = borrow_global_mut<AllowedValidators>(aptos_framework_address);
            allowed.accounts = accounts;
        }
    }

    fun is_allowed(account: address): bool acquires AllowedValidators {
        if (!exists<AllowedValidators>(@aptos_framework)) {
            true
        } else {
            let allowed = borrow_global<AllowedValidators>(@aptos_framework);
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

      let perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);

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
    public fun end_epoch() acquires StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet {
        // Set the number of blocks to 1, to give out rewards to non-failing validators.
        set_validator_perf_at_least_one_block();
        timestamp::fast_forward_seconds(EPOCH_DURATION);
        on_new_epoch();
    }




    #[test_only]
    public fun set_validator_perf_at_least_one_block() acquires ValidatorPerformance {
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);
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
        public_key: &bls12381::PublicKey,
        proof_of_possession: &bls12381::ProofOfPossession,
        validator: &signer,
        _amount: u64,
        should_join_validator_set: bool,
        should_end_epoch: bool,
    ) acquires AllowedValidators, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet {
        let validator_address = signer::address_of(validator);
        if (!account::exists_at(signer::address_of(validator))) {
            account::create_account_for_test(validator_address);
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
}
