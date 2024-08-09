use anyhow::Result;
use clap::{Parser, Subcommand};
use diem_db_tool::DBTool;
use diem_logger::{Level, Logger};
use diem_push_metrics::MetricsPusher;
use std::{fs, path::PathBuf};

use crate::{read_snapshot, restore, restore_bundle::RestoreBundle};

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
                let mut b = RestoreBundle::new(fs::canonicalize(bundle_path)?);

                b.load()?;

                restore::full_restore(&fs::canonicalize(destination_db)?, &b).await?;

                println!(
                    "SUCCESS: restored to epoch: {}, version: {}",
                    b.epoch, b.version
                );
            }
            _ => {} // prints help
        }

        Ok(())
    }
}
