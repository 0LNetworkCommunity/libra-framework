use crate::query_type::QueryType;

use anyhow::Result;
use clap::Parser;
use libra_types::exports::Client;
use serde_json;
use url::Url;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
/// Query the blockchain for views and resources.
pub struct QueryCli {
    #[clap(subcommand)]
    /// what to query
    subcommand: QueryType,

    /// optional, URL of the upstream node to send tx to, including port
    /// Otherwise will default to what is in the config file
    #[clap(short, long)]
    pub url: Option<Url>,
}

impl QueryCli {
    pub async fn run(&self) -> Result<()> {
        // TODO: get client from configs

        let client_opt = if let Some(u) = &self.url {
            // Initialize client
            Some(Client::new(u.clone()))
        } else {
            None
        };

        let res = self.subcommand.query_to_json(client_opt).await?;
        let pretty_json = serde_json::to_string_pretty(&res)?;
        println!("{}", pretty_json);

        Ok(())
    }
}
