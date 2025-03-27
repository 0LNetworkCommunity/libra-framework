use std::path::PathBuf;
use std::time::Duration;

use diem_forge::SwarmExt;
use libra_framework::release::ReleaseTarget;
use libra_rescue::cli_bootstrapper::BootstrapOpts;
use libra_rescue::cli_main::RescueCli;
use libra_rescue::cli_main::Sub;
use libra_rescue::cli_main::UPGRADE_FRAMEWORK_BLOB;
use libra_rescue::node_config::post_rescue_node_file_updates;
use libra_rescue::test_support::update_node_config_restart;
use libra_rescue::test_support::wait_for_node;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm;
use smoke_test::test_utils::MAX_CATCH_UP_WAIT_SECS;

// start a swarm, to produce some db artifacts
// and use those artifacts to create a twin swarm
// it's effectively a noop, that just tests the tooling
#[tokio::test]
async fn test_twin_swarm_noop() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(Some(1), None).await?;

    twin_swarm::awake_frankenswarm(&mut smoke, None).await?;
    Ok(())
}

#[ignore]
#[tokio::test]
async fn alt_noop() -> anyhow::Result<()> {
    let num_nodes: usize = 1;
    let mut s = LibraSmoke::new(Some(num_nodes as u8), None)
        .await
        .expect("could not start libra smoke");

    // Get the current directory using CARGO_MANIFEST_DIR
    let current_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set")
        .into();
    println!("Current directory: {}", current_dir.display());
    // clone here to prevent borrow issues
    // let client = s.client().clone();

    let env: &mut diem_forge::LocalSwarm = &mut s.swarm;

    env.wait_for_all_nodes_to_catchup_to_version(10, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("1. Set sync_only = true for all nodes and restart");
    for node in env.validators_mut() {
        let mut node_config = node.config().clone();

        node_config.consensus.sync_only = true;
        update_node_config_restart(node, node_config)?;
        wait_for_node(node, num_nodes - 1).await?;
    }

    println!("2. verify all nodes are at the same round and no progress being made");
    env.wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("3. stop nodes");

    for node in env.validators_mut() {
        node.stop();
        let dir = current_dir.join("pre_upgrade");
        tokio::fs::create_dir_all(&dir).await?;
        fs_extra::dir::copy(
            node.config().get_data_dir(),
            &dir,
            &fs_extra::dir::CopyOptions::new(),
        )?;
    }

    println!("4. create writeset file");

    let val_db_path = env.validators().next().unwrap().config().storage.dir();
    assert!(val_db_path.exists());
    let blob_path = val_db_path.clone();

    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(blob_path.clone()),
        command: Sub::UpgradeFramework {
            upgrade_mrb: ReleaseTarget::Head
                .find_bundle_path()
                .expect("cannot find head.mrb"),
            set_validators: None,
        },
    };

    r.run()?;

    let genesis_blob_path = blob_path.join(UPGRADE_FRAMEWORK_BLOB);
    assert!(genesis_blob_path.exists());

    // replace with rescue cli
    println!("5. check we can get a waypoint generally");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        update_node_config: None,
        info: false,
    };

    let _waypoint = bootstrap.run()?;

    println!("6. apply genesis transaction to all validators");
    for (expected_to_connect, node) in env.validators_mut().enumerate() {
        node.stop();

        let val_db_path = node.config().storage.dir();
        assert!(val_db_path.exists());

        // replace with rescue cli
        println!("apply on each validator db");
        let bootstrap = BootstrapOpts {
            db_dir: val_db_path,
            genesis_txn_file: genesis_blob_path.clone(),
            waypoint_to_verify: None,
            commit: true, // APPLY THE TX
            update_node_config: None,
            info: false,
        };

        let waypoint = bootstrap.run().unwrap().unwrap();

        // voodoo to update node config
        post_rescue_node_file_updates(&node.config_path(), waypoint, &genesis_blob_path)?;

        let dir = current_dir.join("post_upgrade");
        tokio::fs::create_dir_all(&dir).await?;
        fs_extra::dir::copy(
            node.config().get_data_dir(),
            &dir,
            &fs_extra::dir::CopyOptions::new(),
        )?;
        node.start()?;
        wait_for_node(node, expected_to_connect).await?;
    }

    Ok(())
}
