mod support;

use diem_api_types::ViewRequest;
use diem_config::config::{InitialSafetyRulesConfig, NodeConfig};
use diem_forge::{LocalNode, NodeExt, SwarmExt, Validator};
use diem_logger::prelude::*;
use diem_temppath::TempPath;
use diem_types::transaction::Transaction;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use smoke_test::test_utils::{
    swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS, MAX_CONNECTIVITY_WAIT_SECS,
    MAX_HEALTHY_WAIT_SECS,
};
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};

fn update_node_config_restart(validator: &mut LocalNode, mut config: NodeConfig) {
    validator.stop();
    let node_path = validator.config_path();
    config.save_to_path(node_path).unwrap();
    validator.start().unwrap();
}

async fn wait_for_node(validator: &mut dyn Validator, expected_to_connect: usize) {
    let healthy_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_HEALTHY_WAIT_SECS))
        .unwrap();
    validator
        .wait_until_healthy(healthy_deadline)
        .await
        .unwrap_or_else(|err| {
            let lsof_output = Command::new("lsof").arg("-i").output().unwrap();
            panic!(
                "wait_until_healthy failed. lsof -i: {:?}: {}",
                lsof_output, err
            );
        });
    info!("Validator restart health check passed");

    let connectivity_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_CONNECTIVITY_WAIT_SECS))
        .unwrap();
    validator
        .wait_for_connectivity(expected_to_connect, connectivity_deadline)
        .await
        .unwrap();
    info!("Validator restart connectivity check passed");
}

// #[ignore]
#[tokio::test]
/// This meta test checks that our tools can control a network
/// so the nodes stop producing blocks, shut down, and start again.
async fn test_swarm_can_halt_and_restart() {
    let num_nodes: usize = 5;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    let env: &mut diem_forge::LocalSwarm = &mut s.swarm;

    env.wait_for_all_nodes_to_catchup_to_version(10, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("2. Set sync_only = true for all nodes and restart");
    for node in env.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = true;
        update_node_config_restart(node, node_config);
        wait_for_node(node, num_nodes - 1).await;
    }

    println!("4. verify all nodes are at the same round and no progress being made");
    env.wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("5. kill nodes");
    for node in env.validators_mut().take(3) {
        node.stop();
    }

    for node in env.validators_mut().take(3) {
        assert!(node.liveness_check(1).await.is_err());
    }
}
