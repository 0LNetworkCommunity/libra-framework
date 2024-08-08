//! entry point
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_storage::storage_cli::StorageCli::parse().run().await
}
