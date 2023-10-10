// use anyhow::anyhow;
use diem_api_types::ViewRequest;
use diem_config::config::{InitialSafetyRulesConfig, NodeConfig};
use diem_forge::{get_highest_synced_version, LocalNode, Node, NodeExt, SwarmExt, Validator};
use diem_logger::prelude::*;
use diem_temppath::TempPath;
use diem_types::{account_address::AccountAddress, transaction::Transaction, waypoint::Waypoint};
use libra_framework::framework_cli::make_template_files;
use libra_smoke_tests::libra_smoke::LibraSmoke;
// use move_core_types::language_storage::CORE_CODE_ADDRESS;
// use regex::Regex;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use smoke_test::{
    storage::{db_backup, db_restore},
    test_utils::{
        swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS, MAX_CONNECTIVITY_WAIT_SECS,
        MAX_HEALTHY_WAIT_SECS,
    },
};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    // str::FromStr,
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

    let env = &mut s.swarm;

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

    // let first_validator_address = AccountAddress::random();

    let script_path = make_script(first_validator_address);
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
        commit: true,
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
    let a = s
        .client()
        .view(
            &ViewRequest {
                function: "0x1::stake::get_current_validators".parse().unwrap(),
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;
    let num_nodes = a.unwrap()
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

    // let client = node.rest_client();
    // // wait for it to catch up
    // {
    //     let version = get_highest_synced_version(&env.get_all_nodes_clients_with_names())
    //         .await
    //         .unwrap();
    //     loop {
    //         if let Ok(resp) = client.get_ledger_information().await {
    //             if resp.into_inner().version > version {
    //                 println!("Node 3 catches up on {}", version);
    //                 break;
    //             }
    //         }
    //         tokio::time::sleep(Duration::from_secs(1)).await;
    //     }
    // }

    // check_create_mint_transfer_node(&mut env, 3).await;
}

#[tokio::test]
/// test the backup and restore works on nodes
async fn test_backup_and_restore() {
    let num_nodes: usize = 5;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    let env = &mut s.swarm;

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

    // let (epoch, version) = {
    //     let response = env
    //         .validators()
    //         .next()
    //         .unwrap()
    //         .rest_client()
    //         .get_ledger_information()
    //         .await
    //         .unwrap();
    //     (response.inner().epoch, response.inner().version)
    // };

    // let (backup_path, _) = db_backup(
    //     env.validators()
    //         .next()
    //         .unwrap()
    //         .config()
    //         .storage
    //         .backup_service_address
    //         .port(),
    //     epoch.checked_sub(1).unwrap(), // target epoch: most recently closed epoch
    //     version,                       // target version
    //     version as usize,              // txn batch size (version 0 is in its own batch)
    //     epoch.checked_sub(1).unwrap() as usize, // state snapshot interval
    //     &[waypoint],
    // );

    // println!("10. nuke DB on node 3, and run db restore, test if it rejoins the network okay.");
    // let node = env.validators_mut().nth(3).unwrap();
    // node.stop();
    // let mut node_config = node.config().clone();
    // node_config.consensus.sync_only = false;
    // node_config.save_to_path(node.config_path()).unwrap();

    // let db_dir = node.config().storage.dir();
    // fs::remove_dir_all(&db_dir).unwrap();
    // db_restore(
    //     backup_path.path(),
    //     db_dir.as_path(),
    //     &[waypoint],
    //     node.config().storage.rocksdb_configs.split_ledger_db,
    //     None,
    // );

    // node.start().unwrap();
    // wait_for_node(node, num_nodes - 2).await;
    // let client = node.rest_client();
    // // wait for it to catch up
    // {
    //     let version = get_highest_synced_version(&env.get_all_nodes_clients_with_names())
    //         .await
    //         .unwrap();
    //     loop {
    //         if let Ok(resp) = client.get_ledger_information().await {
    //             if resp.into_inner().version > version {
    //                 println!("Node 3 catches up on {}", version);
    //                 break;
    //             }
    //         }
    //         tokio::time::sleep(Duration::from_secs(1)).await;
    //     }
    // }

    // check_create_mint_transfer_node(&mut env, 3).await;
}


fn make_script(first_validator_address: AccountAddress) -> PathBuf {
    let script = format!(
        r#"
        script {{
            use diem_framework::stake;
            use diem_framework::diem_governance;
            use diem_framework::block;

            fun main(vm_signer: &signer, framework_signer: &signer) {{
                stake::remove_validators(framework_signer, &vector[@0x{:?}]);
                block::emit_writeset_block_event(vm_signer, @0x1);
                diem_governance::reconfigure(framework_signer);
            }}
    }}
    "#,
        first_validator_address
    );

    let framework_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("framework")
        .join("libra-framework");

    let mut temp_script_path = TempPath::new();
    temp_script_path.create_as_dir().unwrap();
    temp_script_path.persist();

    dbg!(&temp_script_path.path());
    assert!(temp_script_path.path().exists());
    // let mut move_script_path = temp_script_path.path().to_path_buf();
    // move_script_path.set_extension("move");

    make_template_files(
        &temp_script_path.path(),
        &framework_path,
        "rescue",
        Some(script),
    )
    .unwrap();

    temp_script_path.path().to_owned()
}
