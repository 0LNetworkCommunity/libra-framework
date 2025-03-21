use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use diem_config::config::{RocksdbConfigs, NO_OP_STORAGE_PRUNER_CONFIG};
use diem_db::DiemDB;
use diem_db_tool::DBTool;
use diem_executor::db_bootstrapper::maybe_bootstrap;
use diem_logger::{Level, Logger};
use diem_push_metrics::MetricsPusher;
use diem_storage_interface::DbReaderWriter;
use diem_types::{transaction::Transaction, waypoint::Waypoint};
use diem_vm::DiemVM;
use std::str::FromStr;
use std::{fs, path::PathBuf};

// Import the correct functions from libra-config
use libra_config::make_yaml_public_fullnode::{download_genesis, get_genesis_waypoint};

use crate::{download_bundle, read_snapshot, restore, restore_bundle::RestoreBundle};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
/// DB tools e.g.: backup, restore, export to json
pub struct StorageCli {
    #[clap(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Sub {
    #[clap(subcommand)]
    /// DB tools for backup, restore, verify, etc.
    Db(DBTool),
    /// simple restore from a bundle for one epoch
    EpochRestore {
        #[clap(short, long)]
        bundle_path: PathBuf,
        #[clap(short, long)]
        destination_db: PathBuf,
    },
    /// downloads the stat, epoch, and transaction
    /// restore files from the `epoch-archive` repo
    DownloadRestoreBundle {
        #[clap(long, default_value = "0LNetworkCommunity")]
        owner: String,
        #[clap(long, default_value = "epoch-archive-mainnet")]
        repo: String,
        #[clap(long, default_value = "v7.0.0")]
        branch: String,
        #[clap(short, long)]
        epoch: String,
        #[clap(short, long)]
        destination: PathBuf,
    },
    /// Read a snapshot, parse and export to JSON
    ExportSnapshot {
        #[clap(short, long)]
        manifest_path: PathBuf,
        #[clap(short, long)]
        out_path: Option<PathBuf>,
    },
    /// Bootstrap a restored DB with genesis and waypoint
    Bootstrap {
        /// Path to the DB to bootstrap
        #[clap(long)]
        db_path: PathBuf,

        /// Optional custom genesis path (defaults to downloading mainnet genesis)
        #[clap(long)]
        genesis_path: Option<PathBuf>,

        /// Optional waypoint (defaults to downloading mainnet genesis waypoint)
        #[clap(long)]
        waypoint: Option<String>,
    },
}

impl StorageCli {
    // Note: using owned self since DBTool::run uses an owned self.
    pub async fn run(self) -> Result<()> {
        Logger::new().level(Level::Info).init();
        let _mp = MetricsPusher::start(vec![]);

        match self.command {
            Some(Sub::Db(tool)) => {
                tool.run().await?;
            }
            Some(Sub::ExportSnapshot {
                manifest_path,
                out_path,
            }) => {
                read_snapshot::manifest_to_json(manifest_path.to_owned(), out_path.to_owned())
                    .await;
            }
            Some(Sub::EpochRestore {
                bundle_path,
                destination_db,
            }) => {
                if !bundle_path.exists() {
                    bail!("bundle directory not found: {}", &bundle_path.display());
                };
                if destination_db.exists() {
                    bail!("you are trying to restore to a directory that already exists, and may have conflicting state: {}", &destination_db.display());
                };

                fs::create_dir_all(&destination_db)?;

                // underlying tools get lost with relative paths
                let bundle_path =
                    fs::canonicalize(bundle_path).context("Failed to canonicalize bundle path")?;
                let destination_db = fs::canonicalize(destination_db)
                    .context("Failed to canonicalize destination path")?;

                // Decompress all .gz files in the bundle directory
                restore::maybe_decompress_gz_files(&bundle_path)
                    .await
                    .context("Failed to decompress gz files")?;

                let mut bundle = RestoreBundle::new(bundle_path);

                bundle.load()?;

                restore::full_restore(&destination_db, &bundle).await?;

                println!(
                    "SUCCESS: restored to epoch: {}, version: {}",
                    bundle.epoch, bundle.version
                );
            }
            Some(Sub::DownloadRestoreBundle {
                owner,
                repo,
                branch,
                epoch,
                destination,
            }) => {
                download_bundle::download_restore_bundle(
                    &owner,
                    &repo,
                    &branch,
                    &epoch,
                    &destination,
                )
                .await?;
            }
            Some(Sub::Bootstrap {
                db_path,
                genesis_path,
                waypoint,
            }) => {
                if !db_path.exists() {
                    bail!("DB path does not exist: {}", db_path.display());
                }

                println!("Bootstrapping DB at {}", db_path.display());

                // Create or use data path for downloaded genesis files
                let data_path = std::env::temp_dir().join("libra_bootstrap");
                if !data_path.exists() {
                    println!("Creating config directory: {}", data_path.display());
                    fs::create_dir_all(&data_path)?;
                }

                // Get genesis blob - either from provided path or download
                let genesis_blob = match genesis_path {
                    Some(path) => {
                        println!("Using custom genesis from: {}", path.display());
                        fs::read(&path)
                            .context(format!("Failed to read genesis from {}", path.display()))?
                    }
                    None => {
                        println!("Downloading genesis blob...");
                        // Download genesis from GitHub
                        download_genesis(Some(data_path.clone())).await?;
                        fs::read(data_path.join("genesis.blob"))
                            .context("Failed to read downloaded genesis blob")?
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
                let genesis_txn: Transaction = bcs::from_bytes(&genesis_blob)
                    .context("Failed to deserialize genesis transaction")?;

                // Create DB instance with correct parameters
                println!("Opening database...");
                let db = DiemDB::open(
                    &db_path,
                    false,                       /* readonly */
                    NO_OP_STORAGE_PRUNER_CONFIG, /* pruner_config */
                    RocksdbConfigs::default(),   /* rocksdb_configs */
                    false,                       /* enable_indexer */
                    1000, /* buffered_state_target_items - using a reasonable default */
                    1000, /* max_num_nodes_per_lru_cache_shard - using a reasonable default */
                )
                .context("Failed to open database")?;

                let db_rw = DbReaderWriter::new(db);

                // Bootstrap the DB
                println!("Bootstrapping database with genesis transaction...");
                maybe_bootstrap::<DiemVM>(&db_rw, &genesis_txn, waypoint)
                    .context("Failed to bootstrap database")?;

                println!("DB bootstrap completed successfully");
            }
            _ => {} // prints help
        }

        Ok(())
    }
}
