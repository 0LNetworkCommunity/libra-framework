mod support;

use std::path::Path;
use crate::support::{deadline_secs, update_node_config_restart};
use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::SwarmExt;
use diem_types::transaction::Transaction;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use smoke_test::test_utils::swarm_utils::insert_waypoint;
use rescue::twin::TwinOpts;
use storage::read_snapshot::{load_snapshot_manifest, accounts_from_snapshot_backup};

#[tokio::test]

// Scenario: create a sample DB from running a swarm of 3. Stop that network,
// and save the db.
// create a new network of 1, just so we can use the configurations.
// change the configurations to point to the old db, after creating a rescue
// which replaces the 3 validators with 1.
async fn test_twin() -> anyhow::Result<()> {
    println!("0. create a valid test database from smoke-tests");
    let num_nodes: usize = 3;

    // Start LibraSmoke to create a test network with `num_nodes` validators
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

    // Stop all validators in the current environment
    for node in env.validators_mut() {
        node.stop();
    }

    println!("1. start new swarm configs, and stop the network");

    // Start a new LibraSmoke instance with only 1 validator
    let _s: LibraSmoke = LibraSmoke::new(Some(1), None)
        .await
        .expect("could not start libra smoke");

    // Retrieve the path of the first validator's config directory
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

    // Prepare options for generating a rescue blob
    let r = RescueTxOpts {
        data_path: brick_db.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: None,
        framework_upgrade: true,
        debug_vals: Some(vec![first_validator_address]),
        snapshot_path: None,
    };
    r.run()?;

    // Validate that the rescue.blob file has been generated
    let file = blob_path.path().join("rescue.blob");
    assert!(file.exists());

    // Deserialize the genesis transaction from the rescue blob
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
    // Apply the genesis transaction to all validators in the environment
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

#[tokio::test]
async fn test_twin_with_snapshot() -> anyhow::Result<()> {

    let prod_db_snapshot = "/path/to/production/db_snapshot.tar.gz";

    let twin_opts = TwinOpts {
        db_dir: prod_db_snapshot.into(),
        oper_file: None,
        info: false,
        snapshot_file: Some(prod_db_snapshot.into()),
    };

    twin_opts.run()?;

    let snapshot_manifest_path = "fixtures/rescue_framework_script/state.manifest"; // Adjust this path
    let snapshot_manifest = load_snapshot_manifest(&snapshot_manifest_path.into())?;

    // Provide archive path (assuming archive path is db_dir)
    let archive_path = Path::new(&prod_db_snapshot);

    // Process account states from snapshot
    let _account_states = accounts_from_snapshot_backup(snapshot_manifest, archive_path)
        .await
        .expect("Could not parse snapshot");

    Ok(())
}
