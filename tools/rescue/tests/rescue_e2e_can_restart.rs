mod support;

use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::SwarmExt;
use diem_temppath::TempPath;
use diem_types::transaction::Transaction;
use libra_smoke_tests::{libra_smoke::LibraSmoke, helpers::get_libra_balance};
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use smoke_test::test_utils::{swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS};
use std::{fs, time::{Duration, Instant}};

use crate::support::{update_node_config_restart, wait_for_node, deadline_secs};

// #[ignore]
#[tokio::test]
/// This test verifies the flow of a genesis transaction after the chain starts.
/// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
/// do want the granularity.
async fn test_can_restart() -> anyhow::Result<()> {
    let num_nodes: usize = 5;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    // clone here to prevent borrow issues
    let client = s.client().clone();

    let env: &mut diem_forge::LocalSwarm = &mut s.swarm;

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

    let remove_last = env.validators().last().unwrap();
    let script_path = support::make_script(remove_last.peer_id());
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
    println!("6. check we can get a waypoint generally");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
    };

    let _waypoint = bootstrap.run()?;

    println!("7. apply genesis transaction to all validators");
    for (expected_to_connect, node) in env.validators_mut().enumerate() {
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
        };

        let waypoint = bootstrap.run().unwrap();

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

    // wait_for_node(env.validators().nth(0).unwrap(), expected_to_connect).await?;

    println!("8. wait for startup and progress");
    env.wait_for_startup().await?;
    // assert!(env.fork_check().is_ok(), "fork detected");

    // assert!(env.liveness_check(deadline_secs(5)).await.is_ok(), "network did not restart");
    // env.validators.iter().for_each(|(addr, _node)| {
    //   env.liveness_check(format_deadline(1))
    // });
    // // todo show progress
    // println!("9. verify trasnsactions work");

    // let rando = s.marlon_rando();
    // s.mint_and_unlock(rando.address(), 123456).await?;

    // let b = get_libra_balance(&client, rando.address()).await?;
    // dbg!(&b);
    Ok(())
}
