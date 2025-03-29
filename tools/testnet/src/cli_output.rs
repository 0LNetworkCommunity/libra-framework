/// We'd like to standardize output from the testnet
/// creation, so that we can automate testing.
/// For example: we need the path of the AppCfg, so we can easily
/// send transactions using `txs`. We'll also need the private keys.
use diem_genesis::config::HostAndPort;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct TestnetCliOut {
    /// The path to the testnet config file
    pub data_dir: PathBuf,
    /// The path to the testnet genesis blob
    pub api_endpoint: HostAndPort,
    /// The path to the testnet genesis blob
    pub app_cfg_paths: Vec<PathBuf>,
    /// Keys for the validators
    pub private_tx_keys: Vec<String>,
}
