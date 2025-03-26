use diem_config::config::{InitialSafetyRulesConfig, NodeConfig};
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use smoke_test::test_utils::swarm_utils;
use std::path::Path;

/// update the node's files with waypoint information
pub fn post_rescue_node_file_updates(
    config_path: &Path,               // validator.yaml
    waypoint: Waypoint,               // waypoint
    genesis_transaction: Transaction, // genesis transaction
) -> anyhow::Result<()> {
    let mut node_config = NodeConfig::load_from_path(config_path)?;
    swarm_utils::insert_waypoint(&mut node_config, waypoint);
    node_config
        .consensus
        .safety_rules
        .initial_safety_rules_config = InitialSafetyRulesConfig::None;
    node_config.execution.genesis = Some(genesis_transaction);

    node_config.consensus.sync_only = false;

    node_config.save_to_path(config_path)?;

    Ok(())
}
