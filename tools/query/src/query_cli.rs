use std::path::PathBuf;

use crate::query_type::QueryType;

use anyhow::Result;
use clap::Parser;
use libra_types::{
    core_types::app_cfg::AppCfg, exports::Client, type_extensions::client_ext::ClientExt,
};
use serde_json;
use url::Url;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
/// Query the blockchain for views and resources.
pub struct QueryCli {
    #[clap(subcommand)]
    /// what to query
    subcommand: QueryType,

    /// optional, path to the libra cli config file
    #[clap(short, long)]
    pub config_path: Option<PathBuf>,

    /// optional, URL of the upstream node to send tx to, including port
    /// Otherwise will default to what is in the config file
    #[clap(short, long)]
    pub url: Option<Url>,
}

impl QueryCli {
    pub async fn run(&self) -> Result<()> {
        // Query requires a URL for upstream
        // the user should set one explicitly
        // Otherwise the tool will try to fetch the libra config from the
        // usual location: ~/.libra
        // The user can set an alternative path the the config,
        // which is useful in testnets.

        // Initialize client
        let client = if let Some(u) = &self.url {
            Client::new(u.clone())
        } else if let Some(p) = &self.config_path {
            let app_cfg = AppCfg::load(Some(p.to_owned()))?;
            let (c, _) = Client::from_libra_config(&app_cfg, None).await?;
            c
        } else {
            Client::default().await?
        };

        let res = self.subcommand.query_to_json(&client).await?;
        let pretty_json = serde_json::to_string_pretty(&res)?;
        println!("{}", pretty_json);

        Ok(())
    }
}
