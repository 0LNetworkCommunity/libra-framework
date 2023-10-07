use anyhow::{ensure, format_err, Context, Result};
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_executor::db_bootstrapper::calculate_genesis;
use diem_storage_interface::DbReaderWriter;
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use diem_vm::DiemVM;
use clap::Parser;
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
struct Opt {
    #[clap(value_parser)]
    db_dir: PathBuf,

    #[clap(short, long, value_parser)]
    genesis_txn_file: PathBuf,

    #[clap(short, long)]
    waypoint_to_verify: Option<Waypoint>,

    #[clap(long, requires("waypoint_to_verify"))]
    commit: bool,
}

pub fn bootstrap(opt: &Opt) -> Result<()> {
    let genesis_txn = load_genesis_txn(&opt.genesis_txn_file)
        .with_context(|| format_err!("Failed loading genesis txn."))?;
    assert!(
        matches!(genesis_txn, Transaction::GenesisTransaction(_)),
        "Not a GenesisTransaction"
    );

    // Opening the DB exclusively, it's not allowed to run this tool alongside a running node which
    // operates on the same DB.
    let db = DiemDB::open(
        &opt.db_dir,
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
    if let Some(waypoint) = opt.waypoint_to_verify {
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

    if let Some(waypoint) = opt.waypoint_to_verify {
        ensure!(
            waypoint == committer.waypoint(),
            "Waypoint verification failed. Expected {:?}, got {:?}.",
            waypoint,
            committer.waypoint(),
        );
        println!("Waypoint verified.");

        if opt.commit {
            committer
                .commit()
                .with_context(|| format_err!("Committing genesis to DB."))?;
            println!("Successfully committed genesis.")
        }
    }

    Ok(())
}

fn load_genesis_txn(path: &Path) -> Result<Transaction> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    Ok(bcs::from_bytes(&buffer)?)
}
