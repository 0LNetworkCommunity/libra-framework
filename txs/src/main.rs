use anyhow::Result;
use clap::Parser;
use txs_cli::TxsCli;

mod txs_cli;

#[tokio::main]
async fn main() -> Result<()> {
    TxsCli::parse().run().await
}
