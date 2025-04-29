use anyhow::Result;
use clap::{Parser, Subcommand};
use diem_db_tool::DBTool;
use diem_logger::{Level, Logger};
use diem_push_metrics::MetricsPusher;
use std::path::PathBuf;

// Import the correct functions from libra-config

use crate::{bootstrap, download_bundle, read_snapshot, restore};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
/// DB tools e.g.: backup, restore, export to json
pub struct StorageCli {
    #[clap(subcommand)]
    pub command: Option<Sub>,
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
        /// path of the restore bundle with epoch_ending, state_epoch, and transaction archives
        bundle_path: PathBuf,
        #[clap(short, long)]
        /// destination db path to restore to
        destination_db: PathBuf,
        #[clap(short, long)]
        /// prevent bootstrap after restore, advanced
        prevent_bootstrap: bool,
    },
    /// downloads the stat, epoch, and transaction
    /// restore files from the `epoch-archive` repo
    DownloadRestoreBundle {
        #[clap(long, default_value = "0LNetworkCommunity")]
        /// github organization or user, canonically: "0LNetworkCommunity"
        owner: String,
        #[clap(long, default_value = "epoch-archive-mainnet")]
        /// repo of archive, canonically: "epoch-archive-mainnet"
        repo: String,
        #[clap(long, default_value = "v7.0.0")]
        /// branch of the archive, canonically: "v7.0.0"
        branch: String,
        #[clap(short, long)]
        /// required, the number of epoch to restore
        epoch: u64,
        #[clap(short, long)]
        /// required, the directory to download the restore bundle to
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

        /// optional, home path for genesis files (defaults to global config dir)
        #[clap(long)]
        home_path: Option<PathBuf>,

        /// Optional custom genesis path (defaults to downloading mainnet genesis)
        #[clap(long)]
        genesis_path: Option<PathBuf>,

        /// Optional waypoint (defaults to downloading mainnet genesis waypoint)
        #[clap(long)]
        waypoint: Option<String>,
    },
}

impl StorageCli {
    pub async fn run(self) -> Result<()> {
        Logger::new().level(Level::Warn).init();
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
                prevent_bootstrap,
            }) => {
                restore::epoch_restore(bundle_path, destination_db.clone()).await?;
                // by default we want to bootstrap a restored db
                // if you have an advanced case you can run with --prevent_bootstrap=true
                // and later run the bootstrap command standalone
                if !prevent_bootstrap {
                    bootstrap::bootstrap_db(destination_db, None, None, None).await?;
                }
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
                home_path,
                genesis_path,
                waypoint,
            }) => {
                bootstrap::bootstrap_db(db_path, home_path, genesis_path, waypoint).await?;
            }
            _ => {} // prints help
        }

        Ok(())
    }
}
