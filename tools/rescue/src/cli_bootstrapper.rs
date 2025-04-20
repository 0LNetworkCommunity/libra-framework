// Copyright © Diem Contributors
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

// refer to vendor: /root/diem/execution/db-bootstrapper/src/bin/diem-db-bootstrapper.rs

use anyhow::{ensure, format_err, Context, Result};
use clap::Parser;
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_executor::db_bootstrapper::calculate_genesis;
use diem_storage_interface::DbReaderWriter;
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use diem_vm::DiemVM;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::node_config;

#[derive(Parser)]
#[clap(
    name = "diem-db-bootstrapper",
    about = "Calculate, verify and commit the genesis to local DB without a consensus among validators."
)]
pub struct BootstrapOpts {
    #[clap(short, long)]
    /// DB directory
    pub db_dir: PathBuf,

    #[clap(short, long)]
    /// path to genesis tx file
    pub genesis_txn_file: PathBuf,

    #[clap(short, long)]
    /// waypoint expected
    pub waypoint_to_verify: Option<Waypoint>,

    #[clap(long, requires("waypoint_to_verify"))]
    /// commit to db (requires --waypoint-to-verify)
    pub commit: bool,

    #[clap(long, requires = "commit")]
    /// Update the node configs in a single step
    pub update_node_config: Option<PathBuf>,

    #[clap(long)]
    /// get info on DB and exit
    pub info: bool,
}

impl BootstrapOpts {
    pub fn run(&self) -> Result<Option<Waypoint>> {
        let genesis_txn = load_genesis_txn(&self.genesis_txn_file)
            .with_context(|| format_err!("Failed loading genesis txn."))?;
        assert!(
            matches!(genesis_txn, Transaction::GenesisTransaction(_)),
            "Not a GenesisTransaction"
        );

        println!("Reading DB ...");

        // Opening the DB exclusively, it's not allowed to run this tool alongside a running node which
        // operates on the same DB.
        let db = DiemDB::open(
            &self.db_dir,
            false,
            NO_OP_STORAGE_PRUNER_CONFIG, /* pruner */
            RocksdbConfigs::default(),
            false, /* indexer */
            BUFFERED_STATE_TARGET_ITEMS,
            DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
        )
        .expect("Failed to open DB.");

        let db_rw = DbReaderWriter::new(db);

        let executed_trees = db_rw
            .reader
            .get_latest_executed_trees()
            .with_context(|| format_err!("Failed to get latest tree state."))?;

        println!("num txs: {:?}", executed_trees.num_transactions());
        println!("version: {:?}", executed_trees.version());
        println!(
            "root hash: {:?}",
            executed_trees.txn_accumulator().root_hash
        );

        if self.info {
            return Ok(None);
        }

        if self.info {
            println!("num txs: {:?}", executed_trees.num_transactions());
            println!("version: {:?}", executed_trees.version());
            println!(
                "root hash: {:?}",
                executed_trees.txn_accumulator().root_hash
            );

            return Ok(None);
        }

        if let Some(waypoint) = self.waypoint_to_verify {
            ensure!(
                waypoint.version() == executed_trees.num_transactions(),
                "Trying to generate waypoint at version {}, but DB has {} transactions.",
                waypoint.version(),
                executed_trees.num_transactions(),
            )
        }

        let committer = calculate_genesis::<DiemVM>(&db_rw, executed_trees, &genesis_txn)
            .with_context(|| format_err!("Failed to calculate genesis."))?;

        println!(
            "Successfully calculated genesis. Got waypoint: {}",
            committer.waypoint()
        );

        let output_waypoint = committer.waypoint();

        if let Some(waypoint) = self.waypoint_to_verify {
            ensure!(
                waypoint == output_waypoint,
                "Waypoint verification failed. Expected {:?}, got {:?}.",
                waypoint,
                output_waypoint,
            );
            println!("waypoint verified");
        }

        if self.commit {
            committer
                .commit()
                .with_context(|| format_err!("Committing genesis to DB."))?;
            println!("Successfully committed genesis.");

            if let Some(p) = &self.update_node_config {
                node_config::post_rescue_node_file_updates(
                    p,
                    output_waypoint,
                    &self.genesis_txn_file,
                )?;
            }
        }

        Ok(Some(output_waypoint))
    }
}

fn load_genesis_txn(path: &Path) -> Result<Transaction> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    Ok(bcs::from_bytes(&buffer)?)
}

#[ignore] // TODO
#[test]
fn test_bootstrap_db() -> anyhow::Result<()> {
    use diem_temppath;
    use std::path::Path;

    let blob_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("basic_genesis.blob");

    assert!(blob_path.exists());
    let db_root_path = diem_temppath::TempPath::new();
    db_root_path.create_as_dir()?;
    let db = diem_db::DiemDB::new_for_test(db_root_path.path());
    // creates db and disconnects
    drop(db);

    let r = BootstrapOpts {
        db_dir: db_root_path.path().to_owned(),
        genesis_txn_file: blob_path,
        waypoint_to_verify: None,
        commit: true,
        update_node_config: None,
        info: false,
    };

    r.run()?;
    assert!(db_root_path.path().exists());

    // cannot run a second time
    assert!(r.run().is_err());

    Ok(())
}

pub fn check_rescue_bootstraps(db_path: &Path, blob_path: &Path) -> Result<()> {
    let b = BootstrapOpts {
        db_dir: db_path.to_owned(),
        genesis_txn_file: blob_path.to_owned(),
        waypoint_to_verify: None,
        commit: false,
        update_node_config: None,

        info: false,
    };
    if let Some(wp) = b.run()? {
        println!("Rescue tx verified. Bootstrap with waypoint: {:?}", wp);
    }
    Ok(())
}

/// helper to apply a single step upgrade to a db
/// usually for internal testing purposes (e.g. twin)
pub fn one_step_apply_rescue_on_db(
    db_to_change_path: &Path,
    rescue_blob: &Path,
) -> anyhow::Result<Waypoint> {
    let mut waypoint: Option<Waypoint> = None;
    let bootstrap = BootstrapOpts {
        db_dir: db_to_change_path.to_owned(),
        genesis_txn_file: rescue_blob.to_owned(),
        waypoint_to_verify: None,
        commit: false, // NOT APPLYING THE TX
        update_node_config: None,
        info: false,
    };

    let waypoint_to_check = bootstrap.run()?.expect("could not get waypoint");

    // give time for any IO to finish
    std::thread::sleep(std::time::Duration::from_secs(1));

    let bootstrap = BootstrapOpts {
        db_dir: db_to_change_path.to_owned(),
        genesis_txn_file: rescue_blob.to_owned(),
        waypoint_to_verify: Some(waypoint_to_check),
        commit: true, // APPLY THE TX
        update_node_config: None,
        info: false,
    };

    let waypoint_post = bootstrap.run()?.expect("could not get waypoint");
    assert!(
        waypoint_to_check == waypoint_post,
        "waypoints are not equal"
    );
    if let Some(w) = waypoint {
        assert!(
            waypoint_to_check == w,
            "waypoints are not equal between nodes"
        );
    }
    waypoint = Some(waypoint_to_check);
    // }
    match waypoint {
        Some(w) => Ok(w),
        None => anyhow::bail!("cannot generate consistent waypoint."),
    }
}
