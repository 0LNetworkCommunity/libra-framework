use ol_types::legacy_recovery::LegacyRecovery;
use zapatos_crypto::{ed25519::Ed25519PublicKey, HashValue};
use zapatos_framework::{self, ReleaseBundle};
use zapatos_gas::{
    AbstractValueSizeGasParameters, ChangeSetConfigs, NativeGasParameters,
    LATEST_GAS_FEATURE_VERSION,
};
use zapatos_types::{
    chain_id::ChainId,
    on_chain_config::{Features, GasScheduleV2, OnChainConsensusConfig, TimedFeatures},
    transaction::{ChangeSet, Transaction, WriteSetPayload},
};
use zapatos_vm::{
    data_cache::AsMoveResolver,
    move_vm_ext::{MoveVmExt, SessionId},
};
use zapatos_vm_genesis::{
    allow_core_resources_to_set_version, create_accounts, create_and_initialize_validators,
    create_and_initialize_validators_with_commission, create_employee_validators,
    default_gas_schedule, emit_new_block_and_epoch_event, genesis_context::GenesisStateView,
    initialize, initialize_aptos_coin, initialize_core_resources_and_aptos_coin,
    initialize_features, initialize_on_chain_governance, mainnet_genesis_config, publish_framework,
    set_genesis_end, validate_genesis_config, verify_genesis_write_set, AccountBalance,
    EmployeePool, GenesisConfiguration, Validator, ValidatorWithCommissionRate, GENESIS_KEYPAIR,
};

use crate::convert_types;

pub fn zapatos_mainnet_genesis(
    validators: &[Validator],
    recovery: Option<&[LegacyRecovery]>,
) -> anyhow::Result<ChangeSet> {
    let genesis = encode_zapatos_recovery_genesis_change_set(
        &GENESIS_KEYPAIR.1,
        validators,
        recovery,
        zapatos_framework::testnet_release_bundle(),
        ChainId::test(),
        &mainnet_genesis_config(),
        &OnChainConsensusConfig::default(),
        &default_gas_schedule(),
    );

    Ok(genesis)
}

/// Generates a genesis using the recovery file for hard forks.
pub fn encode_zapatos_recovery_genesis_change_set(
    core_resources_key: &Ed25519PublicKey,
    validators: &[Validator],
    recovery: Option<&[LegacyRecovery]>,
    framework: &ReleaseBundle,
    chain_id: ChainId,
    genesis_config: &GenesisConfiguration,
    consensus_config: &OnChainConsensusConfig,
    gas_schedule: &GasScheduleV2,
) -> ChangeSet {
    if let Some(r) = recovery {
        r.into_iter().for_each(|a| {
            // dbg!(a.account);
            if let Some(acc) = a.account {
                dbg!(convert_types::convert_account(acc).unwrap());
            }
        })
    }

    validate_genesis_config(genesis_config);

    // Create a Move VM session so we can invoke on-chain genesis intializations.
    let mut state_view = GenesisStateView::new();
    for (module_bytes, module) in framework.code_and_compiled_modules() {
        state_view.add_module(&module.self_id(), module_bytes);
    }
    let data_cache = state_view.as_move_resolver();
    let move_vm = MoveVmExt::new(
        NativeGasParameters::zeros(),
        AbstractValueSizeGasParameters::zeros(),
        LATEST_GAS_FEATURE_VERSION,
        ChainId::test().id(),
        Features::default(),
        TimedFeatures::enable_all(),
    )
    .unwrap();
    let id1 = HashValue::zero();
    let mut session = move_vm.new_session(&data_cache, SessionId::genesis(id1));

    // On-chain genesis process.
    initialize(
        &mut session,
        chain_id,
        genesis_config,
        consensus_config,
        gas_schedule,
    );
    initialize_features(&mut session);
    // dbg!(&genesis_config.is_test);
    if genesis_config.is_test {
        initialize_core_resources_and_aptos_coin(&mut session, core_resources_key);
    } else {
        initialize_aptos_coin(&mut session);
    }
    dbg!(&"initialize_on_chain_governance");
    initialize_on_chain_governance(&mut session, genesis_config);
    dbg!("create_and_initialize_validators");
    create_and_initialize_validators(&mut session, validators);
    if genesis_config.is_test {
        allow_core_resources_to_set_version(&mut session);
    }

    set_genesis_end(&mut session);

    // Reconfiguration should happen after all on-chain invocations.
    emit_new_block_and_epoch_event(&mut session);

    let cs1 = session
        .finish(
            &mut (),
            &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
        )
        .unwrap();

    let state_view = GenesisStateView::new();
    let data_cache = state_view.as_move_resolver();

    // Publish the framework, using a different session id, in case both scripts creates tables
    let mut id2_arr = [0u8; 32];
    id2_arr[31] = 1;
    let id2 = HashValue::new(id2_arr);
    let mut session = move_vm.new_session(&data_cache, SessionId::genesis(id2));
    publish_framework(&mut session, framework);
    let cs2 = session
        .finish(
            &mut (),
            &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
        )
        .unwrap();

    let change_set_ext = cs1.squash(cs2).unwrap();

    let (delta_change_set, change_set) = change_set_ext.into_inner();

    // Publishing stdlib should not produce any deltas around aggregators and map to write ops and
    // not deltas. The second session only publishes the framework module bundle, which should not
    // produce deltas either.
    assert!(
        delta_change_set.is_empty(),
        "non-empty delta change set in genesis"
    );

    assert!(!change_set
        .write_set()
        .iter()
        .any(|(_, op)| op.is_deletion()));
    verify_genesis_write_set(change_set.events());
    change_set
}

pub fn encode_aptos_mainnet_genesis_transaction(
    accounts: &[AccountBalance],
    employees: &[EmployeePool],
    validators: &[ValidatorWithCommissionRate],
    framework: &ReleaseBundle,
    chain_id: ChainId,
    genesis_config: &GenesisConfiguration,
) -> Transaction {
    assert!(!genesis_config.is_test, "This is mainnet!");
    validate_genesis_config(genesis_config);

    // Create a Move VM session so we can invoke on-chain genesis intializations.
    let mut state_view = GenesisStateView::new();
    for (module_bytes, module) in framework.code_and_compiled_modules() {
        state_view.add_module(&module.self_id(), module_bytes);
    }
    let data_cache = state_view.as_move_resolver();
    let move_vm = MoveVmExt::new(
        NativeGasParameters::zeros(),
        AbstractValueSizeGasParameters::zeros(),
        LATEST_GAS_FEATURE_VERSION,
        ChainId::test().id(),
        Features::default(),
        TimedFeatures::enable_all(),
    )
    .unwrap();
    let id1 = HashValue::zero();
    let mut session = move_vm.new_session(&data_cache, SessionId::genesis(id1));

    // On-chain genesis process.
    let consensus_config = OnChainConsensusConfig::default();
    let gas_schedule = default_gas_schedule();

    initialize(
        &mut session,
        chain_id,
        genesis_config,
        &consensus_config,
        &gas_schedule,
    );
    initialize_features(&mut session);
    initialize_aptos_coin(&mut session);
    initialize_on_chain_governance(&mut session, genesis_config);
    create_accounts(&mut session, accounts);
    create_employee_validators(&mut session, employees, genesis_config);
    create_and_initialize_validators_with_commission(&mut session, validators);
    set_genesis_end(&mut session);

    // Reconfiguration should happen after all on-chain invocations.
    emit_new_block_and_epoch_event(&mut session);

    let cs1 = session
        .finish(
            &mut (),
            &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
        )
        .unwrap();

    // Publish the framework, using a different session id, in case both scripts creates tables
    let state_view = GenesisStateView::new();
    let data_cache = state_view.as_move_resolver();

    let mut id2_arr = [0u8; 32];
    id2_arr[31] = 1;
    let id2 = HashValue::new(id2_arr);
    let mut session = move_vm.new_session(&data_cache, SessionId::genesis(id2));
    publish_framework(&mut session, framework);
    let cs2 = session
        .finish(
            &mut (),
            &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
        )
        .unwrap();
    let change_set_ext = cs1.squash(cs2).unwrap();

    let (delta_change_set, change_set) = change_set_ext.into_inner();

    // Publishing stdlib should not produce any deltas around aggregators and map to write ops and
    // not deltas. The second session only publishes the framework module bundle, which should not
    // produce deltas either.
    assert!(
        delta_change_set.is_empty(),
        "non-empty delta change set in genesis"
    );

    assert!(!change_set
        .write_set()
        .iter()
        .any(|(_, op)| op.is_deletion()));
    verify_genesis_write_set(change_set.events());
    Transaction::GenesisTransaction(WriteSetPayload::Direct(change_set))
}

#[test]
pub fn test_mainnet_end_to_end() {
    use zapatos_cached_packages;
    use zapatos_types::{
        account_address::{self, AccountAddress},
        on_chain_config::{OnChainConfig, ValidatorSet},
        state_store::state_key::StateKey,
        write_set::{TransactionWrite, WriteSet},
    };
    use zapatos_vm_genesis::TestValidator;
    const APTOS_COINS_BASE_WITH_DECIMALS: u64 = u64::pow(10, 8);

    let balance = 10_000_000 * APTOS_COINS_BASE_WITH_DECIMALS;
    let non_validator_balance = 10 * APTOS_COINS_BASE_WITH_DECIMALS;

    // currently just test that all functions have the right interface
    let account44 = AccountAddress::from_hex_literal("0x44").unwrap();
    let account45 = AccountAddress::from_hex_literal("0x45").unwrap();
    let account46 = AccountAddress::from_hex_literal("0x46").unwrap();
    let account47 = AccountAddress::from_hex_literal("0x47").unwrap();
    let account48 = AccountAddress::from_hex_literal("0x48").unwrap();
    let account49 = AccountAddress::from_hex_literal("0x49").unwrap();
    let operator0 = AccountAddress::from_hex_literal("0x100").unwrap();
    let operator1 = AccountAddress::from_hex_literal("0x101").unwrap();
    let operator2 = AccountAddress::from_hex_literal("0x102").unwrap();
    let operator3 = AccountAddress::from_hex_literal("0x103").unwrap();
    let operator4 = AccountAddress::from_hex_literal("0x104").unwrap();
    let operator5 = AccountAddress::from_hex_literal("0x105").unwrap();
    let voter0 = AccountAddress::from_hex_literal("0x200").unwrap();
    let voter1 = AccountAddress::from_hex_literal("0x201").unwrap();
    let voter2 = AccountAddress::from_hex_literal("0x202").unwrap();
    let voter3 = AccountAddress::from_hex_literal("0x203").unwrap();
    let admin0 = AccountAddress::from_hex_literal("0x300").unwrap();
    let admin1 = AccountAddress::from_hex_literal("0x301").unwrap();
    let admin2 = AccountAddress::from_hex_literal("0x302").unwrap();

    let accounts = vec![
        AccountBalance {
            account_address: account44,
            balance,
        },
        AccountBalance {
            account_address: account45,
            balance: balance * 3, // Three times the balance so it can host 2 operators.
        },
        AccountBalance {
            account_address: account46,
            balance,
        },
        AccountBalance {
            account_address: account47,
            balance,
        },
        AccountBalance {
            account_address: account48,
            balance,
        },
        AccountBalance {
            account_address: account49,
            balance,
        },
        AccountBalance {
            account_address: admin0,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: admin1,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: admin2,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: operator0,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: operator1,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: operator2,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: operator3,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: operator4,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: operator5,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: voter0,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: voter1,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: voter2,
            balance: non_validator_balance,
        },
        AccountBalance {
            account_address: voter3,
            balance: non_validator_balance,
        },
    ];

    let test_validators = TestValidator::new_test_set(Some(6), Some(balance * 9 / 10));
    let mut employee_validator_1 = test_validators[0].data.clone();
    employee_validator_1.owner_address = admin0;
    employee_validator_1.operator_address = operator0;
    employee_validator_1.voter_address = voter0;
    let mut employee_validator_2 = test_validators[1].data.clone();
    employee_validator_2.owner_address = admin1;
    employee_validator_2.operator_address = operator1;
    employee_validator_2.voter_address = voter1;
    let mut zero_commission_validator = test_validators[2].data.clone();
    zero_commission_validator.owner_address = account44;
    zero_commission_validator.operator_address = operator2;
    zero_commission_validator.voter_address = voter2;
    let mut same_owner_validator_1 = test_validators[3].data.clone();
    same_owner_validator_1.owner_address = account45;
    same_owner_validator_1.operator_address = operator3;
    same_owner_validator_1.voter_address = voter3;
    let mut same_owner_validator_2 = test_validators[4].data.clone();
    same_owner_validator_2.owner_address = account45;
    same_owner_validator_2.operator_address = operator4;
    same_owner_validator_2.voter_address = voter3;
    let mut same_owner_validator_3 = test_validators[5].data.clone();
    same_owner_validator_3.owner_address = account45;
    same_owner_validator_3.operator_address = operator5;
    same_owner_validator_3.voter_address = voter3;

    let employees = vec![
        EmployeePool {
            accounts: vec![account46, account47],
            validator: ValidatorWithCommissionRate {
                validator: employee_validator_1,
                validator_commission_percentage: 10,
                join_during_genesis: true,
            },
            vesting_schedule_numerators: vec![3, 3, 3, 3, 1],
            vesting_schedule_denominator: 48,
            beneficiary_resetter: AccountAddress::ZERO,
        },
        EmployeePool {
            accounts: vec![account48, account49],
            validator: ValidatorWithCommissionRate {
                validator: employee_validator_2,
                validator_commission_percentage: 10,
                join_during_genesis: false,
            },
            vesting_schedule_numerators: vec![3, 3, 3, 3, 1],
            vesting_schedule_denominator: 48,
            beneficiary_resetter: account44,
        },
    ];

    let validators = vec![
        ValidatorWithCommissionRate {
            validator: same_owner_validator_1,
            validator_commission_percentage: 10,
            join_during_genesis: true,
        },
        ValidatorWithCommissionRate {
            validator: same_owner_validator_2,
            validator_commission_percentage: 15,
            join_during_genesis: true,
        },
        ValidatorWithCommissionRate {
            validator: same_owner_validator_3,
            validator_commission_percentage: 10,
            join_during_genesis: false,
        },
        ValidatorWithCommissionRate {
            validator: zero_commission_validator,
            validator_commission_percentage: 0,
            join_during_genesis: true,
        },
    ];

    let transaction = encode_aptos_mainnet_genesis_transaction(
        &accounts,
        &employees,
        &validators,
        zapatos_cached_packages::head_release_bundle(),
        ChainId::mainnet(),
        &mainnet_genesis_config(),
    );

    let direct_writeset = if let Transaction::GenesisTransaction(direct_writeset) = transaction {
        direct_writeset
    } else {
        panic!("Invalid GenesisTransaction");
    };

    let changeset = if let WriteSetPayload::Direct(changeset) = direct_writeset {
        changeset
    } else {
        panic!("Invalid WriteSetPayload");
    };

    let WriteSet::V0(writeset) = changeset.write_set();

    let state_key =
        StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
    let bytes = writeset
        .get(&state_key)
        .unwrap()
        .extract_raw_bytes()
        .unwrap();
    let validator_set: ValidatorSet = bcs::from_bytes(&bytes).unwrap();
    let validator_set_addresses = validator_set
        .active_validators
        .iter()
        .map(|v| v.account_address)
        .collect::<Vec<_>>();

    // let zero_commission_validator_pool_address =
    //     account_address::default_stake_pool_address(account44, operator2);
    // let same_owner_validator_1_pool_address =
    //     account_address::default_stake_pool_address(account45, operator3);
    // let same_owner_validator_2_pool_address =
    //     account_address::default_stake_pool_address(account45, operator4);
    let same_owner_validator_3_pool_address =
        account_address::default_stake_pool_address(account45, operator5);
    // let employee_1_pool_address =
    //     account_address::create_vesting_pool_address(admin0, operator0, 0, &[]);
    // let employee_2_pool_address =
    //     account_address::create_vesting_pool_address(admin1, operator1, 0, &[]);

    // dbg!(&validator_set_addresses);
    // assert!(validator_set_addresses.contains(&zero_commission_validator_pool_address));
    // assert!(validator_set_addresses.contains(&employee_1_pool_address));
    // This validator should not be in the genesis validator set as they specified
    // join_during_genesis = false.
    // assert!(!validator_set_addresses.contains(&employee_2_pool_address));
    // assert!(validator_set_addresses.contains(&same_owner_validator_1_pool_address));
    // assert!(validator_set_addresses.contains(&same_owner_validator_2_pool_address));
    // This validator should not be in the genesis validator set as they specified
    // join_during_genesis = false.
    assert!(!validator_set_addresses.contains(&same_owner_validator_3_pool_address));
}
