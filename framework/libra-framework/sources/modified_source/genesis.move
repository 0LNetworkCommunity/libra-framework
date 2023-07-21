module aptos_framework::genesis {
    // use aptos_std::debug::print;

    use std::error;
    use std::vector;

    use aptos_framework::account;
    use aptos_framework::aggregator_factory;
    use aptos_framework::aptos_coin::{Self, AptosCoin};
    use aptos_framework::aptos_governance;
    use aptos_framework::block;
    use aptos_framework::chain_id;
    use aptos_framework::chain_status;
    use aptos_framework::coin;
    use aptos_framework::consensus_config;
    use aptos_framework::execution_config;
    use aptos_framework::create_signer::create_signer;
    use aptos_framework::gas_schedule;
    use aptos_framework::reconfiguration;
    use aptos_framework::stake;
    use aptos_framework::match_index;
    use aptos_framework::state_storage;
    use aptos_framework::storage_gas;
    use aptos_framework::timestamp;
    use aptos_framework::transaction_fee;
    use aptos_framework::transaction_validation;
    use aptos_framework::version;

    //////// 0L ////////
    use aptos_framework::validator_universe;
    use ol_framework::ol_account;
    use ol_framework::musical_chairs;
    use ol_framework::proof_of_fee;
    use ol_framework::slow_wallet;
    use ol_framework::gas_coin::{Self, GasCoin};
    use ol_framework::infra_escrow;
    use ol_framework::tower_state;
    use ol_framework::safe;
    use ol_framework::donor_directed;
    use ol_framework::epoch_helper;
    use ol_framework::burn;
    use ol_framework::fee_maker;

    const TESTNET_GENESIS_BOOTSTRAP_COIN: u64 = 10000000000; //10,000 coins with 6 digits precision: 10B coins.
    //////// end 0L ////////


    const EDUPLICATE_ACCOUNT: u64 = 1;
    const EACCOUNT_DOES_NOT_EXIST: u64 = 2;

    struct AccountMap has drop {
        account_address: address,
        balance: u64,
    }

    struct EmployeeAccountMap has copy, drop {
        accounts: vector<address>,
        validator: ValidatorConfigurationWithCommission,
        vesting_schedule_numerator: vector<u64>,
        vesting_schedule_denominator: u64,
        beneficiary_resetter: address,
    }

    struct ValidatorConfiguration has copy, drop {
        owner_address: address,
        operator_address: address,
        voter_address: address,
        stake_amount: u64,
        consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
        network_addresses: vector<u8>,
        full_node_network_addresses: vector<u8>,
    }

    struct ValidatorConfigurationWithCommission has copy, drop {
        validator_config: ValidatorConfiguration,
        commission_percentage: u64,
        join_during_genesis: bool,
    }

    /// Genesis step 1: Initialize aptos framework account and core modules on chain.
    fun initialize(
        gas_schedule: vector<u8>,
        chain_id: u8,
        initial_version: u64,
        consensus_config: vector<u8>,
        execution_config: vector<u8>,
        epoch_interval_microsecs: u64,
        _minimum_stake: u64,
        _maximum_stake: u64,
        _recurring_lockup_duration_secs: u64,
        _allow_validator_set_change: bool,
        _rewards_rate: u64,
        _rewards_rate_denominator: u64,
        _voting_power_increase_limit: u64,
    ) {
        // Initialize the aptos framework account. This is the account where system resources and modules will be
        // deployed to. This will be entirely managed by on-chain governance and no entities have the key or privileges
        // to use this account.
        let (aptos_framework_account, aptos_framework_signer_cap) = account::create_framework_reserved_account(@aptos_framework);
        // Initialize account configs on aptos framework account.
        account::initialize(&aptos_framework_account);

        transaction_validation::initialize(
            &aptos_framework_account,
            b"script_prologue",
            b"module_prologue",
            b"multi_agent_script_prologue",
            b"epilogue",
        );

        // Give the decentralized on-chain governance control over the core framework account.
        aptos_governance::store_signer_cap(&aptos_framework_account, @aptos_framework, aptos_framework_signer_cap);

        // put reserved framework reserved accounts under aptos governance
        let framework_reserved_addresses = vector<address>[@0x2, @0x3, @0x4, @0x5, @0x6, @0x7, @0x8, @0x9, @0xa];
        while (!vector::is_empty(&framework_reserved_addresses)) {
            let address = vector::pop_back<address>(&mut framework_reserved_addresses);
            let (_, framework_signer_cap) = account::create_framework_reserved_account(address);
            aptos_governance::store_signer_cap(&aptos_framework_account, address, framework_signer_cap);
        };

        consensus_config::initialize(&aptos_framework_account, consensus_config);
        execution_config::set(&aptos_framework_account, execution_config);
        version::initialize(&aptos_framework_account, initial_version);
        stake::initialize(&aptos_framework_account);
        // staking_config::initialize(
        //     &aptos_framework_account,
        //     minimum_stake,
        //     maximum_stake,
        //     recurring_lockup_duration_secs,
        //     allow_validator_set_change,
        //     rewards_rate,
        //     rewards_rate_denominator,
        //     voting_power_increase_limit,
        // );
        storage_gas::initialize(&aptos_framework_account);
        gas_schedule::initialize(&aptos_framework_account, gas_schedule);

        // Ensure we can create aggregators for supply, but not enable it for common use just yet.
        aggregator_factory::initialize_aggregator_factory(&aptos_framework_account);
        coin::initialize_supply_config(&aptos_framework_account);

        chain_id::initialize(&aptos_framework_account, chain_id);
        reconfiguration::initialize(&aptos_framework_account);
        block::initialize(&aptos_framework_account, epoch_interval_microsecs);
        state_storage::initialize(&aptos_framework_account);

        //////// 0L ////////

        validator_universe::initialize(&aptos_framework_account);
        //TODO!: genesis seats
        let genesis_seats = 10;
        musical_chairs::initialize(&aptos_framework_account, genesis_seats);
        proof_of_fee::init_genesis_baseline_reward(&aptos_framework_account);
        slow_wallet::initialize(&aptos_framework_account);
        infra_escrow::initialize(&aptos_framework_account);
        tower_state::initialize(&aptos_framework_account);
        safe::initialize(&aptos_framework_account);
        donor_directed::initialize(&aptos_framework_account);
        epoch_helper::initialize(&aptos_framework_account);
        burn::initialize(&aptos_framework_account);
        match_index::initialize(&aptos_framework_account);
        fee_maker::initialize(&aptos_framework_account);

        // end 0L

        timestamp::set_time_has_started(&aptos_framework_account);
    }


    // TODO: 0L: this is called at genesis from Rust side. Need to replace names there.
    // we are leaving vendor's coin in place.
    /// Genesis step 2: Initialize Aptos coin.
    fun initialize_aptos_coin(aptos_framework: &signer) {

        let (burn_cap, mint_cap) = aptos_coin::initialize(aptos_framework);
        // Give stake module MintCapability<AptosCoin> so it can mint rewards.
        // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
        coin::destroy_mint_cap(mint_cap);
        // Give transaction_fee module BurnCapability<AptosCoin> so it can burn gas.
        // transaction_fee::store_aptos_coin_burn_cap(aptos_framework, burn_cap);
        coin::destroy_burn_cap(burn_cap);

        // 0L: genesis ceremony is calling this
        gas_coin::initialize(aptos_framework);
        // Give stake module MintCapability<AptosCoin> so it can mint rewards.
        // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
        // gas_coin::restore_mint_cap(aptos_framework, mint_cap);
        // coin::destroy_burn_cap(burn_cap);
        transaction_fee::initialize_fee_collection_and_distribution(aptos_framework, 0);
    }

    // TODO: 0L: replace this with gas coin. using vendor's to preserve tests while WIP.
    /// Only called for testnets and e2e tests.
    fun initialize_core_resources_and_aptos_coin(
        aptos_framework: &signer,
        core_resources_auth_key: vector<u8>,
    ) {
        let (burn_cap, mint_cap) = aptos_coin::initialize(aptos_framework);

        let core_resources = account::create_account(@core_resources);
        account::rotate_authentication_key_internal(&core_resources, core_resources_auth_key);
        aptos_coin::configure_accounts_for_test(aptos_framework, &core_resources, mint_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);

        // initialize gas
        let (burn_cap_two, mint_cap_two) = gas_coin::initialize_for_core(aptos_framework);
        // give coins to the 'core_resources' account, which has sudo
        // core_resources is a temporary account, not the same as framework account.
        gas_coin::configure_accounts_for_test(aptos_framework, &core_resources, mint_cap_two);

        // NOTE: 0L: the commented code below shows that we are destroying
        // there capabilities elsewhere, at gas_coin::initialize
        coin::destroy_mint_cap(mint_cap_two);
        coin::destroy_burn_cap(burn_cap_two);

    }

    fun create_accounts(aptos_framework: &signer, accounts: vector<AccountMap>) {
        let i = 0;
        let num_accounts = vector::length(&accounts);
        let unique_accounts = vector::empty();

        while (i < num_accounts) {
            let account_map = vector::borrow(&accounts, i);
            assert!(
                !vector::contains(&unique_accounts, &account_map.account_address),
                error::already_exists(EDUPLICATE_ACCOUNT),
            );
            vector::push_back(&mut unique_accounts, account_map.account_address);

            create_account(
                aptos_framework,
                account_map.account_address,
                account_map.balance,
            );

            i = i + 1;
        };
    }

    /// This creates an funds an account if it doesn't exist.
    /// If it exists, it just returns the signer.
    fun create_account(_aptos_framework: &signer, account_address: address, _balance: u64): signer {
        if (account::exists_at(account_address)) {
            create_signer(account_address)
        } else {
            let account = account::create_account(account_address);
            coin::register<AptosCoin>(&account);
            coin::register<GasCoin>(&account);

            // NO COINS MINTED AT GENESIS, NO PREMINE FOR TEST OR PROD.
            // aptos_coin::mint(aptos_framework, account_address, balance);
            // gas_coin::mint(aptos_framework, account_address, TESTNET_GENESIS_BOOTSTRAP_COIN);
            account
        }
    }

    // This creates an funds an account if it doesn't exist.
    /// If it exists, it just returns the signer.
    fun ol_create_account(root: &signer, account_address: address): signer {
        // assert!(!account::exists_at(account_address), error::already_exists(EDUPLICATE_ACCOUNT));
        if (!account::exists_at(account_address)) {
            ol_account::create_account(root, account_address);
            gas_coin::mint(root, account_address, 10000);

        };
        // NOTE: after the inital genesis set up, the validators can rotate their auth keys.
        // let new_signer = account::create_account(account_address);
        // coin::register<GasCoin>(&new_signer);
        // mint genesis bootstrap coins


        create_signer(account_address)
    }

    fun create_initialize_validators_with_commission(
        aptos_framework: &signer,
        use_staking_contract: bool,
        validators: vector<ValidatorConfigurationWithCommission>,
    ) {
        let i = 0;
        let num_validators = vector::length(&validators);

        while (i < num_validators) {

            let validator = vector::borrow(&validators, i);
            create_validator_accounts(aptos_framework, validator, use_staking_contract);

            i = i + 1;
        };



        // Destroy the aptos framework account's ability to mint coins now that we're done with setting up the initial
        // validators.
        // aptos_coin::destroy_mint_cap(aptos_framework);

        stake::on_new_epoch();

    }

    /// Sets up the initial validator set for the network.
    /// The validator "owner" accounts, and their authentication
    /// Addresses (and keys) are encoded in the `owners`
    /// Each validator signs consensus messages with the private key corresponding to the Ed25519
    /// public key in `consensus_pubkeys`.
    /// Finally, each validator must specify the network address
    /// (see types/src/network_address/mod.rs) for itself and its full nodes.
    ///
    /// Network address fields are a vector per account, where each entry is a vector of addresses
    /// encoded in a single BCS byte array.
    fun ol_create_initialize_validators(aptos_framework: &signer, validators: vector<ValidatorConfiguration>) {
        let i = 0;
        let num_validators = vector::length(&validators);

        // let validators_with_commission = vector::empty();

        while (i < num_validators) {
            ol_create_validator_accounts(aptos_framework, vector::borrow(&validators, i));

            i = i + 1;
        };

        stake::on_new_epoch();

        // create_initialize_validators_with_commission(aptos_framework, false, validators_with_commission);
    }

    fun create_initialize_validators(aptos_framework: &signer, validators: vector<ValidatorConfiguration>) {
        let i = 0;
        let num_validators = vector::length(&validators);

        let validators_with_commission = vector::empty();

      while (i < num_validators) {
          let validator_with_commission = ValidatorConfigurationWithCommission {
              validator_config: vector::pop_back(&mut validators),
              commission_percentage: 0,
              join_during_genesis: true,
          };
          vector::push_back(&mut validators_with_commission, validator_with_commission);

          i = i + 1;
      };

      create_initialize_validators_with_commission(aptos_framework, false, validators_with_commission);
    }

    fun ol_create_validator_accounts(
        aptos_framework: &signer,
        validator: &ValidatorConfiguration,
        // _use_staking_contract: bool,
    ) { // NOTE: after the accounts are created, the migration will restore the previous authentication keys.
        let owner = &ol_create_account(aptos_framework, validator.owner_address);
        // TODO: we probably don't need either of these accounts.
        ol_create_account(aptos_framework, validator.operator_address);
        ol_create_account(aptos_framework, validator.voter_address);

        // Initialize the stake pool and join the validator set.
        // let pool_address = if (use_staking_contract) {

        //     staking_contract::create_staking_contract(
        //         owner,
        //         validator.operator_address,
        //         validator.voter_address,
        //         validator.stake_amount,
        //         commission_config.commission_percentage,
        //         x"",
        //     );
        //     staking_contract::stake_pool_address(validator.owner_address, validator.operator_address)
        // } else
        let pool_address = {


            stake::initialize_stake_owner(
                owner,
                validator.stake_amount,
                validator.operator_address,
                validator.voter_address,
            );

            validator.owner_address
        };


        // if (commission_config.join_during_genesis) { // TODO: remove this check
            initialize_validator(pool_address, validator);
        // };
    }

    fun create_validator_accounts(
        aptos_framework: &signer,
        commission_config: &ValidatorConfigurationWithCommission,
        _use_staking_contract: bool,
    ) {
        let validator = &commission_config.validator_config;

        let owner = &create_account(aptos_framework, validator.owner_address, validator.stake_amount);
        // TODO: we probably don't need either of these accounts.
        create_account(aptos_framework, validator.operator_address, 0);
        create_account(aptos_framework, validator.voter_address, 0);

        // Initialize the stake pool and join the validator set.
        // let pool_address = if (use_staking_contract) {

        //     staking_contract::create_staking_contract(
        //         owner,
        //         validator.operator_address,
        //         validator.voter_address,
        //         validator.stake_amount,
        //         commission_config.commission_percentage,
        //         x"",
        //     );
        //     staking_contract::stake_pool_address(validator.owner_address, validator.operator_address)
        // } else
        let pool_address = {


            stake::initialize_stake_owner(
                owner,
                validator.stake_amount,
                validator.operator_address,
                validator.voter_address,
            );

            validator.owner_address
        };


        if (commission_config.join_during_genesis) { // TODO: remove this check
            initialize_validator(pool_address, validator);
        };
    }

    fun initialize_validator(pool_address: address, validator: &ValidatorConfiguration) {
        let operator = &create_signer(validator.operator_address);

        stake::rotate_consensus_key(
            operator,
            pool_address,
            validator.consensus_pubkey,
            validator.proof_of_possession,
        );

        stake::update_network_and_fullnode_addresses(
            operator,
            pool_address,
            validator.network_addresses,
            validator.full_node_network_addresses,
        );

        stake::join_validator_set_internal(operator, pool_address);

    }

    /// The last step of genesis.
    fun set_genesis_end(aptos_framework: &signer) {
        chain_status::set_genesis_end(aptos_framework);
    }

    #[verify_only]
    use std::features;

    #[verify_only]
    fun initialize_for_verification(
        gas_schedule: vector<u8>,
        chain_id: u8,
        initial_version: u64,
        consensus_config: vector<u8>,
        execution_config: vector<u8>,
        epoch_interval_microsecs: u64,
        minimum_stake: u64,
        maximum_stake: u64,
        recurring_lockup_duration_secs: u64,
        allow_validator_set_change: bool,
        rewards_rate: u64,
        rewards_rate_denominator: u64,
        voting_power_increase_limit: u64,
        aptos_framework: &signer,
        min_voting_threshold: u128,
        required_proposer_stake: u64,
        voting_duration_secs: u64,
        accounts: vector<AccountMap>,
        _employee_vesting_start: u64,
        _employee_vesting_period_duration: u64,
        _employees: vector<EmployeeAccountMap>,
        _validators: vector<ValidatorConfigurationWithCommission>
    ) {
        initialize(
            gas_schedule,
            chain_id,
            initial_version,
            consensus_config,
            execution_config,
            epoch_interval_microsecs,
            minimum_stake,
            maximum_stake,
            recurring_lockup_duration_secs,
            allow_validator_set_change,
            rewards_rate,
            rewards_rate_denominator,
            voting_power_increase_limit
        );
        features::change_feature_flags(aptos_framework, vector[1, 2], vector[]);
        initialize_aptos_coin(aptos_framework);
        aptos_governance::initialize_for_verification(
            aptos_framework,
            min_voting_threshold,
            required_proposer_stake,
            voting_duration_secs
        );
        create_accounts(aptos_framework, accounts);
        // create_employee_validators(employee_vesting_start, employee_vesting_period_duration, employees);
        // create_initialize_validators_with_commission(aptos_framework, true, validators);
        set_genesis_end(aptos_framework);
    }

    //////// 0L ////////
    #[test_only]
    public fun test_end_genesis(framework: &signer) {
        set_genesis_end(framework);
    }

    //////// 0L ////////
    #[test_only]
    public fun test_initalize_coins(framework: &signer) {
        initialize_aptos_coin(framework);
    }


    #[test_only]
    public fun setup() {
        initialize(
            x"000000000000000000", // empty gas schedule
            4u8, // TESTING chain ID
            0,
            x"12",
            x"13",
            1,
            0,
            1,
            1,
            true,
            1,
            1,
            30,
        )
    }

    #[test]
    fun test_setup() {
        setup();
        assert!(account::exists_at(@aptos_framework), 1);
        assert!(account::exists_at(@0x2), 1);
        assert!(account::exists_at(@0x3), 1);
        assert!(account::exists_at(@0x4), 1);
        assert!(account::exists_at(@0x5), 1);
        assert!(account::exists_at(@0x6), 1);
        assert!(account::exists_at(@0x7), 1);
        assert!(account::exists_at(@0x8), 1);
        assert!(account::exists_at(@0x9), 1);
        assert!(account::exists_at(@0xa), 1);
    }

    #[test(aptos_framework = @0x1)]
    fun test_create_account(aptos_framework: &signer) {
        setup();
        initialize_aptos_coin(aptos_framework);

        let addr = @0x121341; // 01 -> 0a are taken
        let test_signer_before = create_account(aptos_framework, addr, 15);
        let test_signer_after = create_account(aptos_framework, addr, 500);
        assert!(test_signer_before == test_signer_after, 0);
        assert!(coin::balance<AptosCoin>(addr) == 0, 1); //////// 0L ////////
    }

    #[test(aptos_framework = @0x1)]
    fun test_create_accounts(aptos_framework: &signer) {
        setup();
        initialize_aptos_coin(aptos_framework);

        // 01 -> 0a are taken
        let addr0 = @0x121341;
        let addr1 = @0x121345;

        let accounts = vector[
            AccountMap {
                account_address: addr0,
                balance: 12345,
            },
            AccountMap {
                account_address: addr1,
                balance: 67890,
            },
        ];

        create_accounts(aptos_framework, accounts);
        assert!(coin::balance<AptosCoin>(addr0) == 0, 0); //////// 0L //////// no coins minted at genesis
        assert!(coin::balance<AptosCoin>(addr1) == 0, 1); //////// 0L ////////

        create_account(aptos_framework, addr0, 23456);
        assert!(coin::balance<AptosCoin>(addr0) == 0, 2); //////// 0L ////////
    }
}
