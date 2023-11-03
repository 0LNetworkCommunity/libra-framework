use anyhow::{anyhow, bail};
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
        // validators typically aren't looking for verbose logs.
        // but they can set it if they wish with RUST_LOG=info
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "warn")
        }

        let path = self
            .config_path
            .clone()
            .unwrap_or_else(|| find_a_config().expect("no config"));
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

/// helper to find a default config
fn find_a_config() -> anyhow::Result<PathBuf> {
    let d = global_config_dir();
    let val_file = d.join("validator.yaml");

    let help = "If this is not what you expected explicitly set it with --config-file <path>";

    // we assume if this is set up as a validator that's the preferred profile
    if val_file.exists() {
        println!(
            "\nUsing validator profile at {}.\n{}",
            val_file.display(),
            help
        );
        return Ok(val_file);
    }

    let fn_file = d.join("fullnode.yaml");
    if fn_file.exists() {
        println!(
            "\nUsing fullnode profile at {}.\n{}",
            fn_file.display(),
            help
        );
        return Ok(fn_file);
    }

    bail!("ERROR: you have no node *.yaml configured in the default directory $HOME/.libra/");
}
