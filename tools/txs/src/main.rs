use anyhow::Result;
use clap::Parser;
use libra_txs::txs_cli::TxsCli;

#[tokio::main]
async fn main() -> Result<()> {
    TxsCli::parse().run().await
}
