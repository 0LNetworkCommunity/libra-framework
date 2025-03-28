use anyhow::Result;
use diem_config::config::{
    InitialSafetyRulesConfig, NodeConfig, OnDiskStorageConfig, PersistableConfig, SecureBackend,
    WaypointConfig,
};
use diem_forge::node;
use diem_types::{network_address::NetworkAddress, waypoint::Waypoint, PeerId};
use smoke_test::test_utils::swarm_utils::insert_waypoint;
use std::path::Path;

// This is a hail mary to force safety rules into an OK state
// InitialSafetyRulesConfig with the new waypoint as set in the validator.yaml
// file after bootstrapping
pub fn runtime_hot_fix(
    config_path: &Path, // validator.yaml
    waypoint: Waypoint, // waypoint
) -> Result<()>{
    let mut node_config = NodeConfig::load_config(config_path)?;

    insert_waypoint(&mut node_config, waypoint);

    node_config.save_to_path(config_path)?;
    Ok(())
}

// NOTE: since safety rules can exist on disk, vault, or in memory
// we can't really apply it until the node is started.
// in testnet cases we use disk, and it would be possible to apply
// prior to starting a node, but in other cases it would not work.
/// update the node's files with waypoint information
pub fn post_rescue_node_file_updates(
    config_path: &Path,  // validator.yaml
    waypoint: Waypoint,  // waypoint
    restore_blob: &Path, // genesis transaction
) -> anyhow::Result<NodeConfig> {
    let mut node_config = NodeConfig::load_config(config_path)?;

        ////////// SETTING WAYPOINT IN SAFETY RULES //////////
    // Important: this must be set in the safety rules config.
    // The "waypoint" and "genesis-waypoint" keys
    // must be set to the same value, which is the
    // post-rescue waypoint,
    // NOTHING WILL WORK IF THIS IS NOT SET
    //
    //                 ^    ^
    //                / \  //\
    //  |\___/|      /   \//  .\
    //  /O  O  \__  /    //  | \ \
    // /     /  \/_/    //   |  \  \
    // @___@'    \/_   //    |   \   \
    //    |       \/_ //     |    \    \
    //    |        \///      |     \     \
    //   _|_ /   )  //       |      \     _\
    //  '/,_ _ _/  ( ; -.    |    _ _\.-~        .-~~~^-.
    //  ,-{        _      `-.|.-~-.           .~         `.
    //   '/\      /                 ~-. _ .-~      .-~^-.  \
    //      `.   {            }                   /      \  \
    //    .----~-.\        \-'                 .~         \  `. \^-.
    //   ///.----..&gt;    c   \             _ -~             `.  ^-`   ^-_
    //     ///-._ _ _ _ _ _ _}^ - - - - ~                     ~--,   .-~

    node_config.base.waypoint = WaypointConfig::FromConfig(waypoint.clone());
    /////////////////

    // make sure the config file is not using  a relative path like ./secure_storage.json , but the
    // canonical path to the validators data_dir
    // This is a known problem with swarm tests.
    if let SecureBackend::OnDiskStorage(_) = node_config.consensus.safety_rules.backend {
        node_config
            .consensus
            .safety_rules
            .set_data_dir(node_config.base.data_dir.clone());
    }

    // NOTE: it's not clear why the safety rules don't
    // fully initialize on node start up with with the "initial configs".
    // on a restore, the node is creating initial configs with the old waypoint
    // TODO: find out why, might be a bug.
    let configs_dir = config_path.parent().unwrap();
    let validator_identity_file = configs_dir.join("validator-identity.yaml");

    assert!(
        validator_identity_file.exists(),
        "validator-identity.yaml not found"
    );

    println!("\n\n\n waypoint: {:?} \n\n\n", waypoint);
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
    // This must reference the latest rescue writeset, not the
    // original genesis

    // You must first reset the genesis transaction in the config file
    // since it may have a serialized version (more below)
    node_config.execution.genesis = None; // clear whatever value is there.
    // ... and point to the rescue file
    node_config.execution.genesis_file_location = restore_blob.to_path_buf();
    // NOTE: Alternatively, you can use the following to serialize the genesis transaction
    // Note: Example of getting genesis transaction serialized to include in config.
    // let genesis_transaction = {
    //     let buf = std::fs::read(rescue_blob.clone()).unwrap();
    //     bcs::from_bytes::<Transaction>(&buf).unwrap()
    // };
    /////////

    node_config.save_to_path(config_path)?;

    println!("success: updated safety rules");
    Ok(node_config)
}

// TODO: maybe we'll want to facilitate peer discovery.
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
