use libra_genesis_tools::{
    genesis::{make_recovery_genesis_from_vec_legacy_recovery, save_genesis},
    parse_json,
    supply::SupplySettings,
};
use std::path::PathBuf;
use zapatos_types::{
    chain_id::ChainId,
    on_chain_config::OnChainConfig,
    on_chain_config::ValidatorSet,
    state_store::state_key::StateKey,
    transaction::{Transaction, WriteSetPayload},
    write_set::TransactionWrite,
};
use zapatos_vm_genesis::{TestValidator, Validator};

use libra_framework::head_release_bundle;
use libra_types::legacy_types::legacy_address::LegacyAddress;
use std::fs;

#[test]
fn end_to_end_single() {

    let blob = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/genesis.blob");

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");

    let recovery = parse_json::parse(p).unwrap();

    let num_vals = 6;
    let test_validators = TestValidator::new_test_set(Some(num_vals), Some(100_000_000_000_000));
    let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();

    let mut supply_settings = SupplySettings::default();
    supply_settings.target_future_uses = 0.70;
    supply_settings.map_dd_to_slow = vec![
        // FTW
        "3A6C51A0B786D644590E8A21591FA8E2"
            .parse::<LegacyAddress>()
            .unwrap(),
        // tip jar
        "2B0E8325DEA5BE93D856CFDE2D0CBA12"
            .parse::<LegacyAddress>()
            .unwrap(),
    ];

    let tx = make_recovery_genesis_from_vec_legacy_recovery(
        Some(&recovery),
        &validators,
        &head_release_bundle(),
        ChainId::test(),
        Some(supply_settings),
    )
    .expect("could not write genesis.blob");

    save_genesis(&tx, &blob).unwrap();
    assert!(blob.exists(), "genesis.blob does not exist");

    let gen_bytes = fs::read(blob).unwrap();

    match bcs::from_bytes(&gen_bytes).unwrap() {
        Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset)) => {
            let state_key =
                StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
            let bytes = recovery_changeset
                .write_set()
                .get(&state_key)
                .unwrap()
                .extract_raw_bytes()
                .unwrap();
            let validator_set: ValidatorSet = bcs::from_bytes(&bytes).unwrap();
            // dbg!(&validator_set.active_validators().len());
            assert!(
                validator_set.active_validators().len() == num_vals,
                "validator set count does not match"
            );
        }
        _ => panic!("not a genesis transaction"),
    }

    // drop.maybe_cleanup();
}

#[test]
fn end_to_end_all() {
    let blob = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/genesis.blob");

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let recovery = parse_json::parse(p).unwrap();

    let num_vals = 6;
    let test_validators = TestValidator::new_test_set(Some(num_vals), Some(100_000_000_000_000));
    let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();

    let mut supply_settings = SupplySettings::default();
    supply_settings.target_future_uses = 0.70;
    supply_settings.map_dd_to_slow = vec![
        // FTW
        "3A6C51A0B786D644590E8A21591FA8E2"
            .parse::<LegacyAddress>()
            .unwrap(),
        // tip jar
        "2B0E8325DEA5BE93D856CFDE2D0CBA12"
            .parse::<LegacyAddress>()
            .unwrap(),
    ];

    let tx = make_recovery_genesis_from_vec_legacy_recovery(
        Some(&recovery),
        &validators,
        &head_release_bundle(),
        ChainId::test(),
        Some(supply_settings),
    )
    .expect("could not write genesis.blob");

    save_genesis(&tx, &blob).unwrap();
    assert!(blob.exists(), "genesis.blob does not exist");

    let gen_bytes = fs::read(blob).unwrap();

    match bcs::from_bytes(&gen_bytes).unwrap() {
        Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset)) => {
            let state_key =
                StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
            let bytes = recovery_changeset
                .write_set()
                .get(&state_key)
                .unwrap()
                .extract_raw_bytes()
                .unwrap();
            let validator_set: ValidatorSet = bcs::from_bytes(&bytes).unwrap();
            // dbg!(&validator_set.active_validators().len());
            assert!(
                validator_set.active_validators().len() == num_vals,
                "validator set count does not match"
            );
        }
        _ => panic!("not a genesis transaction"),
    }

    // drop.maybe_cleanup();
}
