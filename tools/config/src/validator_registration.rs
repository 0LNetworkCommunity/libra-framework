use std::path::PathBuf;
use anyhow::{self, Context};
use diem_genesis::config::OperatorConfiguration;
use libra_types::global_config_dir;
use libra_wallet::validator_files::OPERATOR_FILE;
use std::fs;

/// Public data structure for validators to register their nodes on chain
/// creating this depends on access to private keys.
///
// TODO: this matches the  libra framework sdk ValidatorUniverseRegisterValidator, there may be other duplications elsewhere.
pub struct ValRegistrationPublic {
    /// key for signing consensus transactions
    pub consensus_pubkey: Vec<u8>,
    /// proof that the node is in possession of the keys
    pub proof_of_possession: Vec<u8>,
    /// network addresses for consensus
    pub network_addresses: Vec<u8>,
    /// network addresses for public validator fullnode
    pub fullnode_addresses: Vec<u8>,
}

pub fn registration_from_file(operator_keyfile: Option<PathBuf>) -> anyhow::Result<ValRegistrationPublic> {
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

    Ok(ValRegistrationPublic {
        consensus_pubkey: oc.consensus_public_key.to_bytes().to_vec(),
        proof_of_possession: oc.consensus_proof_of_possession.to_bytes().to_vec(),
        network_addresses: bcs::to_bytes(&vec![val_net_protocol])?,
        fullnode_addresses: bcs::to_bytes(&vec![vfn_fullnode_protocol])?,
    })
}
