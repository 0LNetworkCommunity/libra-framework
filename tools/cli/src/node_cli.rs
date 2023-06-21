
use clap::Parser;
use std::path::PathBuf;
use zapatos_config::config::NodeConfig;
use anyhow::anyhow;

#[derive(Parser)]
/// Start a libra node
pub struct NodeCli { 
  #[clap(short,long)]
  /// filepath to the validator or fullnode yaml config file.
  config_path: PathBuf 
}

impl NodeCli {
  pub async fn run(&self) -> anyhow::Result<()> {
    // A config file exists, attempt to parse the config
    let config = NodeConfig::load_from_path(self.config_path.clone()).
    map_err(|error| {
        anyhow!(
            "Failed to load the node config file! Given file path: {:?}. Error: {:?}",
            self.config_path.display(),
            error
        )
    })?;

    // Start the node
    zapatos_node::start(config, None, true).expect("Node should start correctly");
    
    Ok(())
  }

}