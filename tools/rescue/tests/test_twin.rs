mod support;
use crate::support::{deadline_secs, update_node_config_restart};

use std::str::FromStr;
use std::{fs, path::PathBuf};

use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::SwarmExt;
use diem_types::transaction::Transaction;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts, user_file::UserBlob};
use serde_json::json;
use smoke_test::test_utils::swarm_utils::insert_waypoint;

#[tokio::test]

// Scenario: create a sample DB from running a swarm of 3. Stop that network,
// and save the db.
// create a new network of 1, just so we can use the configurations.
// change the configurations to point to the old db, after creating a rescue
// which replaces the 3 validators with 1.

async fn test_twin() -> anyhow::Result<()> {
    println!("0. create a valid test database from smoke-tests");
    let num_nodes: usize = 3;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    let env = &mut s.swarm;
    println!(
        "Number of validators in the swarm: {}",
        env.validators().count()
    );

    let brick_db = env.validators().next().unwrap().config().storage.dir();
    assert!(brick_db.exists());

    for node in env.validators_mut() {
        node.stop();
    }

    println!("1. start new swarm configs, and stop the network");
    let _s = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");

    let first_validator_address = env
        .validators()
        .next()
        .unwrap()
        .config()
        .get_peer_id()
        .unwrap();

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    println!("2. compile the script");

    let j = json!(UserBlob {
        account: first_validator_address
    });

    let temp = diem_temppath::TempPath::new();
    temp.create_as_file()?;
    fs::write(temp.path(), j.to_string())?;

    let this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;
    let mrb_path = this_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("framework/releases/head.mrb");

    let r = RescueTxOpts {
        db_dir: brick_db.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: None,
        framework_mrb_file: Some(mrb_path),
        validators_file: Some(temp.path().to_owned()),
    };
    r.run()?;

    let file = blob_path.path().join("rescue.blob");
    assert!(file.exists());

    let genesis_transaction = {
        let buf = std::fs::read(&file).unwrap();
        bcs::from_bytes::<Transaction>(&buf).unwrap()
    };

    println!("3. get waypoint");
    let bootstrap = BootstrapOpts {
        db_dir: brick_db,
        genesis_txn_file: file.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        info: false,
    };

    let waypoint = bootstrap.run()?;
    dbg!(&waypoint);

    //////////

    println!("4. apply genesis transaction to all validators");
    for (_expected_to_connect, node) in env.validators_mut().enumerate() {
        let mut node_config = node.config().clone();

        let val_db_path = node.config().storage.dir();
        assert!(val_db_path.exists());

        // replace with rescue cli
        println!("7b. each validator db");
        let bootstrap = BootstrapOpts {
            db_dir: val_db_path,
            genesis_txn_file: file.clone(),
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
        // wait_for_node(node, expected_to_connect).await?;
    }

    assert!(
        // NOTE: liveness check fails because the test tool doesn't
        // have a way of removing a validator from the test suite. I tried...
        env.liveness_check(deadline_secs(1)).await.is_err(),
        "test suite thinks dead node is live"
    );

    Ok(())
}
