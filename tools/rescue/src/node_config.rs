use diem_config::config::{
    InitialSafetyRulesConfig, NodeConfig, OnDiskStorageConfig, PersistableConfig, SecureBackend,
    WaypointConfig,
};
use diem_forge::node;
use diem_types::{network_address::NetworkAddress, waypoint::Waypoint, PeerId};
use smoke_test::test_utils::swarm_utils::insert_waypoint;
use std::path::Path;

/// update the node's files with waypoint information
pub fn post_rescue_node_file_updates(
    config_path: &Path,  // validator.yaml
    waypoint: Waypoint,  // waypoint
    restore_blob: &Path, // genesis transaction
) -> anyhow::Result<NodeConfig> {
    let mut node_config = NodeConfig::load_config(config_path)?;
    // node_config.base.working_dir = Some(node_config.base.data_dir.clone());
    if let SecureBackend::OnDiskStorage(_) = node_config.consensus.safety_rules.backend {
        node_config
            .consensus
            .safety_rules
            .set_data_dir(node_config.base.data_dir.clone());
    }
    //   node_config.consensus.safety_rules.backend = SecureBackend::OnDiskStorage(OnDiskStorageConfig {
    //     path: todo!(),
    //     namespace: todo!(),
    //     data_dir: todo!(),
    // }
    //     // (&node_config.base.data_dir.join("secure_storage.json")));
    // }
    ////////// SETTING WAYPOINT IN SAFETY RULES //////////
    // try to start a blank safety rules file
    // note that this is using a relateive path
    // so you may notice the safety configs
    // being initialized to a different path
    insert_waypoint(&mut node_config, waypoint);

    // TODO: this part is tricky
    // ideally we would not need to access any validator
    // identity files to set a new safety rules config
    let configs_dir = config_path.parent().unwrap();
    let validator_identity_file = configs_dir.join("validator-identity.yaml");

    assert!(
        validator_identity_file.exists(),
        "validator-identity.yaml not found"
    );

    // IF this does not get set, you will see safety rules no initialized.
    let init_safety = InitialSafetyRulesConfig::from_file(
        validator_identity_file,
        WaypointConfig::FromConfig(waypoint),
    );
    node_config
        .consensus
        .safety_rules
        .initial_safety_rules_config = init_safety;

    //////// SETTING GENESIS TRANSACTION ////////
    // This must reference the lastest writeset, not the
    // original genesis

    // NOTE: Must reset the genesis transaction in the config file
    // Or overwrite with a serialized versions
    node_config.execution.genesis = None; // see above to use bin: Some(genesis_transaction);
                                          // ... and point to file
    node_config.execution.genesis_file_location = restore_blob.to_path_buf();
    // NOTE: Alternatively, you can use the following to serialize the genesis transaction
    // Note: Example of getting genesis transaction serialized to include in config.
    // let genesis_transaction = {
    //     let buf = std::fs::read(rescue_blob.clone()).unwrap();
    //     bcs::from_bytes::<Transaction>(&buf).unwrap()
    // };
    /////////

    // just in case our tests set to sync_only=true
    // node_config.consensus.sync_only = false;

    node_config.save_to_path(config_path)?;

    println!("success: updated safety rules");
    Ok(node_config)
}

pub fn set_validator_peers(
    config_path: &Path,
    peers: Vec<(PeerId, Vec<NetworkAddress>)>,
) -> anyhow::Result<NodeConfig> {
    let mut node_config = NodeConfig::load_config(config_path)?;

    // Update the validator network seed peers
    if let Some(network) = &mut node_config.validator_network {
        // Clear existing seeds and add new ones
        // network.seeds.clear();
        network.seed_addrs.clear();

        // Add the new peers
        for (peer_id, addresses) in peers {
            // network.seeds.insert(peer_id, addresses);
            network.seed_addrs.insert(peer_id, addresses);
        }
        network.verify_seeds()?;

        println!("success: updated validator network seed peers");
    } else {
        return Err(anyhow::anyhow!("Validator network configuration not found"));
    }
    node_config.save_to_path(config_path)?;

    Ok(node_config)
}
