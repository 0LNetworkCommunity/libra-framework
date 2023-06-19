use std::path::PathBuf;
use clap::{Parser, Subcommand};
use zapatos_config::config::NodeConfig;
use anyhow::anyhow;
use zapatos::move_tool;
use tokio;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct LibraCli {
    #[command(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
  Node { 
    #[clap(short,long)]
    config_path: PathBuf 
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let cli = LibraCli::parse();
    match cli.command {
        Some(Sub::Node { config_path }) => {
                      // A config file exists, attempt to parse the config
            let config = NodeConfig::load_from_path(config_path.clone()).
            map_err(|error| {
                anyhow!(
                    "Failed to load the node config file! Given file path: {:?}. Error: {:?}",
                    config_path.display(),
                    error
                )
            })?;

            // Start the node
            zapatos_node::start(config, None, true).expect("Node should start correctly");

        },
        Some(tool) => tool.exectute().await,

        _ => { println!("\nliving is easy with eyes closed") }
    }

    // Continued program logic goes here...
    Ok(())
}
