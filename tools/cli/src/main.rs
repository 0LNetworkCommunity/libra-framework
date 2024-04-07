mod move_cli;
mod node_cli;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use libra_config::config_cli::ConfigCli;
use libra_genesis_tools::cli::GenesisCli;
use libra_query::query_cli::QueryCli;
use libra_txs::txs_cli::TxsCli;
use libra_wallet::wallet_cli::WalletCli;

#[derive(Parser)]
#[clap(author, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct LibraCli {
    #[clap(subcommand)]
    command: Option<Sub>,
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
    Genesis(GenesisCli),
    Version,
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
                    Some(Sub::Genesis(genesis_cli)) => {
                        if let Err(e) = genesis_cli.execute().await {
                            eprintln!("Failed to execute genesis tool, message: {}", &e);
                        }
                    }
                    Some(Sub::Version) => {
                        println!("LIBRA VERSION {}", env!("CARGO_PKG_VERSION"));
                        println!("Build Timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
                        println!("Git Branch: {}", env!("VERGEN_GIT_BRANCH"));
                        println!("Git SHA: {}", env!("VERGEN_GIT_SHA"));
                        println!(
                            "Git Commit Timestamp: {}",
                            env!("VERGEN_GIT_COMMIT_TIMESTAMP")
                        );
                    }
                    // Some(Sub::Version(v)) => {
                    //     clap_vergen::print!(version);
                    // }
                    _ => {
                        println!("\nliving is easy with eyes closed")
                    }
                }
            });
        }
    }

    Ok(())
}
