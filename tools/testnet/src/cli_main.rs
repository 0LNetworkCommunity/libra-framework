use crate::{cli_config::TestnetConfigOpts, cli_swarm::SwarmCliOpts};
use clap::Subcommand;
use clap::{self, Parser};
use diem_framework::ReleaseBundle;
use std::path::PathBuf;

/// Twin of the network
#[derive(Parser)]
/// Set up a twin of the network, with a synced db
pub struct TestnetCli {
    #[clap(short, long)]
    /// Path to a framework mrb file
    /// If not provided, will try to search in this path
    /// at ./framework/releases/head.mrb
    framework_mrb_path: Option<PathBuf>,

    #[clap(long, conflicts_with = "twin_db")]
    /// Run a twin of mainnet, instead of a virgin network
    twin_epoch: bool,

    #[clap(long, conflicts_with = "twin_epoch")]
    /// If running a twin with a reference db
    twin_db: Option<PathBuf>,

    #[clap(subcommand)]
    command: Sub,
}

#[derive(Subcommand)]
pub enum Sub {
    /// configs for genesis
    Configure(TestnetConfigOpts),
    /// start using containers
    StartContainer,
    /// start using Diem swarm
    StartSwarm(SwarmCliOpts),
}

impl TestnetCli {
    pub async fn run(self) -> anyhow::Result<()> {
        let bundle = if let Some(p) = self.framework_mrb_path.clone() {
            ReleaseBundle::read(p)?
        } else {
            println!("assuming you are running this in the source repo. Will try to search in this path at ./framework/releases/head.mrb");
            libra_framework::testing_local_release_bundle()
        };

        if self.twin_epoch {
            println!("do you want to download an epoch archive and restore?");
        } else if self.twin_db.is_some() {
            println!("using reference database: {:?}", self.twin_db);
        } else {
            println!("configuring virgin network...");
            // check that the user has DIEM_FORGE_NODE_BIN_PATH=libra in their path
            let libra_var = std::env::var("DIEM_FORGE_NODE_BIN_PATH");
            match libra_var {
                Ok(value) => {
                    let path = PathBuf::from(&value);
                    if !path.exists() {
                        anyhow::bail!(
                            "DIEM_FORGE_NODE_BIN_PATH '{}' does not exist as a valid path",
                            value
                        );
                    }
                }
                Err(_) => {
                    anyhow::bail!("DIEM_FORGE_NODE_BIN_PATH environment variable not set");
                }
            }
        }

        match self.command {
            Sub::Configure(cli) => {
                // first configure a vanilla genesis
                cli.run(self.framework_mrb_path, self.twin_db).await?;
            }
            Sub::StartContainer => {
                println!("starting local testnet using containers...");
                todo!();
            }
            Sub::StartSwarm(cli) => {
                println!("starting local testnet using Diem swarm...");
                cli.run(self.twin_db, bundle).await?;
            }
        }
        Ok(())
    }
}
