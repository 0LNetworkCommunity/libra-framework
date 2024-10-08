//! entry point
use clap::Parser;
use libra_warehouse::warehouse_cli::WarehouseCli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    WarehouseCli::parse().run();
    Ok(())
}
