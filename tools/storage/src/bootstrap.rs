use anyhow::{bail, Context, Result};
use diem_config::config::{RocksdbConfigs, NO_OP_STORAGE_PRUNER_CONFIG};
use diem_db::DiemDB;
use diem_executor::db_bootstrapper::maybe_bootstrap;
use diem_storage_interface::DbReaderWriter;
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use diem_vm::DiemVM;
use libra_types::global_config_dir;
use std::{fs, path::PathBuf, str::FromStr};

// Import functions from libra-config
use libra_config::get_genesis_artifacts::{download_genesis, get_genesis_waypoint};

/// Bootstrap a restored database with genesis transaction and waypoint
pub async fn bootstrap_db(
    db_path: PathBuf,
    home_path: Option<PathBuf>,
    genesis_path: Option<PathBuf>,
    waypoint: Option<String>,
) -> Result<()> {
    if !db_path.exists() {
        bail!("DB path does not exist: {}", db_path.display());
    }
    println!("Bootstrapping DB at {}", db_path.display());

    let data_path = home_path.unwrap_or_else(global_config_dir);
    println!(
        "Storing genesis artifacts at home path: {}",
        data_path.display()
    );

    // Create or use data path for downloaded genesis files

    assert!(
        data_path.exists(),
        "home path provided does not exist: {}",
        data_path.display()
    );

    // Get genesis blob - either from provided path or download
    let genesis_blob = match genesis_path {
        Some(path) => {
            println!("Using custom genesis from: {}", path.display());
            fs::read(&path).context(format!("Failed to read genesis from {}", path.display()))?
        }
        None => {
            println!("Downloading genesis blob...");
            // Download genesis from GitHub
            let p = download_genesis(Some(data_path.clone())).await?;
            fs::read(&p).context("Failed to read downloaded genesis blob")?
        }
    };

    // Get waypoint - either from provided string or download
    let waypoint = match waypoint {
        Some(wp_str) => {
            println!("Using custom waypoint: {}", wp_str);
            Waypoint::from_str(&wp_str).context("Failed to parse waypoint")?
        }
        None => {
            println!("Getting genesis waypoint...");
            // Download waypoint from GitHub
            get_genesis_waypoint(Some(data_path.clone())).await?
        }
    };

    println!("Using waypoint: {}", waypoint);

    // Parse genesis transaction
    let genesis_txn: Transaction =
        bcs::from_bytes(&genesis_blob).context("Failed to deserialize genesis transaction")?;

    // Create DB instance with correct parameters
    println!("Opening database...");
    let db = DiemDB::open(
        &db_path,
        false,                       /* readonly */
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner_config */
        RocksdbConfigs::default(),   /* rocksdb_configs */
        false,                       /* enable_indexer */
        1000,                        /* buffered_state_target_items */
        1000,                        /* max_num_nodes_per_lru_cache_shard */
    )
    .context("Failed to open database")?;

    let db_rw = DbReaderWriter::new(db);

    // Bootstrap the DB
    println!("Bootstrapping database with genesis transaction...");
    maybe_bootstrap::<DiemVM>(&db_rw, &genesis_txn, waypoint)
        .context("Failed to bootstrap database")?;

    println!("DB bootstrap completed successfully");
    Ok(())
}
