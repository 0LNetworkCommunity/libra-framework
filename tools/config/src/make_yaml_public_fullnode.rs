use diem_config::config::NodeConfig;
use diem_crypto::x25519;
use diem_types::{
    network_address::{DnsName, NetworkAddress},
    waypoint::Waypoint,
    PeerId,
};
use libra_types::global_config_dir;
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::get_genesis_artifacts::get_genesis_waypoint;

pub const FN_FILENAME: &str = "fullnode.yaml";
pub const VFN_FILENAME: &str = "vfn.yaml";
pub const GENESIS_FILES_VERSION: &str = "7.0.0";
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GithubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String, // Use `content_type` to avoid conflict with Rust's `type` keyword
    download_url: Option<String>,
}

/// fetch seed peers and make a yaml file from template
pub async fn init_fullnode_yaml(
    home_dir: Option<PathBuf>,
    overwrite_peers: bool,
) -> anyhow::Result<PathBuf> {
    let waypoint = get_genesis_waypoint(home_dir.clone()).await?;

    let yaml = make_fullnode_yaml(home_dir.clone(), waypoint)?;

    let home = home_dir.unwrap_or_else(global_config_dir);
    let p = home.join(FN_FILENAME);
    std::fs::write(&p, yaml)?;

    if overwrite_peers {
        let peers = fetch_seed_addresses(None).await?;
        add_peers_to_yaml(&p, peers)?;
    }

    Ok(p)
}

/// Add peers to the fullnode YAML configuration file.
pub fn add_peers_to_yaml(
    path: &Path,
    peers: HashMap<PeerId, Vec<NetworkAddress>>,
) -> anyhow::Result<()> {
    // Read the existing YAML content
    let string = std::fs::read_to_string(path)?;
    let mut parsed: NodeConfig = serde_yaml::from_str(&string)?;

    // Update seed addresses for public networks
    parsed.full_node_networks.iter_mut().for_each(move |e| {
        if e.network_id.is_public_network() {
            e.seed_addrs.clone_from(&peers);
        }
    });

    // Serialize the updated content and write it back to the file
    let ser = serde_yaml::to_string(&parsed)?;
    std::fs::write(path, ser)?;

    Ok(())
}
/// get seed peers from an upstream url
pub async fn fetch_seed_addresses(
    url: Option<&str>,
) -> anyhow::Result<HashMap<PeerId, Vec<NetworkAddress>>> {
    let u = url.unwrap_or(
        "https://raw.githubusercontent.com/0LNetworkCommunity/seed-peers/main/seed_peers.yaml",
    );
    // Fetch and parse the seed addresses from the URL
    let res = reqwest::get(u).await?;
    let out = res.text().await?;
    let seeds: HashMap<PeerId, Vec<NetworkAddress>> = serde_yaml::from_str(&out)?;

    Ok(seeds)
}


/// A fullnode config which does a fast sync to the chain
/// as opposed to the default fullnode config for archive nodes.
pub fn make_fast_sync_yaml(home_dir: Option<PathBuf>, waypoint: Waypoint) -> anyhow::Result<String> {
    let home_dir = home_dir.unwrap_or_else(global_config_dir);
    let path = home_dir.display().to_string();

    // Create the YAML template with necessary configurations
    let template = format!(
        "
base:
  role: 'full_node'
  data_dir: '{path}/data'
  waypoint:
    from_config: '{waypoint}'

execution:
  genesis_file_location: '{path}/genesis/genesis.blob'

state_sync:
     state_sync_driver:
        bootstrapping_mode: DownloadLatestStates
        continuous_syncing_mode: ApplyTransactionOutputs

full_node_networks:
- network_id: 'public'
  listen_address: '/ip4/0.0.0.0/tcp/6182'


api:
  enabled: true
  address: '0.0.0.0:8080'
"
    );
    Ok(template)
}

/// Create a fullnode yaml to bootstrap node
pub fn make_fullnode_yaml(home_dir: Option<PathBuf>, waypoint: Waypoint) -> anyhow::Result<String> {
    let home_dir = home_dir.unwrap_or_else(global_config_dir);
    let path = home_dir.display().to_string();

    // Create the YAML template with necessary configurations
    let template = format!(
        "
base:
  role: 'full_node'
  data_dir: '{path}/data'
  waypoint:
    from_config: '{waypoint}'

execution:
  genesis_file_location: '{path}/genesis/genesis.blob'

state_sync:
     state_sync_driver:
        bootstrapping_mode: ExecuteOrApplyFromGenesis
        continuous_syncing_mode: ApplyTransactionOutputs
storage:
  storage_pruner_config:
    ledger_pruner_config:
      enable: false
    state_merkle_pruner_config:
      enable: false
    epoch_snapshot_pruner_config:
      enable: false

full_node_networks:
- network_id: 'public'
  listen_address: '/ip4/0.0.0.0/tcp/6182'


api:
  enabled: true
  address: '0.0.0.0:8080'
"
    );
    Ok(template)
}

/// Create a VFN file to for validators to seed the public network
pub fn make_private_vfn_yaml(
    home_dir: &Path,
    val_net_pubkey: x25519::PublicKey,
    val_host_addr: DnsName,
) -> anyhow::Result<String> {
    let path = home_dir.display().to_string();
    let val_net_pubkey = val_net_pubkey.to_string();
    let val_host_addr = val_host_addr.to_string();

    // Create the YAML template with necessary configurations for VFN
    let template = format!(
        "
base:
  role: 'full_node'
  data_dir: '{path}/data'
  waypoint:
    from_file: '{path}/genesis/waypoint.txt'

execution:
  genesis_file_location: '{path}/genesis/genesis.blob'

state_sync:
     state_sync_driver:
        bootstrapping_mode: ApplyTransactionOutputsFromGenesis
        continuous_syncing_mode: ApplyTransactionOutputs

storage:
  storage_pruner_config:
    ledger_pruner_config:
      enable: false
    state_merkle_pruner_config:
      enable: false
    epoch_snapshot_pruner_config:
      enable: false

full_node_networks:
- network_id: 'public'
  listen_address: '/ip4/0.0.0.0/tcp/6182'
  identity:
    type: 'from_file'
    path: {path}/validator-full-node-identity.yaml

- network_id:
    private: 'vfn'
  listen_address: '/ip4/0.0.0.0/tcp/6181'
  identity:
    type: 'from_file'
    path: {path}/validator-full-node-identity.yaml
  seeds:
    {val_net_pubkey}:
      addresses:
      - '/ip4/{val_host_addr}/tcp/6181/noise-ik/0x{val_net_pubkey}/handshake/0'
      role: 'Validator'

api:
  enabled: true
  address: '0.0.0.0:8080'

mempool:
  default_failovers: 3
"
    );

    let p = home_dir.join(VFN_FILENAME);
    std::fs::write(p, &template)?;

    Ok(template)
}


#[tokio::test]
async fn get_yaml() {
    use std::str::FromStr;
    let p = diem_temppath::TempPath::new().path().to_owned();

    let seeds = fetch_seed_addresses(None).await.unwrap();

    let y = make_fullnode_yaml(
        Some(p.clone()),
        Waypoint::from_str("0:95023f4d6a7e24cac3e52cad29697184db260214210b57aef3f1031ad4d8c02c")
            .unwrap(),
    )
    .unwrap();

    std::fs::write(&p, y).unwrap();

    add_peers_to_yaml(&p, seeds).unwrap();

    let _text = std::fs::read_to_string(&p).unwrap();
}
