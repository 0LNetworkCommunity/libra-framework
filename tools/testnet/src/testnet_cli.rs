use clap::{self, Parser};
use clap::Subcommand;
use crate::twin_cli::TwinCli;
/// Twin of the network
#[derive(Parser)]

/// Set up a twin of the network, with a synced db
pub struct TestnetCli {
    #[clap(subcommand)]
    command: Sub,
}

#[derive(Subcommand)]
pub enum Sub {
    /// configs for clean genesis and fixed validators
    Genesis,
    /// configs for a fork from a mainnet db
    Twin,
    /// use Diem swarm for a local testnet
    Swarm(TwinCli),
}

impl TestnetCli {
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
            Sub::Genesis => {
                println!("configuring clean genesis...");
            }
            Sub::Twin => {
                println!("configuring network twin from mainnet db...");
            }
            Sub::Swarm(cli) => {
                println!("starting local testnet using Diem swarm...");
                // Delegate to nested TwinCli if Swarm variant is used
                cli.run().await?;
            }
        }
        Ok(())
    }
}
