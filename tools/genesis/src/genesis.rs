//! create a genesis from a LegacyRecovery struct

use crate::vm::zapatos_mainnet_genesis;
use anyhow::Error;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
use zapatos_framework::ReleaseBundle;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use zapatos_types::{transaction::{Transaction, WriteSetPayload}, chain_id::ChainId};
use zapatos_vm_genesis::Validator;
/// Make a recovery genesis blob
pub fn make_recovery_genesis_from_vec_legacy_recovery(
    recovery: Option<&[LegacyRecovery]>,
    genesis_vals: &[Validator],
    // genesis_blob_path: &PathBuf,
    framework_release: &ReleaseBundle,
    chain_id: ChainId,
    // append_user_accounts: bool,
) -> Result<Transaction, Error> {
    // dbg!(&"make_recovery_genesis_from_vec_legacy_recovery");

    // Note: For `recovery` on a real upgrade or fork, we want to include all user accounts. If a None is passed, then we'll just run the default genesis
    // which only uses the validator accounts.
    let recovery_changeset = zapatos_mainnet_genesis(genesis_vals, recovery, framework_release, chain_id)?;

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
  use libra_framework::head_release_bundle;
  use zapatos_vm_genesis::TestValidator;
  let test_validators = TestValidator::new_test_set(Some(4), Some(100_000_000));
  let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();
  make_recovery_genesis_from_vec_legacy_recovery(
      None,
      &validators,
      &head_release_bundle(),
      ChainId::test(),
    ).unwrap();
}

#[test]
fn test_recovery_genesis() {
    use crate::parse_json;
    use libra_framework::head_release_bundle;
    use zapatos_vm_genesis::TestValidator;
    use zapatos_types::{state_store::state_key::StateKey, on_chain_config::ValidatorSet, on_chain_config::OnChainConfig, 
    write_set::TransactionWrite};

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");

    let recovery = parse_json::parse(p).unwrap();

    let test_validators = TestValidator::new_test_set(Some(4), Some(100_000_000));
    let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();
    let tx = make_recovery_genesis_from_vec_legacy_recovery(
        Some(&recovery),
        &validators,
        &head_release_bundle(),
        ChainId::test(),
      ).unwrap();

    match tx  {
      Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset)) => {

        let state_key = StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
        let bytes = recovery_changeset.write_set()
            .get(&state_key)
            .unwrap()
            .extract_raw_bytes()
            .unwrap();
        let validator_set: ValidatorSet = bcs::from_bytes(&bytes).unwrap();
        assert!(validator_set.active_validators().len() == 4, "validator set is empty");
      }
      _ => panic!("not a genesis transaction")
    }
}