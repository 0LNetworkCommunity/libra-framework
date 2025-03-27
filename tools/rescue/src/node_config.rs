use diem_config::config::{InitialSafetyRulesConfig, NodeConfig, WaypointConfig};
use diem_types::waypoint::Waypoint;
use smoke_test::test_utils::swarm_utils::insert_waypoint;
use std::path::Path;

/// update the node's files with waypoint information
pub fn post_rescue_node_file_updates(
    config_path: &Path,  // validator.yaml
    waypoint: Waypoint,  // waypoint
    restore_blob: &Path, // genesis transaction
) -> anyhow::Result<()> {
    let mut node_config = NodeConfig::load_from_path(config_path)?;

    ////////// SETTING WAYPOINT IN SAFETY RULES //////////s
    // TODO: unclear why testnet/swarm tests don't need insert_waypoint()
    // while rescue tests do
    insert_waypoint(&mut node_config, waypoint);

    // TODO: this part is tricky
    // ideally we would not need to access any validator
    // identity files to set a new safety rules config
    let configs_dir = config_path.parent().unwrap();
    let validator_identity_file = configs_dir.join("validator-identity.yaml");
    dbg!(&validator_identity_file);
    assert!(
        validator_identity_file.exists(),
        "validator-identity.yaml not found"
    );

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
    node_config.consensus.sync_only = false;

    node_config.save_to_path(config_path)?;

    println!("success: updated safety rules");
    Ok(())
}
