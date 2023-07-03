use anyhow::Result;
use clap::Parser;
use libra_tower::tower_cli::TowerCli;

#[tokio::main]
async fn main() -> Result<()> {
    TowerCli::parse().run().await
}
