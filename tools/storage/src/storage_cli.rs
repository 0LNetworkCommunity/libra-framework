use anyhow::{bail, Result, Context};  // Add Context import
use clap::{Parser, Subcommand};
use diem_db_tool::DBTool;
use diem_logger::{Level, Logger};
use diem_push_metrics::MetricsPusher;
use std::{fs, path::PathBuf};

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
                let bundle_path = fs::canonicalize(bundle_path)
                    .context("Failed to canonicalize bundle path")?;
                let destination_db = fs::canonicalize(destination_db)
                    .context("Failed to canonicalize destination path")?;

                // Decompress all .gz files in the bundle directory
                restore::decompress_gz_files(&bundle_path)
                    .await
                    .context("Failed to decompress gz files")?;
                println!("Decompression completed, starting restore sequence");

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
            _ => {} // prints help
        }

        Ok(())
    }
}
