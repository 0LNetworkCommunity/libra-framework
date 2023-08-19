use crate::{
    query_type::{ QueryType, OutputType },
    utils::colorize_and_print,
};
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
        // let client = Client::default()?;
        // TODO: get client from configs

        let output = self.subcommand.query(None).await?;

        match output {
            OutputType::Json(json_str) => {
                colorize_and_print(&json_str)?;
            },
            OutputType::KeyValue(kv_str) => {
                println!("{}", kv_str);
            },
        }
        Ok(())
    }
}
