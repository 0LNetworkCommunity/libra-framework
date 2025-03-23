mod support;
use crate::support::{update_node_config_restart, wait_for_node};

use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::SwarmExt;
use diem_types::transaction::Transaction;
use libra_framework::release::ReleaseTarget;
use libra_rescue::{
    diem_db_bootstrapper::BootstrapOpts,
    rescue_cli::{RescueCli, Sub},
};
use libra_smoke_tests::{helpers::get_libra_balance, libra_smoke::LibraSmoke};
use smoke_test::test_utils::{swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS};
use std::{fs, time::Duration};

// #[ignore]
#[tokio::test]
/// This test verifies the flow of a genesis transaction after the chain starts.
/// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
/// do want the granularity.
/// NOTE: You should `tail` the logs from the advertised logs location. It
/// looks something like this `/tmp/.tmpM9dF7w/0/log`
async fn smoke_can_upgrade_and_restart() -> anyhow::Result<()> {
    let num_nodes: usize = 2;
    let mut s = LibraSmoke::new(Some(num_nodes as u8), None)
        .await
        .expect("could not start libra smoke");

    // clone here to prevent borrow issues
    let client = s.client().clone();

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

    let genesis_blob_path = blob_path.join("rescue.blob");
    assert!(genesis_blob_path.exists());

    let genesis_transaction = {
        let buf = fs::read(&genesis_blob_path).unwrap();
        bcs::from_bytes::<Transaction>(&buf).unwrap()
    };

    // replace with rescue cli
    println!("5. check we can get a waypoint generally");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        info: false,
    };

    let _waypoint = bootstrap.run()?;

    println!("6. apply genesis transaction to all validators");
    for (expected_to_connect, node) in env.validators_mut().enumerate() {
        let mut node_config = node.config().clone();

        let val_db_path = node.config().storage.dir();
        assert!(val_db_path.exists());

        // replace with rescue cli
        println!("apply on each validator db");
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

    println!("7. wait for startup and progress");

    // show progress
    println!("8. verify transactions work");
    std::thread::sleep(Duration::from_secs(5));
    let second_val = env.validators().nth(1).unwrap().peer_id();
    let old_bal = get_libra_balance(&client, second_val).await?;
    s.mint_and_unlock(second_val, 123456).await?;
    let bal = get_libra_balance(&client, second_val).await?;
    assert!(bal.total > old_bal.total, "transaction did not post");

    Ok(())
}
