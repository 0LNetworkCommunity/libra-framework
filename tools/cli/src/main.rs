use std::path::PathBuf;
use clap::{Parser, Subcommand};
use libra_txs::txs_core::TxsCli;
use libra_query::query_cli::QueryCli;
use libra_config::libra_config_cli::ConfigCli;
use libra_wallet::wallet_cli::WalletCli;
use zapatos::move_tool::MoveTool;
use zapatos_config::config::NodeConfig;
use anyhow::anyhow;
use tokio;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct LibraCli {
    #[clap(subcommand)]
    command: Option<Sub>,

}

#[derive(Subcommand)]
enum Sub {
  Node { 
    #[clap(short,long)]
    config_path: PathBuf 
  },
  #[clap(subcommand)]
  Move(MoveTool), // from vendor
  Txs(TxsCli),
  Query(QueryCli),
  Config(ConfigCli),
  Wallet(WalletCli),


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
        Some(Sub::Move(move_tool)) => {
            let res = move_tool.execute().await
            .map_err(|e| anyhow!("Failed to execute move tool, message: {}", e.to_string()))?;
            dbg!(&res);
        },
        Some(Sub::Txs(txs_cli)) => {
            txs_cli.run().await?;
        },
        _ => { println!("\nliving is easy with eyes closed") }
    }


    Ok(())
}
