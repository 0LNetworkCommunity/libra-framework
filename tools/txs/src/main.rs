use libra_txs::txs_cli::TxsCli;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    TxsCli::parse().run().await
}
