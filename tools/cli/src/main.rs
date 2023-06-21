mod node_cli;

use clap::{Parser, Subcommand};
use libra_txs::txs_cli::TxsCli;
use libra_query::query_cli::QueryCli;
use libra_config::config_cli::ConfigCli;
use libra_wallet::wallet_cli::WalletCli;
use zapatos::move_tool::MoveTool;
use anyhow::anyhow;
use tokio;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct LibraCli {
    #[clap(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
  Node(node_cli::NodeCli),
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
        Some(Sub::Node(n)) => {
          n.run().await?;
        },
        Some(Sub::Move(move_tool)) => {
            move_tool.execute().await
            .map_err(|e| anyhow!("Failed to execute move tool, message: {}", e.to_string()))?;
        },
        Some(Sub::Txs(txs_cli)) => {
            txs_cli.run().await?;
        },
        Some(Sub::Query(query_cli)) => {
            query_cli.run().await?;
        },
        Some(Sub::Config(config_cli)) => {
            config_cli.run().await?;
        },
        Some(Sub::Wallet(wallet_cli)) => {
            wallet_cli.run().await?;
        },
        _ => { println!("\nliving is easy with eyes closed") }
    }


    Ok(())
}
