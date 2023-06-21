use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_config::config_cli::ConfigCli::parse().run().await
}
