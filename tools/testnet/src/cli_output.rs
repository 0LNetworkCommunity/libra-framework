/// We'd like to standardize output from the testnet
/// creation, so that we can automate testing.
/// For example: we need the path of the AppCfg, so we can easily
/// send transactions using `txs`. We'll also need the private keys.
use diem_genesis::config::HostAndPort;
use libra_types::exports::AccountAddress;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize)]
pub struct TestInfo {
    // Address of the validator
    pub validator_address: AccountAddress,
    /// index of validator in validator set
    pub val_set_index: usize,
    /// The path to the testnet config file
    pub data_dir: PathBuf,
    /// The path to the testnet genesis blob
    pub api_endpoint: HostAndPort,
    /// The path to the libra-cli-config.yaml which includes pk and api_endpoint
    pub app_cfg_path: PathBuf,
}
