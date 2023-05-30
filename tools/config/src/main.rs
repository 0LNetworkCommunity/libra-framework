use anyhow::Result;
use clap::Parser;
use libra_config_cli::LibraConfigCli;

mod libra_config_cli;

#[tokio::main]
async fn main() -> Result<()> {
    LibraConfigCli::parse().run().await
}
