use anyhow::anyhow;
use clap::Parser;
use diem_config::config::NodeConfig;
use libra_types::global_config_dir;
use std::path::PathBuf;

#[derive(Parser)]
/// Start a libra node
pub struct NodeCli {
    #[clap(short, long)]
    /// filepath to the validator or fullnode yaml config file.
    config_path: Option<PathBuf>,
}

impl NodeCli {
    pub async fn run(&self) -> anyhow::Result<()> {
        let path = self
            .config_path
            .clone()
            .unwrap_or_else(|| global_config_dir().join("validator.yaml"));
        // A config file exists, attempt to parse the config
        let config = NodeConfig::load_from_path(path.clone()).map_err(|error| {
            anyhow!(
                "Failed to load the node config file! Given file path: {:?}. Error: {:?}",
                path.display(),
                error
            )
        })?;

        // Start the node
        diem_node::start(config, None, true).expect("Node should start correctly");

        Ok(())
    }
}
