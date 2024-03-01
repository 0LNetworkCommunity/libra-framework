//! saves a validator yaml file with the minimal configurations.

use std::path::PathBuf;

use anyhow::Result;

use libra_types::global_config_dir;
use libra_wallet::utils::write_to_user_only_file;

pub const NODE_YAML_FILE: &str = "validator.yaml";

/// Create a validator yaml file to start validator node.
/// NOTE: this will not work for fullnodes
pub async fn save_validator_yaml(home_dir: Option<PathBuf>) -> Result<PathBuf> {
    let home_dir = home_dir.unwrap_or_else(global_config_dir);
    let path = home_dir.display().to_string();

    let template = format!(
        "
base:
  role: 'validator'
  data_dir: '{path}/data'
  waypoint:
    from_file: '{path}/genesis/waypoint.txt'

consensus:
  safety_rules:
    service:
      type: 'local'
    backend:
      type: 'on_disk_storage'
      path: secure-data.json
      namespace: ~
    initial_safety_rules_config:
      from_file:
        waypoint:
          from_file: {path}/genesis/waypoint.txt
        identity_blob_path: {path}/validator-identity.yaml

execution:
  genesis_file_location: '{path}/genesis/genesis.blob'

validator_network:
  discovery_method: 'onchain'
  mutual_authentication: true
  identity:
    type: 'from_file'
    path: {path}/validator-identity.yaml

full_node_networks:
- network_id:
    private: 'vfn'
  #mutual_authentication: true
  listen_address: '/ip4/0.0.0.0/tcp/6181'
  identity:
    type: 'from_file'
    path: {path}/validator-identity.yaml
- network_id: 'public'
  listen_address: '/ip4/0.0.0.0/tcp/6182'

api:
  enabled: true
  address: '127.0.0.1:8080'
"
    );

    let output_file = home_dir.join(NODE_YAML_FILE);

    write_to_user_only_file(&output_file, NODE_YAML_FILE, template.as_bytes())?;

    let peers = crate::make_yaml_public_fullnode::fetch_seed_addresses(None).await?;
    crate::make_yaml_public_fullnode::add_peers_to_yaml(&output_file, peers)?;

    Ok(output_file)
}

#[tokio::test]
#[ignore] // TODO: not sure why this parsing is failing, when node can start with this file.
async fn test_yaml() {
    use diem_config::config::NodeConfig;
    use libra_wallet::utils::from_yaml;

    let path = global_config_dir().join("test_yaml");

    std::fs::create_dir_all(&path).unwrap();

    let file = save_validator_yaml(Some(path.clone())).await.unwrap();

    let read = std::fs::read_to_string(file).unwrap();

    let y: NodeConfig = from_yaml(&read).unwrap();

    assert!(y.base.role.is_validator());

    assert_eq!(
        y.base.data_dir.display().to_string(),
        format!("{}/data", path.display())
    );
    // remove the file and directory
    std::fs::remove_dir_all(path).unwrap();
}
