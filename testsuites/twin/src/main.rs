use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    libra_twin_tests::runner::Twin::parse().run().await?;
    Ok(())
}
