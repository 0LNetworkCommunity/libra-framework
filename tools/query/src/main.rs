use anyhow::Result;
use clap::Parser;
use query_cli::QueryCli;

mod query_cli;

#[tokio::main]
async fn main() -> Result<()> {
    QueryCli::parse().run().await
}
