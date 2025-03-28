use anyhow::Result;
use diem_forge::{LocalSwarm, SwarmExt};
use diem_genesis::config::HostAndPort;
use diem_types::waypoint::Waypoint;
use fs_extra::dir;
use libra_config::validator_config;
use libra_rescue::cli_bootstrapper::one_step_apply_rescue_on_db;
use libra_rescue::cli_main::REPLACE_VALIDATORS_BLOB;
use libra_rescue::node_config::post_rescue_node_file_updates;
use libra_rescue::test_support::{update_node_config_restart, wait_for_node};
use libra_rescue::{
    cli_bootstrapper::BootstrapOpts,
    cli_main::{RescueCli, Sub},
};
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_wallet::core::wallet_library::WalletLibrary;
use smoke_test::test_utils::swarm_utils::insert_waypoint;
use smoke_test::test_utils::MAX_CATCH_UP_WAIT_SECS;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;

// Note: swarm has several limitations
// Most swarm and LocalNode properties are not mutable
// after init.
// For example: it's not possible to point a swarm node to a new config file
// in a different directory. It is always self.directory.join("node.yaml")
// Similarly it's not possible to change the directory itself.
// So these tests are a bit of a hack.
// we replace some files in the original directories that
// swarm created, using same file names. The function used is called
// brain_salad_surgery().

/// swaps out the
/// preserving the external structure.
async fn brain_salad_surgery(swarm: &LocalSwarm) -> Result<()> {
    // Write the modified config back to the original location
    let swarm_dir = swarm.dir();
    let backup_path = swarm_dir.join("bak".to_string());

    for (i, n) in swarm.validators().enumerate() {
        let node_data_path = n.config_path();
        let node_data_path = node_data_path.parent().unwrap();

        if !backup_path.exists() {
            fs::create_dir_all(&backup_path).await?;
        }
        fs_extra::move_items(&[node_data_path], &backup_path, &dir::CopyOptions::new())?;

        if !node_data_path.exists() {
            fs::create_dir_all(&node_data_path).await?;
        }
        let cfg = n.config();
        let net = cfg.validator_network.iter().next().unwrap();
        let port = net.listen_address.find_port().expect("to find port");
        creates_random_val_account(node_data_path, port).await?;

        // Swarm uses a fixed node.yaml for the config file,
        // while we use role-based names.
        // we'll deprecate the one we would normally use.
        fs::rename(
            node_data_path.join("validator.yaml"),
            node_data_path.join("validator.depr"),
        )
        .await?;

        fs::rename(
            node_data_path.join("public-keys.yaml"),
            node_data_path.join("public-identity.yaml"),
        )
        .await?;
        fs::rename(
            node_data_path.join("private-keys.yaml"),
            node_data_path.join("private-identity.yaml"),
        )
        .await?;

        fs::rename(
            node_data_path.join("validator-full-node-identity.yaml"),
            node_data_path.join("vfn-identity.yaml"),
        )
        .await?;
        // now copy back the node.yaml, it has randomly generated addresses
        // and is generally in a format the swarm tool will understand
        fs::copy(
            backup_path.join(format!("{i}/node.yaml")),
            node_data_path.join("node.yaml"),
        )
        .await?;
        // // now copy back the genesis.blob
        // fs::copy(
        //     backup_path.join(&format!("{i}/genesis.blob")),
        //     node_data_path.join("genesis.blob"),
        // )
        // .await?;

        // fs::copy(
        //     backup_path.join(&format!("{i}/secure_storage.json")),
        //     node_data_path.join("secure_storage.json"),
        // )
        // .await?;

        // and copy the db which we will modify
        fs_extra::dir::copy(
            backup_path.join(format!("{i}/db")),
            node_data_path,
            &dir::CopyOptions::new(),
        )?;
    }

    // We've got a ballad
    // About a salad brain
    // With assurgence
    // In a dirty bit again

    // [Verse 2]
    // Brain salad certainty
    // It will work for you, it works for me
    // Brain rot perversity
    // Brain salad surgery
    Ok(())
}

#[tokio::test]
// can restart after brain salad surgery
// but will not progress
async fn test_brain_salad() -> anyhow::Result<()> {
    let mut s = test_setup_start_then_pause().await?;
    let data_dir = s.swarm.dir();
    save_debug_dir(data_dir, "init").await?;

    brain_salad_surgery(&s.swarm).await?;
    save_debug_dir(data_dir, "post_brain").await?;

    for node in s.swarm.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = false;
        update_node_config_restart(node, node_config)?;
    }
    Ok(())
}

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

async fn save_debug_dir(from: &Path, to: &str) -> Result<()> {
    // Get the current directory using CARGO_MANIFEST_DIR
    let current_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set")
        .into();
    let to_dir = current_dir.join(to);
    if to_dir.exists() {
        tokio::fs::remove_dir_all(&to_dir).await?;
    }
    tokio::fs::create_dir_all(&to).await?;
    fs_extra::dir::copy(
        from,
        &to_dir,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;
    Ok(())
}
async fn creates_random_val_account(data_path: &Path, port: u16) -> anyhow::Result<WalletLibrary> {
    let wallet = WalletLibrary::new();
    let mnemonic_string = wallet.mnemonic();

    let my_host = HostAndPort::local(port)?;

    // Initializes the validator configuration.
    validator_config::initialize_validator(
        Some(data_path.to_path_buf()),
        Some(&mnemonic_string.clone()[..3]),
        my_host,
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
        creates_random_val_account(&rando_dir.join(i.to_string()), local_node.port()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn meta_can_add_random_vals() -> anyhow::Result<()> {
    let s = test_setup_start_then_pause().await?;

    make_test_randos(&s).await?;

    save_debug_dir(s.swarm.dir(), "post_random_init").await?;

    Ok(())
}

#[tokio::test]
async fn can_create_replace_blob_from_randos() -> anyhow::Result<()> {
    let s = test_setup_start_then_pause().await?;

    make_test_randos(&s).await?;
    let data_dir = s.swarm.dir();
    save_debug_dir(data_dir, "post_random_init").await?;

    let val_db_path = s.swarm.validators().next().unwrap().config().storage.dir();

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

    save_debug_dir(data_dir, "post_blob").await?;

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
    save_debug_dir(data_dir, "post_random_init").await?;

    let val_db_path = s.swarm.validators().next().unwrap().config().storage.dir();

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

    save_debug_dir(data_dir, "post_blob").await?;

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
    assert!(wp == wp2, "waypoints don't match");

    save_debug_dir(data_dir, "post_rescue").await?;

    Ok(())
}

#[tokio::test]
async fn uses_written_db_from_rando_account() -> anyhow::Result<()> {
    let mut s = test_setup_start_then_pause().await?;
    let data_dir = &s.swarm.dir().to_path_buf();
    save_debug_dir(data_dir, "init").await?;

    brain_salad_surgery(&s.swarm).await?;
    save_debug_dir(data_dir, "post_brain").await?;

    // use glob to find the operator.yaml under the data_dir/
    let operator_yaml_pattern = format!("{}/**/operator.yaml", data_dir.display());
    let operator_yamls: Vec<PathBuf> = glob::glob(&operator_yaml_pattern)?
        .filter_map(Result::ok)
        .collect();

    if operator_yamls.is_empty() {
        anyhow::bail!("No operator.yaml files found in rando directory");
    }

    let val_db_path = s.swarm.validators().next().unwrap().config().storage.dir();
    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(data_dir.to_path_buf()), // save to top level test dir
        command: Sub::RegisterVals {
            operator_yaml: operator_yamls.clone(),
            upgrade_mrb: None,
        },
    };

    r.run()?;

    save_debug_dir(data_dir, "post_blob").await?;

    let genesis_blob_path = data_dir.join(REPLACE_VALIDATORS_BLOB);

    let mut wp: Option<Waypoint> = None;
    // apply the rescue to the dbs
    for n in s.swarm.validators() {
        wp = one_step_apply_rescue_on_db(&n.config().storage.dir(), &genesis_blob_path).ok();

        post_rescue_node_file_updates(&n.config_path(), wp.unwrap(), &genesis_blob_path)?;
    }

    save_debug_dir(data_dir, "post_rescue").await?;

    for node in s.swarm.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = false;
        update_node_config_restart(node, node_config)?;
    }

    s.swarm.wait_for_startup().await?;
    // tokio::time::sleep(Duration::from_secs(1)).await;

    // for n in s.swarm.validators_mut() {
    //     n.stop();
    //     post_rescue_node_file_updates(&n.config_path(), wp.unwrap(), &genesis_blob_path)?;

    //     n.start()?;
    // }

    tokio::time::sleep(Duration::from_secs(100)).await;

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
