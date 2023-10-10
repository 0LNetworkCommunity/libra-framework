
use diem_config::config::NodeConfig;
use diem_forge::{LocalNode, Node, NodeExt, SwarmExt, Validator};
use diem_logger::prelude::*;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use smoke_test::test_utils::{
    MAX_CATCH_UP_WAIT_SECS,
    MAX_CONNECTIVITY_WAIT_SECS,
    MAX_HEALTHY_WAIT_SECS,
};
use std::{
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

#[tokio::test]
/// Testing sync only
async fn test_sync_only_mode() {
    let num_nodes: usize = 5;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    let env = &mut s.swarm;

    println!("1. Set sync_only = true for the last node and check it can sync to others");
    let node = env.validators_mut().nth(4).unwrap();
    let mut new_config = node.config().clone();
    new_config.consensus.sync_only = true;
    update_node_config_restart(node, new_config.clone());
    wait_for_node(node, num_nodes - 1).await;
    // wait for some versions
    env.wait_for_all_nodes_to_catchup_to_version(10, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .expect("could not catc up to version 10");

    println!("2. Set sync_only = true for all nodes and restart");
    for node in env.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = true;
        update_node_config_restart(node, node_config);
        wait_for_node(node, num_nodes - 1).await;
    }

    println!("3. delete one node's db and test they can still sync when sync_only is true for every nodes");
    let node = env.validators_mut().nth(3).unwrap();
    node.stop();
    node.clear_storage().await.unwrap();
    node.start().expect("could not restart node");

    println!("4. verify all nodes are at the same round and no progress being made");
    env.wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();
}
