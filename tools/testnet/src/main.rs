use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_testnet::twin_cli::TwinCli::parse().run().await?;
    Ok(())
}
