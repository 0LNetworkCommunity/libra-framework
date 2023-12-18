mod support;
use crate::support::{deadline_secs, update_node_config_restart, wait_for_node};
use diem_api_types::{EntryFunctionId, ViewRequest};
use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::{NodeExt, SwarmExt};
use diem_temppath::TempPath;
use diem_types::transaction::Transaction;
use libra_smoke_tests::helpers::get_libra_balance;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use serde_json::json;
use smoke_test::test_utils::{swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS};
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, time::Duration};

// #[ignore]
#[tokio::test]
/// This test verifies the flow of a genesis transaction after the chain starts.
/// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
/// do want the granularity.
/// NOTE: You should `tail` the logs from the advertised logs location. It
/// looks something like this `/tmp/.tmpM9dF7w/0/log`
async fn test_framework_upgrade_has_new_module() -> anyhow::Result<()> {
    let num_nodes: usize = 4;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    // this should produce an error
    let v = s
        .client()
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str("0x1::all_your_base::are_belong_to")?,
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;
    assert!(
        v.is_err(),
        "all_your_base::are_belong_to is found when it shouldn't be published yet"
    );

    s.swarm
        .wait_for_all_nodes_to_catchup_to_version(10, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await
        .unwrap();

    println!("2. Set sync_only = true for all nodes and restart");
    for node in s.swarm.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = true;
        update_node_config_restart(node, node_config)?;
        wait_for_node(node, num_nodes - 1).await?;
    }

    println!("4. verify all nodes are at the same round and no progress being made");
    s.swarm
        .wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await?;

    println!("5. kill nodes");
    for node in s.swarm.validators_mut() {
        node.stop();
    }

    println!("6. prepare a fork with the new FRAMEWORK");

    let data_path = TempPath::new();
    data_path.create_as_dir().unwrap();

    let rescue_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script_path = rescue_script
        .join("fixtures")
        .join("rescue_framework_script");

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
        let buf = fs::read(&genesis_blob_path).unwrap();
        bcs::from_bytes::<Transaction>(&buf).unwrap()
    };

    let val_db_path = s.swarm.validators().next().unwrap().config().storage.dir();
    assert!(val_db_path.exists());

    // replace with rescue cli
    println!("6. check we can get a waypoint using db, don't commit");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
    };

    let waypoint_check = bootstrap.run()?;

    println!("7. apply genesis transaction to all validators");
    for (idx, node) in s.swarm.validators_mut().enumerate() {
        let mut node_config = node.config().clone();

        let val_db_path = node.config().storage.dir();
        assert!(val_db_path.exists());

        // replace with rescue cli
        println!("7b. each validator db {}", idx);
        let bootstrap = BootstrapOpts {
            db_dir: val_db_path,
            genesis_txn_file: genesis_blob_path.clone(),
            waypoint_to_verify: Some(waypoint_check),
            commit: true, // APPLY THE TX
        };

        let waypoint = bootstrap.run().unwrap();
        assert!(waypoint_check == waypoint, "waypoint mismatch");

        insert_waypoint(&mut node_config, waypoint);
        node_config
            .consensus
            .safety_rules
            .initial_safety_rules_config = InitialSafetyRulesConfig::None;
        node_config.execution.genesis = Some(genesis_transaction.clone());
        // reset the sync_only flag to false
        node_config.consensus.sync_only = false;
        update_node_config_restart(node, node_config)?;
        wait_for_node(node, idx).await?;
    }

    println!("8. wait for startup and progress");

    s.swarm.wait_for_startup().await?;
    // assert!(
    //     s.swarm.liveness_check(deadline_secs(60)).await.is_ok(),
    //     "not all nodes connected after restart"
    // );

    s.swarm
        .wait_for_all_nodes_to_catchup_to_version(100, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await?;

    // show progress
    println!("10. verify transactions work");
    std::thread::sleep(Duration::from_secs(5));
    let second_val = s.swarm.validators().nth(1).unwrap().peer_id();
    let client = s.client();
    let old_bal = get_libra_balance(&client, second_val).await?;
    s.mint_and_unlock(second_val, 123456).await?;
    let bal = get_libra_balance(&client, second_val).await?;
    assert!(bal.total > old_bal.total, "transaction did not post");

    let res = client
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str(
                    "0x1::code::get_module_names_for_package_index",
                )?,
                type_arguments: vec![],
                arguments: vec![json!(CORE_CODE_ADDRESS.to_string()), json!("0")],
            },
            None,
        )
        .await?;
    assert!(
        res.inner()[0].as_array().unwrap()[0].as_str().unwrap() == "all_your_base",
        "all_your_base.move not included in package metadata"
    );

    let v = s
        .client()
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str("0x1::all_your_base::are_belong_to")?,
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;

    dbg!(&v);

    let val = s.swarm.validators_mut().next().unwrap();
    val.restart().await?;
    val.wait_until_healthy(deadline_secs(10)).await?;
    let rc = val.rest_client();

    let res = rc
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str("0x1::all_your_base::are_belong_to")?,
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;
    dbg!(&res);

    std::thread::sleep(Duration::from_secs(100));
    Ok(())
}
