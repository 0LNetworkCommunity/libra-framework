mod support;
use crate::support::{deadline_secs, update_node_config_restart, wait_for_node};

use diem_api_types::ViewRequest;
use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::{NodeExt, SwarmExt};
use diem_temppath::TempPath;
use diem_types::transaction::Transaction;
use futures_util::future::try_join_all;
use libra_smoke_tests::{helpers::get_libra_balance, libra_smoke::LibraSmoke};
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use smoke_test::test_utils::{swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS};
use std::{fs, time::Duration};

/// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
/// do want the granularity.
/// NOTE: You should `tail` the logs from the advertised logs location. It
/// looks something like this `/tmp/.tmpM9dF7w/0/log`

/// This test verifies the flow of a genesis transaction after the chain starts.
/// 1. Test the consensus sync_only mode, every node should stop at the same version.
/// 2. Test the db-bootstrapper applying a manual genesis transaction (remove validator 0) on diemdb directly
/// 3. Test the nodes and clients resume working after updating waypoint
/// 4. Test a node lagging behind can sync to the waypoint
async fn test_rescue_e2e_with_sync() -> anyhow::Result<()> {
    //increase thread stack size
    let num_nodes: usize = 5;
    let current_path = std::env::current_dir()?;
    //path to diem-node binary
    let diem_node_path = current_path.join("tests/diem-ghost");

    let mut s: LibraSmoke = LibraSmoke::new(Some(num_nodes as u8), Some(diem_node_path.clone()))
        .await
        .expect("could not start libra smoke");

    // clone here to prevent borrow issues
    let client = s.client().clone();

    let env = &mut s.swarm;

    env.wait_for_all_nodes_to_catchup_to_version(10, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("2. Set sync_only = true for all nodes and restart");
    for node in env.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = true;
        update_node_config_restart(node, node_config)?;
        wait_for_node(node, num_nodes - 1).await?;
    }

    println!("4. verify all nodes are at the same round and no progress being made");
    env.wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("5. kill nodes");
    for node in env.validators_mut() {
        node.stop();
    }

    println!("6. prepare a genesis txn to remove the first validator");

    let remove_last = env.validators().last().unwrap().peer_id();
    let script_path = support::make_script(remove_last);
    assert!(script_path.exists());

    let data_path = TempPath::new();
    data_path.create_as_dir().unwrap();
    let rescue = RescueTxOpts {
        data_path: data_path.path().to_owned(),
        blob_path: None, // defaults to data_path/rescue.blob
        script_path: Some(script_path),
        framework_upgrade: false,
        debug_vals: None,
    };
    let genesis_blob_path = rescue.run().unwrap();

    assert!(genesis_blob_path.exists());

    let genesis_transaction = {
        let buf: Vec<u8> = fs::read(&genesis_blob_path).unwrap();
        bcs::from_bytes::<Transaction>(&buf).unwrap()
    };

    let val_db_path = env.validators().next().unwrap().config().storage.dir();
    assert!(val_db_path.exists());

    // replace with rescue cli
    println!("6. check we can get a waypoint generally");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        info: false,
    };

    let _waypoint = bootstrap.run()?;

    println!("7. apply genesis transaction to all validators");
    for (expected_to_connect, node) in env.validators_mut().enumerate() {
        // skip the dead validator
        if node.peer_id() == remove_last {
            continue;
        }
        let mut node_config = node.config().clone();

        let val_db_path = node.config().storage.dir();
        assert!(val_db_path.exists());

        // replace with rescue cli
        println!("7b. each validator db");
        let bootstrap = BootstrapOpts {
            db_dir: val_db_path,
            genesis_txn_file: genesis_blob_path.clone(),
            waypoint_to_verify: None,
            commit: true, // APPLY THE TX
            info: false,
        };

        let waypoint = bootstrap.run().unwrap().unwrap();

        insert_waypoint(&mut node_config, waypoint);
        node_config
            .consensus
            .safety_rules
            .initial_safety_rules_config = InitialSafetyRulesConfig::None;
        node_config.execution.genesis = Some(genesis_transaction.clone());
        // reset the sync_only flag to false
        node_config.consensus.sync_only = false;
        update_node_config_restart(node, node_config)?;
        wait_for_node(node, expected_to_connect).await?;
    }

    println!("8. wait for startup and progress");

    assert!(
        // NOTE: liveness check fails because the test tool doesn't
        // have a way of removing a validator from the test suite. I tried...
        env.liveness_check(deadline_secs(1)).await.is_err(),
        "test suite thinks dead node is live"
    );

    // check some nodes to see if alive, since the test suite doesn't
    // allow us to drop a node
    let _res = try_join_all(
        env.validators()
            .take(3) // check first three
            .map(|node| node.liveness_check(10)),
    )
    .await?;

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

    // have a node sync from the beginning
    // TODO: the order of sync, and tx below, might be inverted
    println!("11. Nuke node 3 and sync from last rescue, test if it rejoins the network okay.");
    let node = env.validators_mut().nth(3).unwrap();
    node.stop();

    let mut node_config = node.config().clone();
    node_config.consensus.sync_only = false;
    node_config.save_to_path(node.config_path()).unwrap();

    let db_dir = node.config().storage.dir();
    fs::remove_dir_all(&db_dir).unwrap();

    node.start().unwrap();
    wait_for_node(node, num_nodes - 2).await?;

    // send a transaction while node 3 is down
    // show progress
    println!("10. verify transactions work");
    std::thread::sleep(Duration::from_secs(5));
    let second_val = env.validators().nth(1).unwrap().peer_id();
    let old_bal = get_libra_balance(&client, second_val).await?;

    s.mint_and_unlock(second_val, 123456).await?;
    let bal = get_libra_balance(&client, second_val).await?;
    assert!(bal.total > old_bal.total, "transaction did not post");

    Ok(())
}

use tokio::runtime::Builder;

#[test]

fn test_with_custom_stack_size() {
    let rt = Builder::new_multi_thread()
        .worker_threads(1) // Set the number of threads
        .thread_stack_size(32 * 1024 * 1024) // Stack size in bytes
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        test_rescue_e2e_with_sync().await.unwrap();
    });
}
