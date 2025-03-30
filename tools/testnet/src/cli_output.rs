/// We'd like to standardize output from the testnet
/// creation, so that we can automate testing.
/// For example: we need the path of the AppCfg, so we can easily
/// send transactions using `txs`. We'll also need the private keys.
use diem_forge::{LocalSwarm, Node};
use diem_genesis::config::HostAndPort;
use libra_types::{core_types::app_cfg::CONFIG_FILE_NAME, exports::AccountAddress};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
use url::Url;
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

impl TestInfo {
    pub fn from_smoke(smoke_instance: &LocalSwarm) -> anyhow::Result<Vec<TestInfo>> {
        // for each LocalNode in the LocalSwarm
        // make the TestInfo struct
        // and collect into a vector
        let mut test_infos = Vec::new();

        for (index, node) in smoke_instance.validators().enumerate() {
            let validator_address = node.peer_id();
            let data_dir = node.config_path().parent().unwrap().to_path_buf();
            let api_endpoint = url_to_host_and_port(&node.rest_api_endpoint())?;
            let app_cfg_path = data_dir.join(CONFIG_FILE_NAME);

            test_infos.push(TestInfo {
                validator_address,
                val_set_index: index,
                data_dir,
                api_endpoint,
                app_cfg_path,
            });
        }

        Ok(test_infos)
    }
}

/// Converts a Url type to a HostAndPort struct.
fn url_to_host_and_port(url: &Url) -> anyhow::Result<HostAndPort> {
    // Parse the URL to get the host and port
    let host = url.host_str().expect("invalid host");
    let port = url.port().expect("invalid port");
    HostAndPort::from_str(&format!("{}:{}", host, port))
}
