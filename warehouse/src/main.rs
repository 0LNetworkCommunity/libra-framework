//! entry point
use clap::Parser;
use libra_warehouse::{log_setup, warehouse_cli::WarehouseCli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log_setup();
    WarehouseCli::parse().run().await?;
    Ok(())
}
