use anyhow::Result;
use clap::Parser;
use diem_db_tool::DBTool;
use diem_logger::{Level, Logger};
use diem_push_metrics::MetricsPusher;
use std::path::PathBuf;
use storage::read_snapshot::manifest_to_json;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[allow(clippy::large_enum_variant)]
enum StorageCli {
    #[clap(subcommand)]
    Db(DBTool),
    ExportSnapshot {
        #[clap(short, long)]
        manifest_path: PathBuf,
        #[clap(short, long)]
        out_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    Logger::new().level(Level::Info).init();
    let _mp = MetricsPusher::start(vec![]);

    match StorageCli::parse() {
        StorageCli::Db(tool) => {
            tool.run().await?;
        }
        StorageCli::ExportSnapshot {
            manifest_path,
            out_path,
        } => {
            manifest_to_json(manifest_path, out_path).await;
        }
    }

    DBTool::parse().run().await?;

    Ok(())
}
