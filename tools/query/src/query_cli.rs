use crate::query_type::QueryType;
use anyhow::Result;
use clap::Parser;

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
        // TODO: get client from configs

        let res = self.subcommand.query_to_json(None).await?;
        match res {
            serde_json::Value::String(a) => println!("{:#}", a.as_str()),
            _ => println!("{:#}", res),
        }

        Ok(())
    }
}
