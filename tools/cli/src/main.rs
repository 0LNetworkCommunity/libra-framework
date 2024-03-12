mod node_cli;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use diem::move_tool::MoveTool;
use libra_config::config_cli::ConfigCli;
use libra_query::query_cli::QueryCli;
use libra_tower::tower_cli::TowerCli;
use libra_txs::txs_cli::TxsCli;
use libra_wallet::wallet_cli::WalletCli;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct LibraCli {
    #[clap(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    Config(ConfigCli),
    #[clap(subcommand)]
    Move(MoveTool), // from vendor
    Node(node_cli::NodeCli),
    Query(QueryCli),
    Tower(TowerCli),
    Txs(TxsCli),
    Wallet(WalletCli),
}

fn main() -> anyhow::Result<()> {
    let cli = LibraCli::parse();
    match cli.command {
        Some(Sub::Node(n)) => {
            n.run()?;
        }
        _ => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                match cli.command {
                    Some(Sub::Config(config_cli)) => {
                        if let Err(e) = config_cli.run().await {
                            eprintln!("Failed to execute config tool, message: {}", &e);
                        }
                    }
                    Some(Sub::Move(move_tool)) => {
                        if let Err(e) = move_tool
                            .execute()
                            .await
                            .map_err(|e| anyhow!("Failed to execute move tool, message: {}", &e))
                        {
                            eprintln!("Failed to execute move tool, message: {}", &e);
                        }
                    }
                    Some(Sub::Query(query_cli)) => {
                        if let Err(e) = query_cli.run().await {
                            eprintln!("Failed to execute query tool, message: {}", &e);
                        }
                    }
                    Some(Sub::Tower(tower_cli)) => {
                        if let Err(e) = tower_cli.run().await {
                            eprintln!("Failed to execute tower tool, message: {}", &e);
                        }
                    }
                    Some(Sub::Txs(txs_cli)) => {
                        if let Err(e) = txs_cli.run().await {
                            eprintln!("Failed to execute txs tool, message: {}", &e);
                        }
                    }
                    Some(Sub::Wallet(wallet_cli)) => {
                        if let Err(e) = wallet_cli.run().await {
                            eprintln!("Failed to execute wallet tool, message: {}", &e);
                        }
                    }
                    _ => {
                        println!("\nliving is easy with eyes closed")
                    }
                }
            });
        }
    }

    Ok(())
}
