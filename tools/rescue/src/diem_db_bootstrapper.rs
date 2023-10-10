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

#[derive(Parser)]
#[clap(
    name = "diem-db-bootstrapper",
    about = "Calculate, verify and commit the genesis to local DB without a consensus among validators."
)]
pub struct BootstrapOpts {
    #[clap(value_parser)]
    pub db_dir: PathBuf,

    #[clap(short, long, value_parser)]
    pub genesis_txn_file: PathBuf,

    #[clap(short, long)]
    pub waypoint_to_verify: Option<Waypoint>,

    #[clap(long, requires("waypoint_to_verify"))]
    pub commit: bool,
}

impl BootstrapOpts {
    pub fn run(&self) -> Result<Waypoint> {
        let genesis_txn = load_genesis_txn(&self.genesis_txn_file)
            .with_context(|| format_err!("Failed loading genesis txn."))?;
        assert!(
            matches!(genesis_txn, Transaction::GenesisTransaction(_)),
            "Not a GenesisTransaction"
        );

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
        let db = DbReaderWriter::new(db);

        let executed_trees = db
            .reader
            .get_latest_executed_trees()
            .with_context(|| format_err!("Failed to get latest tree state."))?;
        println!("Db has {} transactions", executed_trees.num_transactions());
        if let Some(waypoint) = self.waypoint_to_verify {
            ensure!(
                waypoint.version() == executed_trees.num_transactions(),
                "Trying to generate waypoint at version {}, but DB has {} transactions.",
                waypoint.version(),
                executed_trees.num_transactions(),
            )
        }

        let committer = calculate_genesis::<DiemVM>(&db, executed_trees, &genesis_txn)
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
            println!("Waypoint verified.");
        }

        if self.commit {
            committer
                .commit()
                .with_context(|| format_err!("Committing genesis to DB."))?;
            println!("Successfully committed genesis.")
        }

        Ok(output_waypoint)
    }
}

fn load_genesis_txn(path: &Path) -> Result<Transaction> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    Ok(bcs::from_bytes(&buffer)?)
}


#[test]
fn test_bootstrap_db() -> anyhow::Result<()>{
  use std::path::Path;
  use diem_temppath;

  let blob_path = Path::new(env!("CARGO_MANIFEST_DIR"))
  .join("fixtures")
  .join("basic_genesis.blob");

  assert!(blob_path.exists());
  let db_root_path = diem_temppath::TempPath::new();
  db_root_path.create_as_dir()?;
  let db  = diem_db::DiemDB::new_for_test(db_root_path.path());
  // creates db and disconnects
  drop(db);


  let r = BootstrapOpts {
    db_dir: db_root_path.path().to_owned(),
    genesis_txn_file: blob_path,
    waypoint_to_verify: None,
    commit: true
  };

  r.run()?;
  assert!(db_root_path.path().exists());

  // cannot run a second time
  assert!(r.run().is_err());


  Ok(())

}