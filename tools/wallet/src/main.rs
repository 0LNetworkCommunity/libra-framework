use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_wallet::wallet_cli::WalletCli::parse().run().await
}
