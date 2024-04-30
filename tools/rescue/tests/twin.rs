mod support;

use crate::support::{deadline_secs, update_node_config_restart};

use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::SwarmExt;
use diem_types::transaction::Transaction;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
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

    // The diem-node should be compiled externally to avoid any potential conflicts with the current build
    //get the current path
    let mut s = LibraSmoke::new(Some(num_nodes as u8), None)
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
    let _s: LibraSmoke = LibraSmoke::new(Some(1), None)
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

    let r = RescueTxOpts {
        data_path: brick_db.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: None,
        framework_upgrade: true,
        debug_vals: Some(vec![first_validator_address]),
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
    for node in env.validators_mut() {
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
