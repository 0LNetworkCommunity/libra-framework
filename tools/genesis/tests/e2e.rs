use std::path::PathBuf;
use libra_genesis_tools::{parse_json, genesis::{make_recovery_genesis_from_vec_legacy_recovery, save_genesis}};
use zapatos_vm_genesis::{TestValidator, Validator};
use zapatos_types::{
  state_store::state_key::StateKey,
  on_chain_config::ValidatorSet,
  on_chain_config::OnChainConfig,
  write_set::TransactionWrite,
  chain_id::ChainId,
  transaction::{Transaction, WriteSetPayload},
};

use libra_framework::head_release_bundle;
use libra_types::test_drop_helper::DropTemp;
use std::fs;

#[test]
fn end_to_end() {
  let drop = DropTemp::new_in_crate("temp_genesis_e2e");

  let blob = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/genesis.blob");

  let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");
  
  let recovery = parse_json::parse(p).unwrap();

  let num_vals = 6;
  let test_validators = TestValidator::new_test_set(Some(num_vals), Some(100_000_000_000_000));
  let validators: Vec<Validator> = test_validators.iter().map(|t| t.data.clone()).collect();

      let tx = make_recovery_genesis_from_vec_legacy_recovery(
        Some(&recovery),
        &validators,
        &head_release_bundle(),
        ChainId::test(),
      ).expect("could not write genesis.blob");

  save_genesis(&tx, &blob).unwrap();
  assert!(blob.exists(), "genesis.blob does not exist");

  let gen_bytes = fs::read(blob).unwrap();

  match bcs::from_bytes(&gen_bytes).unwrap() {

      Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset)) => {

        let state_key = StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
        let bytes = recovery_changeset.write_set()
            .get(&state_key)
            .unwrap()
            .extract_raw_bytes()
            .unwrap();
        let validator_set: ValidatorSet = bcs::from_bytes(&bytes).unwrap();
        dbg!(&validator_set.active_validators().len());
        assert!(validator_set.active_validators().len() == num_vals, "validator set count does not match");
      }
      _ => panic!("not a genesis transaction"),
  }

  drop.maybe_cleanup();
}