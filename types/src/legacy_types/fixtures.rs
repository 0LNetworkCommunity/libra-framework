//! helper to get fixtures data from files in ol/fixtures folder.
use std::{fs, path::{Path, PathBuf}};

use diem_types::chain_id::NamedChain;

use crate::{block::VDFProof, config::{AppCfg, parse_toml}};

/// get mnemonic
pub fn get_persona_mnem(persona: &str) -> String {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path)
        .parent()
        .unwrap()
        .join("fixtures/mnemonic")
        .join(format!("{}.mnem", persona));

    fs::read_to_string(&buf).expect("could not find mnemonic file")
}

/// get account json
pub fn get_persona_account_json(persona: &str) -> (String, PathBuf) {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path)
        .parent()
        .unwrap()
        .join("fixtures/account")
        .join(format!("{}.account.json", persona));
    (
        fs::read_to_string(&buf).expect("could not account file"),
        buf,
    )
}

/// get autopay
pub fn get_persona_autopay_json(persona: &str) -> (String, PathBuf) {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path)
        .parent()
        .unwrap()
        .join("fixtures/autopay")
        .join(format!("{}.autopay_batch.json", persona));
    (
        fs::read_to_string(&buf).expect("could not find autopay file"),
        buf,
    )
}

/// get demo autopay
pub fn get_demo_autopay_json() -> (String, PathBuf) {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path)
        .parent()
        .unwrap()
        .join("fixtures/autopay")
        .join("all.autopay_batch.json");
    (
        fs::read_to_string(&buf).expect("could not find autopay file"),
        buf,
    )
}

/// get genesis blob for tests
pub fn get_test_genesis_blob() -> PathBuf {
    let path = env!("CARGO_MANIFEST_DIR");
    Path::new(path)
        .parent()
        .unwrap()
        .join("fixtures/genesis")
        .join("swarm_genesis.blob")
}

/// get configs from toml
pub fn get_persona_toml_configs(persona: &str) -> AppCfg {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path)
        .parent()
        .unwrap()
        .join("fixtures/configs")
        .join(format!("{}.toml", persona));
    parse_toml(buf).expect("could not get fixtures for persona")
}

/// get block 0
pub fn get_persona_block_zero_path(persona: &str, env: &str) -> PathBuf {
    let path = env!("CARGO_MANIFEST_DIR");
    Path::new(path).parent().unwrap().join(format!(
        "fixtures/vdf_proofs/{}/{}/proof_0.json",
        env, persona
    ))
}

/// get block 0
pub fn get_persona_block_zero(persona: &str, env: NamedChain) -> VDFProof {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path).parent().unwrap().join(format!(
        "fixtures/vdf_proofs/{}/{}/proof_0.json",
        env.to_string(), persona
    ));

    let s = fs::read_to_string(&buf).expect(&format!("could not find fixture proof_0.json from file: {:?}", &buf));
    serde_json::from_str(&s).expect(&format!("could not parse block from file: {:?}", &buf))
}

/// get block 0
pub fn get_persona_block_one(persona: &str, env: &str) -> VDFProof {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = Path::new(path).parent().unwrap().join(format!(
        "fixtures/vdf_proofs/{}/{}/proof_1.json",
        env, persona
    ));

    let s = fs::read_to_string(&buf).expect("could not find block file");
    serde_json::from_str(&s).expect(&format!("could not parse block from file: {:?}", &buf))
}

#[test]
fn test_block() {
    let b = get_persona_block_zero("alice", "test");
    assert_eq!(b.difficulty, Some(100));
}
