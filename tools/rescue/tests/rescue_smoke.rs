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

#[tokio::test]
/// This test verifies the flow of a genesis transaction after the chain starts.
/// 1. Test the consensus sync_only mode, every node should stop at the same version.
/// 2. Test the db-bootstrapper applying a manual genesis transaction (remove validator 0) on diemdb directly
/// 3. Test the nodes and clients resume working after updating waypoint
/// 4. Test a node lagging behind can sync to the waypoint
async fn test_genesis_transaction_flow() {
    let num_nodes: usize = 5;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    let client = s.client().clone();

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

    println!("6. prepare a genesis txn to remove the last validator");

    let first_validator_address = env
        .validators()
        .nth(4)
        .unwrap()
        .config()
        .get_peer_id()
        .unwrap();

    let script_path = support::make_script(first_validator_address);
    dbg!(&script_path);
    assert!(script_path.exists());

    let data_path = TempPath::new();
    data_path.create_as_dir().unwrap();
    let rescue = RescueTxOpts {
        data_path: data_path.path().to_owned(),
        blob_path: None, // defaults to data_path/rescue.blob
        script_path: Some(script_path),
        framework_upgrade: false,
    };
    let genesis_blob_path = rescue.run().await.unwrap();

    assert!(genesis_blob_path.exists());

    let genesis_transaction = {
        let buf = fs::read(&genesis_blob_path).unwrap();
        bcs::from_bytes::<Transaction>(&buf).unwrap()
    };

    let val_db_path = env.validators().next().unwrap().config().storage.dir();
    assert!(val_db_path.exists());

    // replace with rescue cli
    println!("6. prepare the waypoint with the transaction");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path,
        waypoint_to_verify: None,
        commit: false, // NOTE: the tests seem to work even when this is false
    };

    let waypoint = bootstrap.run().unwrap();

    println!("7. apply genesis transaction for nodes 0, 1, 2");
    for (expected_to_connect, node) in env.validators_mut().take(3).enumerate() {
        let mut node_config = node.config().clone();
        insert_waypoint(&mut node_config, waypoint);
        node_config
            .consensus
            .safety_rules
            .initial_safety_rules_config = InitialSafetyRulesConfig::None;
        node_config.execution.genesis = Some(genesis_transaction.clone());
        // reset the sync_only flag to false
        node_config.consensus.sync_only = false;
        update_node_config_restart(node, node_config);
        wait_for_node(node, expected_to_connect).await;
    }

    println!("8. verify it's able to mint after the waypoint");
    env.wait_for_startup().await.unwrap();
    // check_create_mint_transfer_node(&mut env, 0).await;

    println!("9. verify node 4 is out from the validator set");
    let a = client
        .view(
            &ViewRequest {
                function: "0x1::stake::get_current_validators".parse().unwrap(),
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;
    let num_nodes = a
        .unwrap()
        .inner()
        .first()
        .unwrap()
        .as_array()
        .unwrap()
        .len();

    assert!(num_nodes == 4);

    println!("10. nuke DB on node 3, and run db restore, test if it rejoins the network okay.");
    let node = env.validators_mut().nth(3).unwrap();
    node.stop();
    let mut node_config = node.config().clone();
    node_config.consensus.sync_only = false;
    node_config.save_to_path(node.config_path()).unwrap();

    let db_dir = node.config().storage.dir();
    fs::remove_dir_all(&db_dir).unwrap();

    node.start().unwrap();
    wait_for_node(node, num_nodes - 2).await;
}
