use anyhow::{self, Context};
use diem_genesis::config::OperatorConfiguration;
use diem_types::account_address::AccountAddress;
use libra_types::global_config_dir;
use libra_wallet::validator_files::OPERATOR_FILE;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Public data structure for validators to register their nodes on chain
/// creating this depends on access to private keys.
///
// TODO: this matches the  libra framework sdk ValidatorUniverseRegisterValidator, there may be other duplications elsewhere.

#[derive(Debug, Serialize, Deserialize)]
pub struct ValCredentials {
    pub account: AccountAddress,
    /// key for signing consensus transactions
    pub consensus_pubkey: Vec<u8>,
    /// proof that the node is in possession of the keys
    pub proof_of_possession: Vec<u8>,
    /// network addresses for consensus
    pub network_addresses: Vec<u8>,
    /// network addresses for public validator fullnode
    pub fullnode_addresses: Vec<u8>,
}

/// given the operators private keys file at operator.yaml (usually)
/// create the data structure needed for a registration transaction
pub fn registration_from_private_file(
    operator_keyfile: Option<PathBuf>,
) -> anyhow::Result<ValCredentials> {
    let file = operator_keyfile.to_owned().unwrap_or_else(|| {
        let a = global_config_dir();
        a.join(OPERATOR_FILE)
    });

    let yaml_str = fs::read_to_string(file)?;
    let oc: OperatorConfiguration = serde_yaml::from_str(&yaml_str)?;

    let val_net_protocol = oc
        .validator_host
        .as_network_address(oc.validator_network_public_key)?;

    let fullnode_host = oc
        .full_node_host
        .context("cannot find fullnode host in operator config file")?;
    let vfn_fullnode_protocol = fullnode_host.as_network_address(
        oc.full_node_network_public_key
            .context("cannot find fullnode network public key in operator config file")?,
    )?;

    Ok(ValCredentials {
        account: oc.operator_account_address.into(),
        consensus_pubkey: oc.consensus_public_key.to_bytes().to_vec(),
        proof_of_possession: oc.consensus_proof_of_possession.to_bytes().to_vec(),
        network_addresses: bcs::to_bytes(&vec![val_net_protocol])?,
        fullnode_addresses: bcs::to_bytes(&vec![vfn_fullnode_protocol])?,
    })
}

pub fn registration_to_file(reg: ValCredentials, out: &Path) -> anyhow::Result<()> {
    let yaml_str = serde_yaml::to_string(&reg)?;
    fs::write(out, yaml_str.into_bytes())?;
    Ok(())
}

/// given a list of public validator registrations,
/// create the list of validators that need to be created on chain
/// NOTE: this is for testnet purposes.
pub fn parse_pub_files_to_vec(files: Vec<PathBuf>) -> Vec<ValCredentials> {
    files
        .iter()
        .map(|p| {
            println!("reading file: {}", p.display());
            let s = fs::read_to_string(p).expect("could not read registration file");
            let vr: ValCredentials = serde_yaml::from_str(&s).expect("could not parse file");
            vr
        })
        .collect()
}

#[test]

fn test_registration_pub_file_output() {
    use diem_temppath::TempPath;

    let vr = ValCredentials {
        account: AccountAddress::ZERO,
        consensus_pubkey: vec![],
        proof_of_possession: vec![],
        network_addresses: vec![],
        fullnode_addresses: vec![],
    };
    let file = TempPath::new();
    file.create_as_file().unwrap();
    registration_to_file(vr, file.path()).unwrap();
}
