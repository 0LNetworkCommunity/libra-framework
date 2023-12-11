mod support;
use crate::support::{deadline_secs, update_node_config_restart, wait_for_node};
use diem_api_types::{EntryFunctionId, ViewRequest};
use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::{NodeExt, SwarmExt};
use diem_temppath::TempPath;
use diem_types::transaction::Transaction;
use futures_util::future::try_join_all;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use move_core_types::value::MoveValue;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use serde_json::json;
use smoke_test::test_utils::{self, swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS};
use std::f32::consts::E;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, time::Duration};
use libra_smoke_tests::helpers::get_libra_balance;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
// #[ignore]
#[tokio::test]
/// This test verifies the flow of a genesis transaction after the chain starts.
/// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
/// do want the granularity.
/// NOTE: You should `tail` the logs from the advertised logs location. It
/// looks something like this `/tmp/.tmpM9dF7w/0/log`
async fn test_framework_upgrade_has_new_module() -> anyhow::Result<()> {
    let num_nodes: usize = 5;

    let mut s = LibraSmoke::new(Some(num_nodes as u8))
        .await
        .expect("could not start libra smoke");

    // clone here to prevent borrow issues
    let client = s.client().clone();

    let env: &mut diem_forge::LocalSwarm = &mut s.swarm;

    // this should produce an error
    let v = client
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str("0x1::all_your_base::are_belong_to")?,
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;
    assert!(v.is_err(), "cannot call function");
    dbg!(&v);

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
    let res = client
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str("0x1::epoch_helper::get_current_epoch")?,
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;

    dbg!(&res);

    println!("5. kill nodes");
    for node in env.validators_mut() {
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
        script_path,
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

    let waypoint = bootstrap.run()?;

    println!("7. apply genesis transaction to all validators");
    for (_expected_to_connect, node) in env.validators_mut().enumerate() {
        let mut node_config = node.config().clone();

        let val_db_path = node.config().storage.dir();
        assert!(val_db_path.exists());

        // replace with rescue cli
        println!("7b. each validator db");
        let bootstrap = BootstrapOpts {
            db_dir: val_db_path,
            genesis_txn_file: genesis_blob_path.clone(),
            waypoint_to_verify: Some(waypoint),
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

    println!("8. wait for startup and progress");

    assert!(
        env.liveness_check(deadline_secs(30)).await.is_ok(),
        "not all nodes connected after restart"
    );

    // let git_v = client.get_account_resource(CORE_CODE_ADDRESS, "0x1::version::Git").await?;
    // dbg!(&git_v);



    // // show progress
    // println!("10. verify transactions work");
    // std::thread::sleep(Duration::from_secs(5));
    // let second_val = env.validators().nth(1).unwrap().peer_id();
    // let old_bal = get_libra_balance(&client, second_val).await?;
    // s.mint_and_unlock(second_val, 123456).await?;
    // let bal = get_libra_balance(&client, second_val).await?;
    // assert!(bal.total > old_bal.total, "transaction did not post");

    // let res = client
    //     .view(
    //         &ViewRequest {
    //             function: EntryFunctionId::from_str("0x1::code::get_module_names_for_package_index")?,
    //             type_arguments: vec![],
    //             arguments: vec![
    //               json!(CORE_CODE_ADDRESS.to_string()),
    //               json!("0"),
    //             ],
    //         },
    //         None,
    //     )
    //     .await;

    // dbg!(&v);

    let res = client
        .view(
            &ViewRequest {
                function: EntryFunctionId::from_str("0x1::epoch_helper::get_current_epoch")?,
                type_arguments: vec![],
                arguments: vec![],
            },
            None,
        )
        .await;

    dbg!(&res);

    // let (_val, rest) = s.swarm.get_all_nodes_clients_with_names().iter().nth(0).unwrap();


    let v = client
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

    Ok(())
}
