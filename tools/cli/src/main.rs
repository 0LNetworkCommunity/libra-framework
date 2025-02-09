mod move_cli;
mod node_cli;
mod ops_cli;

use std::{path::PathBuf, process::exit};

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use libra_config::config_cli::ConfigCli;
use libra_query::query_cli::QueryCli;
use libra_txs::txs_cli::TxsCli;
use libra_wallet::wallet_cli::WalletCli;
use node_cli::NodeCli;

#[derive(Parser)]
#[clap(author, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct LibraCli {
    #[clap(subcommand)]
    command: Option<Sub>,
    // Note: this is for compatibility with how Diem Force calls diem-node
    // TODO: change this upstream.
    #[clap(hide(true), long, short = 'f')]
    swarm_config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Sub {
    Config(ConfigCli),
    #[clap(subcommand)]
    Move(move_cli::MoveTool),
    Node(node_cli::NodeCli),
    Query(QueryCli),
    Txs(TxsCli),
    Wallet(WalletCli),
    #[clap(subcommand)]
    Ops(ops_cli::OpsTool),
    /// I just checked in to see what condition my condition was in
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = LibraCli::parse();

    // Hack. Diem forge API compatibility.
    if let Some(p) = cli.swarm_config {
        let n = NodeCli {
            config_path: Some(p),
        };
        n.run()?;
        exit(0);
    }

    match cli.command {
        // Execute Node CLI subcommand
        Some(Sub::Node(n)) => {
            n.run()?;
        }
        _ => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                match cli.command {
                    // Execute Config CLI subcommand
                    Some(Sub::Config(config_cli)) => {
                        if let Err(e) = config_cli.run().await {
                            eprintln!("Failed to execute config tool, message: {}", &e);
                        }
                    }

                    // Execute Move CLI subcommand
                    Some(Sub::Move(move_tool)) => {
                        if let Err(e) = move_tool
                            .execute()
                            .await
                            .map_err(|e| anyhow!("Failed to execute move tool, message: {}", &e))
                        {
                            eprintln!("Failed to execute move tool, message: {}", &e);
                        }
                    }

                    // Execute Query CLI subcommand
                    Some(Sub::Query(query_cli)) => {
                        if let Err(e) = query_cli.run().await {
                            eprintln!("Failed to execute query tool, message: {}", &e);
                        }
                    }

                    // Execute Transactions CLI subcommand
                    Some(Sub::Txs(txs_cli)) => {
                        if let Err(e) = txs_cli.run().await {
                            eprintln!("Failed to execute txs tool, message: {}", &e);
                        }
                    }

                    // Execute Wallet CLI subcommand
                    Some(Sub::Wallet(wallet_cli)) => {
                        if let Err(e) = wallet_cli.run().await {
                            eprintln!("Failed to execute wallet tool, message: {}", &e);
                        }
                    }

                    Some(Sub::Ops(tool)) => {
                        if let Err(e) = tool.run().await {
                            eprintln!("Failed to execute ops tool, message: {}", &e);
                        }
                    }
                    // Display version information
                    Some(Sub::Version) => {
                        println!("LIBRA VERSION {}", env!("CARGO_PKG_VERSION"));
                        println!("build timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
                        // TODO:
                        // println!("rustc version: {}", env!("VERGEN_RUSTC_SEMVER"));
                        println!("git branch: {}", env!("VERGEN_GIT_BRANCH"));
                        println!("git SHA: {}", env!("VERGEN_GIT_SHA"));
                        println!(
                            "git commit timestamp: {}",
                            env!("VERGEN_GIT_COMMIT_TIMESTAMP")
                        );
                    }

                    // Default message if no valid subcommand is provided
                    _ => {
                        println!("\nliving is easy with eyes closed")
                    }
                }
            });
        }
    }

    Ok(())
}
