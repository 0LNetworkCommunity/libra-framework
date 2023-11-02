use std::{collections::HashMap, path::PathBuf};

use diem_types::{PeerId, network_address::NetworkAddress, waypoint::Waypoint};



/// download genesis blob
///
pub fn download_genesis(home_dir: Option<PathBuf>) {

}

/// get seed peers from an upstream url
pub async fn fetch_seed_addresses(home_dir: Option<PathBuf>, url: Option<&str>) -> anyhow::Result<HashMap<PeerId, Vec<NetworkAddress>>> {
    let u = url.unwrap_or("https://raw.githubusercontent.com/0LNetworkCommunity/seed-peers/main/seed_peers.yaml");
    let res = reqwest::get(u).await?;
    let out = res.text().await?;
    let seeds: HashMap<PeerId, Vec<NetworkAddress>> = serde_yaml::from_str(&out)?;

    Ok(seeds)
}


/// Create a fullnode yaml to bootstrap node
pub fn make_fullnode_yaml(home_dir: Option<PathBuf>, waypoint: Waypoint) -> anyhow::Result<String> {
    let home_dir = home_dir.unwrap_or_else(libra_types::global_config_dir);
    let path = home_dir.display().to_string();

    let template = format!("
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
        continuous_syncing_mode: ExecuteTransactionsOrApplyOutputs

execution:
  genesis_file_location: '{path}/genesis.blob'

full_node_networks:
- network_id: 'public'
  listen_address: '/ip4/0.0.0.0/tcp/6182'
  seed_addrs:
      3c37c7d6a5122a6b9ef07a11cc40e445874eb0841ae028d6326bf67768cce235:
        - '/ip4/204.186.74.42/tcp/6182/noise-ik/0x3c37c7d6a5122a6b9ef07a11cc40e445874eb0841ae028d6326bf67768cce235/handshake/0'

api:
  enabled: true
  address: '0.0.0.0:8080'
"
);
    Ok(template)
}


#[tokio::test]
async fn get_peers() {
  let seed = fetch_seed_addresses(None, None).await.unwrap();

  dbg!(&seed);
}