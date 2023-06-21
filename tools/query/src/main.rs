use anyhow::Result;
use clap::Parser;
use libra_query::query_cli::QueryCli;


#[tokio::main]
async fn main() -> Result<()> {
    QueryCli::parse().run().await
}
