use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_testnet::testnet_cli::TestnetCli::parse()
        .run()
        .await?;
    Ok(())
}
