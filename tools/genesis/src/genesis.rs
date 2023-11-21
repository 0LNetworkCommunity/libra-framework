//! create a genesis from a LegacyRecovery struct

use crate::{supply::SupplySettings, vm::migration_genesis};
use anyhow::Error;
use diem_framework::ReleaseBundle;
use diem_types::{
    chain_id::ChainId,
    transaction::{Transaction, WriteSetPayload},
};
use diem_vm_genesis::{GenesisConfiguration, Validator};
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[cfg(test)]
use crate::vm::libra_genesis_default;
#[cfg(test)]
use diem_types::chain_id::NamedChain;

/// Make a recovery genesis blob
pub fn make_recovery_genesis_from_vec_legacy_recovery(
    recovery: &mut [LegacyRecovery],
    genesis_vals: &[Validator],
    framework_release: &ReleaseBundle,
    chain_id: ChainId,
    supply_settings: Option<SupplySettings>,
    genesis_config: &GenesisConfiguration,
) -> Result<Transaction, Error> {
    let supply_settings = supply_settings.unwrap_or_default();
    // Note: For `recovery` on a real upgrade or fork, we want to include all user accounts. If a None is passed, then we'll just run the default genesis
    // which only uses the validator accounts.
    let recovery_changeset = migration_genesis(
        genesis_vals,
        recovery,
        framework_release,
        chain_id,
        &supply_settings,
        genesis_config,
    )?;

    let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset));

    Ok(gen_tx)
}

/// save the genesis blob
pub fn save_genesis(gen_tx: &Transaction, output_path: &PathBuf) -> Result<(), Error> {
    // let file_path = output_path.join("genesis").with_extension("blob");
    let mut file = File::create(output_path)?;
    let bytes = bcs::to_bytes(&gen_tx)?;
    file.write_all(&bytes)?;
    Ok(())
}

#[test]

fn test_basic_genesis() {
    use diem_vm_genesis::TestValidator;
    use libra_framework::head_release_bundle;
    let test_validators = TestValidator::new_test_set(Some(4), Some(100_000_000));
    let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();
    let _tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut [],
        &validators,
        &head_release_bundle(),
        ChainId::test(),
        None,
        &libra_genesis_default(NamedChain::TESTING),
    )
    .unwrap();

    // std::fs::write(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_genesis.blob"), bcs::to_bytes(&tx).unwrap());
}

#[test]
fn test_recovery_genesis() {
    use crate::parse_json;
    use diem_types::{
        on_chain_config::OnChainConfig, on_chain_config::ValidatorSet,
        state_store::state_key::StateKey, write_set::TransactionWrite,
    };
    use diem_vm_genesis::TestValidator;
    use libra_framework::head_release_bundle;

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");

    let mut recovery = parse_json::recovery_file_parse(p).unwrap();

    let test_validators = TestValidator::new_test_set(Some(4), Some(100_000));
    let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();

    let supply = SupplySettings {
        target_supply: 10_000.0, // because we overflow the u64
        ..Default::default()
    };

    let tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut recovery,
        &validators,
        &head_release_bundle(),
        ChainId::test(),
        Some(supply),
        &libra_genesis_default(NamedChain::TESTING),
    )
    .unwrap();

    match tx {
        Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset)) => {
            let state_key =
                StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
            let bytes = recovery_changeset
                .write_set()
                .get(&state_key)
                .unwrap()
                .extract_raw_bytes()
                .unwrap();
            let validator_set: ValidatorSet =
                bcs::from_bytes(&bytes).expect("no validator set found in bytes");
            assert!(
                validator_set.active_validators().len() == 4,
                "validator set is empty"
            );
        }
        _ => panic!("not a genesis transaction"),
    }
}
