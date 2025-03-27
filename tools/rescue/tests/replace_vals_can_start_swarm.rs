use anyhow::Result;
use diem_forge::SwarmExt;
use diem_genesis::config::HostAndPort;
use libra_config::validator_config;
use libra_framework::release::ReleaseTarget;
use libra_rescue::cli_main::REPLACE_VALIDATORS_BLOB;
use libra_rescue::node_config::post_rescue_node_file_updates;
use libra_rescue::test_support::{update_node_config_restart, wait_for_node};
use libra_rescue::{
    cli_bootstrapper::BootstrapOpts,
    cli_main::{RescueCli, Sub, UPGRADE_FRAMEWORK_BLOB},
};
use libra_smoke_tests::{helpers::get_libra_balance, libra_smoke::LibraSmoke};
use libra_wallet::core::wallet_library::WalletLibrary;
use smoke_test::test_utils::MAX_CATCH_UP_WAIT_SECS;
use std::path::{Path, PathBuf};
use std::time::Duration;

async fn test_setup_start_then_pause() -> Result<LibraSmoke> {
    let num_nodes: usize = 2;
    let mut s = LibraSmoke::new(Some(num_nodes as u8), None)
        .await
        .expect("could not start libra smoke");

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
    Ok(s)
}

async fn copy_dir(from: &Path, to: &str) -> Result<()> {
    // Get the current directory using CARGO_MANIFEST_DIR
    let current_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set")
        .into();
    let to_dir = current_dir.join(to);
    if to_dir.exists() {
        tokio::fs::remove_dir_all(&to_dir).await?;
    }
    tokio::fs::create_dir_all(&to).await?;
    fs_extra::dir::copy(from, &to_dir, &fs_extra::dir::CopyOptions::new().content_only(true))?;
    Ok(())
}
async fn creates_random_val_account(
    data_path: &Path,
    my_host: &HostAndPort,
) -> anyhow::Result<WalletLibrary> {
    let wallet = WalletLibrary::new();
    let mnemonic_string = wallet.mnemonic();
    // Initializes the validator configuration.
    validator_config::initialize_validator(
        Some(data_path.to_path_buf()),
        Some(&mnemonic_string.clone()[..3]),
        my_host.clone(),
        Some(mnemonic_string),
        false,
        Some(diem_types::chain_id::NamedChain::TESTING),
    )
    .await?;

    Ok(wallet)
}

async fn make_test_randos(smoke: &LibraSmoke) -> anyhow::Result<()> {
    let env = &smoke.swarm;

    let rando_dir = env.dir().join("rando");

    for (i, local_node) in env.validators().enumerate() {
        let host = HostAndPort::local(local_node.port())?;
        creates_random_val_account(&rando_dir.join(&i.to_string()), &host).await?;
    }
    Ok(())
}

#[tokio::test]
async fn meta_can_add_random_vals() -> anyhow::Result<()> {
    let s = test_setup_start_then_pause().await?;

    make_test_randos(&s).await?;

    copy_dir(&s.swarm.dir(), "post_random_init").await?;

    Ok(())
}

#[tokio::test]
async fn can_create_replace_blob_from_randos() -> anyhow::Result<()> {
    let s = test_setup_start_then_pause().await?;

    make_test_randos(&s).await?;
    let data_dir = s.swarm.dir();
    copy_dir(data_dir, "post_random_init").await?;

    let val_db_path = s.swarm.validators().nth(0).unwrap().config().storage.dir();

    // use glob to find the operator.yaml under the data_dir/randos
    // Use glob to find all operator YAML files in the rando directory
    let rando_dir = data_dir.join("rando");
    let operator_yaml_pattern = format!("{}/**/operator.yaml", rando_dir.display());
    let operator_yamls: Vec<PathBuf> = glob::glob(&operator_yaml_pattern)?
        .filter_map(Result::ok)
        .collect();

    if operator_yamls.is_empty() {
        anyhow::bail!("No operator.yaml files found in rando directory");
    }

    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(data_dir.to_path_buf()), // save to top level test dir
        command: Sub::RegisterVals {
            operator_yaml: operator_yamls,
            upgrade_mrb: None,
        },
    };

    r.run()?;

    copy_dir(data_dir, "post_blob").await?;

    let genesis_blob_path = data_dir.join(REPLACE_VALIDATORS_BLOB);

    // check it bootstraps while we are here
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        update_node_config: None,
        info: false,
    };
    assert!(bootstrap.run()?.is_some(), "no waypoint");

    Ok(())
}

#[tokio::test]
async fn can_write_to_db_from_randos() -> anyhow::Result<()> {
    let s = test_setup_start_then_pause().await?;

    make_test_randos(&s).await?;
    let data_dir = s.swarm.dir();
    copy_dir(data_dir, "post_random_init").await?;

    let val_db_path = s.swarm.validators().nth(0).unwrap().config().storage.dir();

    // use glob to find the operator.yaml under the data_dir/randos
    // Use glob to find all operator YAML files in the rando directory
    let rando_dir = data_dir.join("rando");
    let operator_yaml_pattern = format!("{}/**/operator.yaml", rando_dir.display());
    let operator_yamls: Vec<PathBuf> = glob::glob(&operator_yaml_pattern)?
        .filter_map(Result::ok)
        .collect();

    if operator_yamls.is_empty() {
        anyhow::bail!("No operator.yaml files found in rando directory");
    }

    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(data_dir.to_path_buf()), // save to top level test dir
        command: Sub::RegisterVals {
            operator_yaml: operator_yamls,
            upgrade_mrb: None,
        },
    };

    r.run()?;

    copy_dir(data_dir, "post_blob").await?;

    let genesis_blob_path = data_dir.join(REPLACE_VALIDATORS_BLOB);

    // check it bootstraps while we are here
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path.clone(),
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        update_node_config: None,
        info: false,
    };
    let wp = bootstrap.run()?.expect("waypoint not found");

    // replace with rescue cli
    println!("apply on each validator db");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path.clone(),
        waypoint_to_verify: Some(wp),
        commit: true, // APPLY THE TX
        update_node_config: None,
        info: false,
    };
    let wp2 = bootstrap.run()?.expect("waypoint 2 not found");
    assert!(wp == wp2, "waypints don't match");

    copy_dir(data_dir, "post_rescue").await?;

    Ok(())
}


// #[tokio::test]
// async fn can_bootstrap_with_replace_blob_initial_db() -> anyhow::Result<()> {
//     let s = test_setup_start_then_pause().await?;

//     make_test_randos(&s).await?;
//     let data_dir = s.swarm.dir();
//     copy_dir(data_dir, "post_random_init").await?;

//     let val_db_path = s.swarm.validators().nth(0).unwrap().config().storage.dir();

//     // use glob to find the operator.yaml under the data_dir/randos
//     // Use glob to find all operator YAML files in the rando directory
//     let rando_dir = data_dir.join("rando");
//     let operator_yaml_pattern = format!("{}/**/operator.yaml", rando_dir.display());
//     let operator_yamls: Vec<PathBuf> = glob::glob(&operator_yaml_pattern)?
//         .filter_map(Result::ok)
//         .collect();

//     if operator_yamls.is_empty() {
//         anyhow::bail!("No operator.yaml files found in rando directory");
//     }

//     let r = RescueCli {
//         db_path: val_db_path.clone(),
//         blob_path: Some(data_dir.to_path_buf()), // save to top level test dir
//         command: Sub::RegisterVals {
//             operator_yaml: operator_yamls,
//             upgrade_mrb: None,
//         },
//     };

//     r.run()?;

//     copy_dir(data_dir, "post_rescue").await?;

//     Ok(())
// }

// #[tokio::test]
// /// This test verifies the flow of a genesis transaction after the chain starts.
// /// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
// /// do want the granularity.
// /// NOTE: You should `tail` the logs from the advertised logs location. It
// /// looks something like this `/tmp/.tmpM9dF7w/0/log`
// async fn smoke_can_replace_vals_and_restart() -> anyhow::Result<()> {
//     let mut s = test_setup_start_then_pause().await?;
//     // clone here to prevent borrow issues
//     let client = s.client().clone();
//     let env = &mut s.swarm;

//     let val_db_path = env.validators().next().unwrap().config().storage.dir();
//     assert!(val_db_path.exists());
//     let blob_path = val_db_path.clone();

//     let r = RescueCli {
//         db_path: val_db_path.clone(),
//         blob_path: Some(blob_path.clone()),
//         command: Sub::UpgradeFramework {
//             upgrade_mrb: ReleaseTarget::Head
//                 .find_bundle_path()
//                 .expect("cannot find head.mrb"),
//             set_validators: None,
//         },
//     };

//     r.run()?;

//     let genesis_blob_path = blob_path.join(UPGRADE_FRAMEWORK_BLOB);
//     assert!(genesis_blob_path.exists());

//     // replace with rescue cli
//     println!("5. check we can get a waypoint generally");
//     let bootstrap = BootstrapOpts {
//         db_dir: val_db_path,
//         genesis_txn_file: genesis_blob_path.clone(),
//         waypoint_to_verify: None,
//         commit: false, // NOT APPLYING THE TX
//         update_node_config: None,
//         info: false,
//     };

//     let _waypoint = bootstrap.run()?;

//     println!("6. apply genesis transaction to all validators");
//     for (expected_to_connect, node) in env.validators_mut().enumerate() {
//         node.stop();

//         let val_db_path = node.config().storage.dir();
//         assert!(val_db_path.exists());

//         // replace with rescue cli
//         println!("apply on each validator db");
//         let bootstrap = BootstrapOpts {
//             db_dir: val_db_path,
//             genesis_txn_file: genesis_blob_path.clone(),
//             waypoint_to_verify: None,
//             commit: true, // APPLY THE TX
//             update_node_config: None,
//             info: false,
//         };

//         let waypoint = bootstrap.run().unwrap().unwrap();

//         // voodoo to update node config
//         post_rescue_node_file_updates(&node.config_path(), waypoint, &genesis_blob_path)?;

//         node.start()?;
//         wait_for_node(node, expected_to_connect).await?;
//     }

//     println!("7. wait for startup and progress");
//     let res = client.get_index().await?;
//     let block_height_pre = res.inner().block_height.inner();

//     std::thread::sleep(Duration::from_secs(5));
//     let i = client.get_index().await?;
//     assert!(
//         i.inner().block_height.inner() > block_height_pre,
//         "chain isn't making progress"
//     );

//     // show progress
//     println!("8. verify transactions work");

//     let second_val = env.validators().nth(1).unwrap().peer_id();
//     let old_bal = get_libra_balance(&client, second_val).await?;
//     s.mint_and_unlock(second_val, 123456).await?;
//     let bal = get_libra_balance(&client, second_val).await?;
//     assert!(bal.total > old_bal.total, "transaction did not post");

//     Ok(())
// }
