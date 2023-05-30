use libra_types::{state_store::state_key::StateKey, on_chain_config::ValidatorSet, write_set::WriteSet, on_chain_config::OnChainConfig, write_set::TransactionWrite};
use genesis_tools::vm::libra_mainnet_genesis;
use libra_vm_genesis::TestValidator;
use std::path::PathBuf;
use genesis_tools::parse_json;

#[test]
fn recovery_genesis() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");

    let recovery = parse_json::parse(p).unwrap();

    // avoid error stake too low: 0x10002
    let test_validators = TestValidator::new_test_set(Some(6), Some(100_000_000_000_000));

    let vec_vals = vec![test_validators[0].data.clone()];
    // dbg!(&vec_vals);
    let recovery_changeset = libra_mainnet_genesis(&vec_vals, Some(&recovery)).unwrap();

    let WriteSet::V0(writeset) = recovery_changeset.write_set();

    let state_key = StateKey::access_path(ValidatorSet::access_path().expect("access path in test"));
    let bytes = writeset
        .get(&state_key)
        .unwrap()
        .extract_raw_bytes()
        .unwrap();
    let validator_set: ValidatorSet = bcs::from_bytes(&bytes).unwrap();
    assert!(validator_set.active_validators().len() == 1, "validator set is empty");

}


