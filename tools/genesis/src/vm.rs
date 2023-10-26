#![allow(clippy::too_many_arguments)]

use diem_crypto::{ed25519::Ed25519PublicKey, HashValue};
use diem_framework::{self, ReleaseBundle};
use diem_gas::{
    AbstractValueSizeGasParameters, ChangeSetConfigs, NativeGasParameters,
    LATEST_GAS_FEATURE_VERSION,
};
use diem_types::{
    chain_id::{ChainId, NamedChain},
    on_chain_config::{
        Features, GasScheduleV2, OnChainConsensusConfig, OnChainExecutionConfig, TimedFeatures,
    },
    transaction::ChangeSet,
};
use diem_vm::{
    data_cache::AsMoveResolver,
    move_vm_ext::{MoveVmExt, SessionId},
};
use diem_vm_genesis::{
    create_and_initialize_validators, default_gas_schedule, emit_new_block_and_epoch_event,
    genesis_context::GenesisStateView, initialize, initialize_diem_coin, initialize_features,
    initialize_on_chain_governance, publish_framework, set_genesis_end, validate_genesis_config,
    verify_genesis_write_set, GenesisConfiguration, Validator, GENESIS_KEYPAIR,
};
use libra_types::{legacy_types::legacy_recovery::LegacyRecovery, ol_progress::OLProgress};

use crate::{
    genesis_functions::{rounding_mint, set_final_supply, set_validator_baseline_reward},
    supply::{populate_supply_stats_from_legacy, SupplySettings},
};

/// set the genesis parameters
/// NOTE: many of the parameters are ignored in libra_framework
/// but are kept for api compatibility.
pub fn libra_genesis_default(chain: NamedChain) -> GenesisConfiguration {
    let epoch_duration_secs = match chain {
        NamedChain::MAINNET => 24 * 60 * 60, // one day
        NamedChain::TESTING => 2 * 60,       // for CI suite: two mins
        _ => 15 * 60, // for all testnets, not using mainnet settings, 15 mins
    };
    GenesisConfiguration {
        allow_new_validators: true,
        epoch_duration_secs,
        is_test: false,
        // hacky, but annoying surgery otherwise.
        // TODO: do the surgery
        max_stake: 0,                                            // no-op
        min_stake: 0,                                            // no-op
        min_voting_threshold: 0,                                 // no-op
        recurring_lockup_duration_secs: epoch_duration_secs + 1, // no-op
        required_proposer_stake: 0,                              // no-op
        rewards_apy_percentage: 1,                               // no-op
        voting_duration_secs: 1,                                 // no-op
        voting_power_increase_limit: 1,                          // no-op
        employee_vesting_start: 1,                               // no-op lol
        employee_vesting_period_duration: 1,                     // no-op srsly
    }
}

pub fn migration_genesis(
    validators: &[Validator],
    recovery: Option<&[LegacyRecovery]>,
    framework: &ReleaseBundle,
    chain_id: ChainId,
    supply_settings: &SupplySettings,
    genesis_config: &GenesisConfiguration,
) -> anyhow::Result<ChangeSet> {
    let genesis = encode_genesis_change_set(
        &GENESIS_KEYPAIR.1,
        validators,
        recovery,
        framework,
        chain_id,
        genesis_config,
        &OnChainConsensusConfig::default(),
        &OnChainExecutionConfig::default(),
        &default_gas_schedule(),
        supply_settings,
    );

    Ok(genesis)
}

/// Generates a genesis using the recovery file for hard forks.
pub fn encode_genesis_change_set(
    _core_resources_key: &Ed25519PublicKey,
    validators: &[Validator],
    recovery: Option<&[LegacyRecovery]>,
    framework: &ReleaseBundle,
    chain_id: ChainId,
    genesis_config: &GenesisConfiguration,
    consensus_config: &OnChainConsensusConfig,
    execution_config: &OnChainExecutionConfig,
    gas_schedule: &GasScheduleV2,
    supply_settings: &SupplySettings,
) -> ChangeSet {
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
    let mut session = move_vm.new_session(&data_cache, SessionId::genesis(id1), true);

    // On-chain genesis process.
    initialize(
        &mut session,
        chain_id,
        genesis_config,
        consensus_config,
        execution_config,
        gas_schedule,
    );

    initialize_features(&mut session);

    // TODO: consolidate with set_final_supply
    initialize_diem_coin(&mut session);

    // final supply must be set after coin is initialized, but before any
    // accounts are created
    set_final_supply(&mut session, supply_settings);

    initialize_on_chain_governance(&mut session, genesis_config);

    if let Some(r) = recovery {
        if !r.is_empty() {
            let mut supply = populate_supply_stats_from_legacy(r, &supply_settings.map_dd_to_slow)
                .expect("could not parse supply from legacy file");

            supply
                .set_ratios_from_settings(supply_settings)
                .expect("could not set supply ratios from settings");

            crate::genesis_functions::genesis_migrate_all_users(&mut session, r, &supply)
                .expect("could not migrate users");

            // need to set the baseline reward based on supply settings
            set_validator_baseline_reward(&mut session, supply.epoch_reward_base_case as u64);
        }
    }
    OLProgress::complete("user migration complete");

    //////// 0L ////////
    // moved this to happen after legacy account migration, since the validators need to have their accounts migrated as well, including the mapping of legacy address to the authkey (which no longer derives to the previous same address).
    // Note: the operator accounts at genesis will be different.
    create_and_initialize_validators(&mut session, validators);

    //////// 0L ////////
    // need to ajust for rounding issues from target supply
    rounding_mint(&mut session, supply_settings);

    // // add some coins in each validator account.
    // if chain_id != ChainId::new(1) || option_env!("LIBRA_CI").is_some() {
    //     mint_genesis_bootstrap_coin(&mut session, validators);
    // }

    OLProgress::complete("initialized genesis validators");

    let spin = OLProgress::spin_steady(100, "publishing framework".to_string());

    set_genesis_end(&mut session);

    // Reconfiguration should happen after all on-chain invocations.
    emit_new_block_and_epoch_event(&mut session);
    let configs = ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION);
    let cs1 = session.finish(&mut (), &configs).unwrap();

    // Publish the framework, using a different session id, in case both scripts creates tables
    let state_view = GenesisStateView::new();
    let data_cache = state_view.as_move_resolver();

    let mut id2_arr = [0u8; 32];
    id2_arr[31] = 1;
    let id2 = HashValue::new(id2_arr);
    let mut session = move_vm.new_session(&data_cache, SessionId::genesis(id2), true);
    publish_framework(&mut session, framework);
    let cs2 = session.finish(&mut (), &configs).unwrap();
    let change_set = cs1.squash(cs2, &configs).unwrap();

    let (write_set, delta_change_set, events) = change_set.unpack();

    // Publishing stdlib should not produce any deltas around aggregators and map to write ops and
    // not deltas. The second session only publishes the framework module bundle, which should not
    // produce deltas either.
    assert!(
        delta_change_set.is_empty(),
        "non-empty delta change set in genesis"
    );

    assert!(!write_set.iter().any(|(_, op)| op.is_deletion()));
    verify_genesis_write_set(&events);
    let change_set = ChangeSet::new(write_set, events);
    spin.finish_and_clear();
    change_set
}
