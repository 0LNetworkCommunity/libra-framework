//! helper to get fixtures data from files in ol/fixtures folder.
use core::fmt;
use std::{fs, path::PathBuf, str::FromStr};

use anyhow::bail;

#[derive(Clone, Debug)]
pub enum TestPersona {
    Alice,
    Bob,
    Carol,
    Dave,
}

impl FromStr for TestPersona {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "alice" => Ok(TestPersona::Alice),
            "bob" => Ok(TestPersona::Bob),
            "carol" => Ok(TestPersona::Carol),
            "dave" => Ok(TestPersona::Dave),
            _ => Err("not found"),
        }
    }
}

impl fmt::Display for TestPersona {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TestPersona::Alice => "alice",
            TestPersona::Bob => "bob",
            TestPersona::Carol => "carol",
            TestPersona::Dave => "dave",
        };
        write!(f, "{}", s)
    }
}

impl TestPersona {
    /// get persona from index. Used for testnet to assign persona to validator seat
    pub fn from(idx: usize) -> anyhow::Result<Self> {
        match idx {
            0 => Ok(TestPersona::Alice),
            1 => Ok(TestPersona::Bob),
            2 => Ok(TestPersona::Carol),
            3 => Ok(TestPersona::Dave),
            _ => bail!("no default persona at this index"),
        }
    }

    /// get persona from index. Used for testnet to assign persona to validator seat
    pub fn idx(&self) -> usize {
        match self {
            TestPersona::Alice => 0,
            TestPersona::Bob => 1,
            TestPersona::Carol => 2,
            TestPersona::Dave => 3,
        }
    }
    /// get mnemonic
    pub fn get_persona_mnem(&self) -> String {
        let path = env!("CARGO_MANIFEST_DIR");
        // dbg!(&self.to_string());
        let buf = PathBuf::from_str(path)
            .unwrap()
            .parent()
            .unwrap()
            .join("util/fixtures/mnemonic")
            .join(format!("{}.mnem", &self.to_string()));
        fs::read_to_string(buf).expect("could not find mnemonic file")
    }
}

// /// get account json
// pub fn get_persona_account_json(persona: &str) -> (String, PathBuf) {
//     let path = env!("CARGO_MANIFEST_DIR");
//     let buf = Path::new(path)
//         .parent()
//         .unwrap()
//         .join("fixtures/account")
//         .join(format!("{}.account.json", persona));
//     (
//         fs::read_to_string(&buf).expect("could not account file"),
//         buf,
//     )
// }

// /// get autopay
// pub fn get_persona_autopay_json(persona: &str) -> (String, PathBuf) {
//     let path = env!("CARGO_MANIFEST_DIR");
//     let buf = Path::new(path)
//         .parent()
//         .unwrap()
//         .join("fixtures/autopay")
//         .join(format!("{}.autopay_batch.json", persona));
//     (
//         fs::read_to_string(&buf).expect("could not find autopay file"),
//         buf,
//     )
// }

// /// get demo autopay
// pub fn get_demo_autopay_json() -> (String, PathBuf) {
//     let path = env!("CARGO_MANIFEST_DIR");
//     let buf = Path::new(path)
//         .parent()
//         .unwrap()
//         .join("fixtures/autopay")
//         .join("all.autopay_batch.json");
//     (
//         fs::read_to_string(&buf).expect("could not find autopay file"),
//         buf,
//     )
// }

// /// get genesis blob for tests
// pub fn get_test_genesis_blob() -> PathBuf {
//     let path = env!("CARGO_MANIFEST_DIR");
//     Path::new(path)
//         .parent()
//         .unwrap()
//         .join("fixtures/genesis")
//         .join("swarm_genesis.blob")
// }

// /// get configs from toml
// pub fn get_persona_toml_configs(persona: &str) -> AppCfg {
//     let path = env!("CARGO_MANIFEST_DIR");
//     let buf = Path::new(path)
//         .parent()
//         .unwrap()
//         .join("fixtures/configs")
//         .join(format!("{}.toml", persona));
//     parse_toml(buf).expect("could not get fixtures for persona")
// }

// /// get block 0
// pub fn get_persona_block_zero_path(persona: &str, env: &str) -> PathBuf {
//     let path = env!("CARGO_MANIFEST_DIR");
//     Path::new(path).parent().unwrap().join(format!(
//         "fixtures/vdf_proofs/{}/{}/proof_0.json",
//         env, persona
//     ))
// }

// /// get block 0
// pub fn get_persona_block_zero(persona: &str, env: NamedChain) -> VDFProof {
//     let path = env!("CARGO_MANIFEST_DIR");
//     let buf = Path::new(path).parent().unwrap().join(format!(
//         "fixtures/vdf_proofs/{}/{}/proof_0.json",
//         env.to_string(), persona
//     ));

//     let s = fs::read_to_string(&buf).expect(&format!("could not find fixture proof_0.json from file: {:?}", &buf));
//     serde_json::from_str(&s).expect(&format!("could not parse block from file: {:?}", &buf))
// }

// /// get block 0
// pub fn get_persona_block_one(persona: &str, env: &str) -> VDFProof {
//     let path = env!("CARGO_MANIFEST_DIR");
//     let buf = Path::new(path).parent().unwrap().join(format!(
//         "fixtures/vdf_proofs/{}/{}/proof_1.json",
//         env, persona
//     ));

//     let s = fs::read_to_string(&buf).expect("could not find block file");
//     serde_json::from_str(&s).expect(&format!("could not parse block from file: {:?}", &buf))
// }

#[test]
fn test_block() {
    let mnem = TestPersona::Alice.get_persona_mnem();
    assert!(mnem.contains("talent"));
}
