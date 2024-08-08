mod support;

use anyhow::Context;
use diem_forge::SwarmExt;
use diem_temppath::TempPath;
use diem_types::transaction::Transaction;
use libra_rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use libra_smoke_tests::libra_smoke::LibraSmoke;
use smoke_test::test_utils::MAX_CATCH_UP_WAIT_SECS;
use std::{fs, time::Duration};

use crate::support::{update_node_config_restart, wait_for_node};

// #[ignore]
#[tokio::test]
/// Tests we can create a genesis blob from the smoke test e2e environment.
/// NOTE: much of this is duplicated in rescue_cli_creates_blob and e2e but we
/// do want the granularity.
async fn test_create_e2e_rescue_tx() -> anyhow::Result<()> {
    let num_nodes: usize = 5;
    let mut s = LibraSmoke::new(Some(num_nodes as u8), None)
        .await
        .expect("could not start libra smoke");

    let env: &mut diem_forge::LocalSwarm = &mut s.swarm;

    env.wait_for_all_nodes_to_catchup_to_version(10, Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await?;

    println!("2. Set sync_only = true for all nodes and restart");
    for node in env.validators_mut() {
        let mut node_config = node.config().clone();
        node_config.consensus.sync_only = true;
        update_node_config_restart(node, node_config)?;
        wait_for_node(node, num_nodes - 1).await?;
    }

    println!("4. verify all nodes are at the same round and no progress being made");
    env.wait_for_all_nodes_to_catchup(Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
        .await?;

    println!("5. kill nodes");
    for node in env.validators_mut().take(3) {
        node.stop();
    }

    println!("6. prepare a genesis txn to remove the last validator");

    let first_validator_address = env
        .validators()
        .nth(4)
        .context("can't get nth val")?
        .config()
        .get_peer_id()
        .context("no peer ID")?;

    let script_path = support::make_script(first_validator_address);
    dbg!(&script_path);
    assert!(script_path.exists());

    let data_path = TempPath::new();
    data_path.create_as_dir()?;
    let rescue = RescueTxOpts {
        data_path: data_path.path().to_owned(),
        blob_path: None, // defaults to data_path/rescue.blob
        script_path: Some(script_path),
        framework_upgrade: false,
        debug_vals: None,
        testnet_vals: None,
    };
    let genesis_blob_path = rescue.run()?;

    assert!(genesis_blob_path.exists());

    let _genesis_transaction = {
        let buf = fs::read(&genesis_blob_path)?;
        bcs::from_bytes::<Transaction>(&buf)?
    };

    let val_db_path = env
        .validators()
        .next()
        .context("no val")?
        .config()
        .storage
        .dir();
    assert!(val_db_path.exists());

    // replace with rescue cli
    println!("6. prepare the waypoint with the transaction");
    let bootstrap = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: genesis_blob_path,
        waypoint_to_verify: None,
        commit: false, // NOTE: the tests seem to work even when this is false
        info: false,
    };

    let _waypoint = bootstrap.run()?;

    Ok(())
}
