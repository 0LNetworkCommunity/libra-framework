use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_testnet::cli_main::TestnetCli::parse().run().await?;
    Ok(())
}
