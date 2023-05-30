module aptos_framework::genesis {
    use std::error;
    // use std::fixed_point32;
    use std::vector;

    // use aptos_std::simple_map;
    use aptos_std::debug::print;

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
    // use aptos_framework::staking_contract;
    // use aptos_framework::staking_config;
    use aptos_framework::state_storage;
    use aptos_framework::storage_gas;
    use aptos_framework::timestamp;
    // use aptos_framework::transaction_fee;
    use aptos_framework::transaction_validation;
    use aptos_framework::version;

    //////// 0L ////////
    use aptos_framework::validator_universe;
    use ol_framework::musical_chairs;
    use ol_framework::proof_of_fee;
    use ol_framework::slow_wallet;
    use ol_framework::gas_coin;

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
        //todo: genesis seats
        let genesis_seats = 10;
        musical_chairs::initialize(&aptos_framework_account, genesis_seats);
        proof_of_fee::init_genesis_baseline_reward(&aptos_framework_account);
        slow_wallet::initialize(&aptos_framework_account);
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
        let (burn_cap, mint_cap) = gas_coin::initialize(aptos_framework);
        // Give stake module MintCapability<AptosCoin> so it can mint rewards.
        // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);
    }

    // TODO: 0L: replace this with gas coin. using vendor's to preserve tests while WIP.
    /// Only called for testnets and e2e tests.
    fun initialize_core_resources_and_aptos_coin(
        aptos_framework: &signer,
        core_resources_auth_key: vector<u8>,
    ) {
        let (burn_cap, mint_cap) = aptos_coin::initialize(aptos_framework);

        
        
        // Give stake module MintCapability<AptosCoin> so it can mint rewards.
        // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
        coin::destroy_mint_cap(mint_cap);
        // Give transaction_fee module BurnCapability<AptosCoin> so it can burn gas.
        // transaction_fee::store_aptos_coin_burn_cap(aptos_framework, burn_cap);
        coin::destroy_burn_cap(burn_cap);

        let core_resources = account::create_account(@core_resources);
        account::rotate_authentication_key_internal(&core_resources, core_resources_auth_key);
        aptos_coin::configure_accounts_for_test(aptos_framework, &core_resources, mint_cap);

        let (burn_cap, mint_cap) = gas_coin::initialize(aptos_framework);
        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);

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
    fun create_account(aptos_framework: &signer, account_address: address, balance: u64): signer {
        if (account::exists_at(account_address)) {
            create_signer(account_address)
        } else {
            let account = account::create_account(account_address);
            coin::register<AptosCoin>(&account);
            aptos_coin::mint(aptos_framework, account_address, balance);
            account
        }
    }

    // fun create_employee_validators(
    //     employee_vesting_start: u64,
    //     employee_vesting_period_duration: u64,
    //     employees: vector<EmployeeAccountMap>,
    // ) {
    //     let i = 0;
    //     let num_employee_groups = vector::length(&employees);
    //     let unique_accounts = vector::empty();

    //     while (i < num_employee_groups) {
    //         let j = 0;
    //         let employee_group = vector::borrow(&employees, i);
    //         let num_employees_in_group = vector::length(&employee_group.accounts);

    //         let buy_ins = simple_map::create();

    //         while (j < num_employees_in_group) {
    //             let account = vector::borrow(&employee_group.accounts, j);
    //             assert!(
    //                 !vector::contains(&unique_accounts, account),
    //                 error::already_exists(EDUPLICATE_ACCOUNT),
    //             );
    //             vector::push_back(&mut unique_accounts, *account);

    //             let employee = create_signer(*account);
    //             let total = coin::balance<AptosCoin>(*account);
    //             let coins = coin::withdraw<AptosCoin>(&employee, total);
    //             simple_map::add(&mut buy_ins, *account, coins);

    //             j = j + 1;
    //         };

    //         let j = 0;
    //         let num_vesting_events = vector::length(&employee_group.vesting_schedule_numerator);
    //         let schedule = vector::empty();

    //         while (j < num_vesting_events) {
    //             let numerator = vector::borrow(&employee_group.vesting_schedule_numerator, j);
    //             let event = fixed_point32::create_from_rational(*numerator, employee_group.vesting_schedule_denominator);
    //             vector::push_back(&mut schedule, event);

    //             j = j + 1;
    //         };

    //         let vesting_schedule = vesting::create_vesting_schedule(
    //             schedule,
    //             employee_vesting_start,
    //             employee_vesting_period_duration,
    //         );

    //         let admin = employee_group.validator.validator_config.owner_address;
    //         let admin_signer = &create_signer(admin);
    //         let contract_address = vesting::create_vesting_contract(
    //             admin_signer,
    //             &employee_group.accounts,
    //             buy_ins,
    //             vesting_schedule,
    //             admin,
    //             employee_group.validator.validator_config.operator_address,
    //             employee_group.validator.validator_config.voter_address,
    //             employee_group.validator.commission_percentage,
    //             x"",
    //         );
    //         let pool_address = vesting::stake_pool_address(contract_address);

    //         if (employee_group.beneficiary_resetter != @0x0) {
    //             // vesting::set_beneficiary_resetter(admin_signer, contract_address, employee_group.beneficiary_resetter);
    //         };

    //         let validator = &employee_group.validator.validator_config;
    //         assert!(
    //             account::exists_at(validator.owner_address),
    //             error::not_found(EACCOUNT_DOES_NOT_EXIST),
    //         );
    //         assert!(
    //             account::exists_at(validator.operator_address),
    //             error::not_found(EACCOUNT_DOES_NOT_EXIST),
    //         );
    //         assert!(
    //             account::exists_at(validator.voter_address),
    //             error::not_found(EACCOUNT_DOES_NOT_EXIST),
    //         );
    //         if (employee_group.validator.join_during_genesis) {
    //             initialize_validator(pool_address, validator);
    //         };

    //         i = i + 1;
    //     }
    // }

    fun create_initialize_validators_with_commission(
        aptos_framework: &signer,
        use_staking_contract: bool,
        validators: vector<ValidatorConfigurationWithCommission>,
    ) {
        let i = 0;
        let num_validators = vector::length(&validators);
        print(&10001);
        while (i < num_validators) {
          print(&10002);
            let validator = vector::borrow(&validators, i);
            create_initialize_validator(aptos_framework, validator, use_staking_contract);

            i = i + 1;
        };
        print(&10003);
        // Destroy the aptos framework account's ability to mint coins now that we're done with setting up the initial
        // validators.
        // aptos_coin::destroy_mint_cap(aptos_framework);
        // print(&10004);

        stake::on_new_epoch();
        print(&10005);
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

    fun create_initialize_validator(
        aptos_framework: &signer,
        commission_config: &ValidatorConfigurationWithCommission,
        _use_staking_contract: bool,
    ) {
        let validator = &commission_config.validator_config;
        print(&20001);
        let owner = &create_account(aptos_framework, validator.owner_address, validator.stake_amount);
        create_account(aptos_framework, validator.operator_address, 0);
        create_account(aptos_framework, validator.voter_address, 0);
        print(&20002);
        // Initialize the stake pool and join the validator set.
        // let pool_address = if (use_staking_contract) {
        //   print(&2000201);
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
            print(&2000202);

            stake::initialize_stake_owner(
                owner,
                validator.stake_amount,
                validator.operator_address,
                validator.voter_address,
            );
            print(&2000203);
            validator.owner_address
        };
        print(&20003);

        if (commission_config.join_during_genesis) {
            print(&20004);
            initialize_validator(pool_address, validator);
        };
    }

    fun initialize_validator(pool_address: address, validator: &ValidatorConfiguration) {
        let operator = &create_signer(validator.operator_address);
        print(&30001);
        stake::rotate_consensus_key(
            operator,
            pool_address,
            validator.consensus_pubkey,
            validator.proof_of_possession,
        );
        print(&30002);
        stake::update_network_and_fullnode_addresses(
            operator,
            pool_address,
            validator.network_addresses,
            validator.full_node_network_addresses,
        );
        print(&30003);
        stake::join_validator_set_internal(operator, pool_address);
        print(&30004);
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
        validators: vector<ValidatorConfigurationWithCommission>
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
        create_initialize_validators_with_commission(aptos_framework, true, validators);
        set_genesis_end(aptos_framework);
    }

    //////// 0L ////////
    #[test_only]
    public fun test_end_genesis(framework: &signer) {
        set_genesis_end(framework);
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
        assert!(coin::balance<AptosCoin>(addr) == 15, 1);
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
        assert!(coin::balance<AptosCoin>(addr0) == 12345, 0);
        assert!(coin::balance<AptosCoin>(addr1) == 67890, 1);

        create_account(aptos_framework, addr0, 23456);
        assert!(coin::balance<AptosCoin>(addr0) == 12345, 2);
    }
}
