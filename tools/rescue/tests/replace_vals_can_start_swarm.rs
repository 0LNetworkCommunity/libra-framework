use anyhow::Result;
use diem_config::config::{NodeConfig, PersistableConfig};
use diem_forge::NodeExt;
use libra_rescue::cli_bootstrapper::one_step_apply_rescue_on_db;
use libra_rescue::cli_main::REPLACE_VALIDATORS_BLOB;
use libra_rescue::node_config::post_rescue_node_file_updates;
use libra_rescue::test_support::update_node_config_restart;
use libra_rescue::{
    cli_bootstrapper::BootstrapOpts,
    cli_main::{RescueCli, Sub},
};
use libra_smoke_tests::brain_salad_surgery::brain_salad_surgery;
use libra_smoke_tests::helpers::{is_making_progress, make_test_randos};
use libra_smoke_tests::libra_smoke::LibraSmoke;
use std::path::{Path, PathBuf};


#[tokio::test]
async fn can_create_replace_blob_from_randos() -> anyhow::Result<()> {
    let s = LibraSmoke::test_setup_start_then_pause(2).await?;

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
    let s = LibraSmoke::test_setup_start_then_pause(2).await?;

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
    let mut s = LibraSmoke::test_setup_start_then_pause(2).await?;

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

    // apply the rescue to the dbs
    for n in s.swarm.validators() {
        let wp = one_step_apply_rescue_on_db(&n.config().storage.dir(), &genesis_blob_path)?;
        post_rescue_node_file_updates(&n.config_path(), wp, &genesis_blob_path)?;
    }

    save_debug_dir(data_dir, "post_rescue").await?;

    for node in s.swarm.validators_mut() {
        // Don't load the instantiated config! Get the saved one always!
        let mut node_config = NodeConfig::load_config(node.config_path())?;
        node_config.consensus.sync_only = false;
        update_node_config_restart(node, node_config)?;
    }

    s.swarm.wait_for_startup().await?;

    let client = s.swarm.validators().next().unwrap().rest_client();
    assert!(
        is_making_progress(&client).await?,
        "chain not making blocks"
    );

    Ok(())
}
