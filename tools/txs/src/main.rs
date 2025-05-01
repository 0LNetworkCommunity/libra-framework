use anyhow::Result;
use clap::Parser;
use diem_logger::{Level, Logger};
use libra_txs::txs_cli::TxsCli;

#[tokio::main]
async fn main() -> Result<()> {
    Logger::new().level(Level::Warn).init();
    TxsCli::parse().run().await
}
