use anyhow::Result;
use clap::Parser;
use diem_db_tool::restore;
use diem_logger::{Level, Logger};
use diem_push_metrics::MetricsPusher;
use std::path::PathBuf;

use crate::read_snapshot;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[allow(clippy::large_enum_variant)]

/// DB tools e.g.: backup, restore, export to json
pub enum StorageCli {
    #[clap(subcommand)]
    Restore(restore::Command),
    #[clap(subcommand)]
    Backup(backup::Command),
    ExportSnapshot {
        #[clap(short, long)]
        manifest_path: PathBuf,
        #[clap(short, long)]
        out_path: Option<PathBuf>,
    },
}

impl StorageCli {
    pub async fn run(&self) -> Result<()> {
        Logger::new().level(Level::Info).init();
        let _mp = MetricsPusher::start(vec![]);

        match StorageCli::parse() {
            StorageCli::Restore(cmd) => cmd.run().await?,
            StorageCli::Backup(cmd) => cmd.run().await?,
            StorageCli::ExportSnapshot {
                manifest_path,
                out_path,
            } => {
                read_snapshot::manifest_to_json(manifest_path, out_path).await;
            }
        }

        Ok(())
    }
}
