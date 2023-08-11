use anyhow::Result;
use clap::Parser;
use crate::query_type::QueryType;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]

/// Query the blockchain for views and resources.
pub struct QueryCli {
    #[clap(subcommand)]
    /// what to query
    subcommand: QueryType,
}

impl QueryCli {
    pub async fn run(&self) -> Result<()> {
        // let client = Client::default()?;
        // TODO: get client from configs

        let res = self.subcommand.query_to_json(None).await?;

        println!("{}", res.to_string());

        Ok(())
    }
}
