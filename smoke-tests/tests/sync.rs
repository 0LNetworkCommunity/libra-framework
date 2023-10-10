
use diem_config::config::NodeConfig;
use diem_forge::{LocalNode, Node, NodeExt, SwarmExt, Validator};
use diem_temppath::TempPath;
use diem_types::{account_address::AccountAddress,
};
use diem_logger::prelude::*;
use libra_framework::framework_cli::make_template_files;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use smoke_test::test_utils::{
    MAX_CATCH_UP_WAIT_SECS,
    MAX_CONNECTIVITY_WAIT_SECS,
    MAX_HEALTHY_WAIT_SECS,
};
use std::{
    path::PathBuf,
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
