use std::{collections::HashMap, path::{PathBuf, Path}};
use diem_config::config::NodeConfig;
use diem_types::{PeerId, network_address::NetworkAddress, waypoint::Waypoint};


pub fn add_peers_to_yaml(path: &Path, peers: HashMap<PeerId, Vec<NetworkAddress>> ) -> anyhow::Result<()> {
  let string = std::fs::read_to_string(path)?;
  let mut parsed: NodeConfig = serde_yaml::from_str(&string)?;

  parsed.full_node_networks.iter_mut().for_each(move |e| {
    if e.network_id.is_public_network() {
      e.seed_addrs = peers.clone();
    }
  });

  let ser = serde_yaml::to_string(&parsed)?;
  std::fs::write(path, ser)?;

  Ok(())
}

/// download genesis blob
///
pub fn download_genesis(_home_dir: Option<PathBuf>) {

}

/// get seed peers from an upstream url
pub async fn fetch_seed_addresses(url: Option<&str>) -> anyhow::Result<HashMap<PeerId, Vec<NetworkAddress>>> {
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
  genesis_file_location: '{path}/genesis.blob'

state_sync:
     state_sync_driver:
        bootstrapping_mode: DownloadLatestStates
        continuous_syncing_mode: ExecuteTransactionsOrApplyOutputs

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


#[tokio::test]
async fn get_peers() {
  let seed = fetch_seed_addresses(None).await.unwrap();

  dbg!(&seed);
}

#[tokio::test]
async fn get_yaml() {
  use std::str::FromStr;
  let p = diem_temppath::TempPath::new().path().to_owned();

  let seeds = fetch_seed_addresses(None).await.unwrap();

  let y = make_fullnode_yaml(
    Some(p.clone()),
    Waypoint::from_str("0:95023f4d6a7e24cac3e52cad29697184db260214210b57aef3f1031ad4d8c02c").unwrap(),
  ).unwrap();

  std::fs::write(&p, y).unwrap();

  // let mut parsed: NodeConfig = serde_yaml::from_str(&y).unwrap();
  // parsed.full_node_networks.iter_mut().for_each(move |e| {
  //   if e.network_id.is_public_network() {
  //     e.seed_addrs = seeds.clone();
  //   }
  // });

  add_peers_to_yaml(&p, seeds).unwrap();

  let text = std::fs::read_to_string(&p).unwrap();

  dbg!(&text);
}

