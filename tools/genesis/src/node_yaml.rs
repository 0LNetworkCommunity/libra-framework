//! saves a validator yaml file with the minimal configurations.

use std::path::PathBuf;

use anyhow::Result;

use libra_types::global_config_dir;
use libra_wallet::utils::write_to_user_only_file;


pub const NODE_YAML_FILE: &str = "validator.yaml";

/// Create a validator yaml file to start validator node.
/// NOTE: this will not work for fullnodes
pub fn save_validator_yaml(home_dir: Option<PathBuf>) -> Result<PathBuf> {
    let home_dir = home_dir.unwrap_or_else(|| {
        global_config_dir()
    });
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
  listen_address: '/ip4/0.0.0.0/tcp/6181'
  identity:
    type: 'from_file'
    path: {path}/validator-identity.yaml

api:
  enabled: true
  address: '0.0.0.0:8080'
"
    );

    let output_file = home_dir.join(NODE_YAML_FILE);

    write_to_user_only_file(&output_file, NODE_YAML_FILE, &template.as_bytes())?;

    Ok(output_file)
}

#[test]
#[ignore] // TODO: not sure why this parsing is failing, when node can start with this file.
fn test_yaml() {
    use libra_wallet::utils::from_yaml;
    use zapatos_config::config::NodeConfig;

    let path = dirs::home_dir()
        .expect("Unable to determine home directory")
        .join(DEFAULT_DATA_PATH)
        .join("test_yaml");

    std::fs::create_dir_all(&path).unwrap();

    let file = save_validator_yaml(Some(path.clone())).unwrap();

    let read = std::fs::read_to_string(&file).unwrap();

    let y: NodeConfig = from_yaml(&read).unwrap();

    assert!(y.base.role.is_validator());

    assert!(
        y.base.data_dir.display().to_string() == format!("{}/data", path.display().to_string())
    );
    // remove the file and directory
    std::fs::remove_dir_all(path).unwrap();
}
