use std::path::PathBuf;

use genesis_tools::{parse_json, genesis::make_recovery_genesis_from_vec_legacy_recovery};
use libra_vm_genesis::TestValidator;

// NOTE: Useing drop trait for cleaning up env
// https://doc.rust-lang.org/std/ops/trait.Drop.html
struct HasDrop;
impl Drop for HasDrop {
    fn drop(&mut self) {
      println!("we dropped, running cleanup");
      maybe_cleanup();
    }
}

fn maybe_cleanup() {
  let blob = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/genesis.blob");

  if blob.exists() {
    println!("\n RUNNING CLEANUP \n");
    std::fs::remove_file(blob).unwrap();
  }
}

#[test]
fn end_to_end() {
  let _drop = HasDrop;

  let blob = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/genesis.blob");

  let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");
  
  let recovery = parse_json::parse(p).unwrap();

  let test_validators = TestValidator::new_test_set(Some(6), Some(100_000_000_000_000));

  let vec_vals = vec![test_validators[0].data.clone()];

  make_recovery_genesis_from_vec_legacy_recovery(&recovery, &vec_vals, &blob).expect("could not write genesis.blob");

  assert!(blob.exists(), "genesis.blob does not exist");
  maybe_cleanup();
}