use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = libra_genesis_tools::cli::GenesisCli::parse();
    cli.run().await
}
