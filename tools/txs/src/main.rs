pub mod txs_core;

use anyhow::Result;
use clap::Parser;
use txs_core::TxsCli;

#[tokio::main]
async fn main() -> Result<()> {
    TxsCli::parse().run().await
}
