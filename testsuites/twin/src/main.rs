use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_twin_tests::twin_cli::TwinCli::parse().run().await?;
    Ok(())
}
