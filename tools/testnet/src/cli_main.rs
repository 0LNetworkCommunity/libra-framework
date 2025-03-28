use crate::restore_helper::one_step_restore_db;
use crate::{cli_config::TestnetConfigOpts, cli_swarm::SwarmCliOpts};
use anyhow::Result;
use clap::Subcommand;
use clap::{self, Parser};
use diem_framework::ReleaseBundle;
use libra_types::global_config_dir;
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
    twin_epoch: Option<u64>,

    #[clap(long, requires = "twin_epoch")]
    /// Data path to use for the restore files and twin db, defaults to $HOME/.libra/
    data_path: Option<PathBuf>,

    #[clap(long, conflicts_with = "twin_epoch")]
    /// You already have a reference db for twin
    twin_db: Option<PathBuf>,

    #[clap(subcommand)]
    command: Sub,
}

#[derive(Subcommand)]
pub enum Sub {
    /// configs for genesis
    Configure(TestnetConfigOpts),
    /// start using Diem swarm
    Smoke(SwarmCliOpts),
}

impl TestnetCli {
    pub async fn run(self) -> anyhow::Result<()> {
        let move_release = if let Some(p) = self.framework_mrb_path.clone() {
            ReleaseBundle::read(p)?
        } else {
            println!("assuming you are running this in the source repo. Will try to search in this path at ./framework/releases/head.mrb");
            libra_framework::testing_local_release_bundle()
        };
        // we have a reference db we'd like to use
        let reference_db = if self.twin_db.is_some() {
            println!("using reference database: {:?}", self.twin_db);
            self.twin_db.clone()
        } else
        // else we might be trying to restore
        if let Some(e) = self.twin_epoch {
            let data_path = self.data_path.unwrap_or_else(global_config_dir);
            println!("downloading restore archive and creating a new db");
            one_step_restore_db(data_path, e, None, None, None)
                .await
                .ok()
        } else {
            println!("configuring virgin network...");
            None
        };

        match self.command {
            Sub::Configure(cli) => {
                // first configure a vanilla genesis
                cli.run(self.framework_mrb_path, reference_db).await?;
            }
            Sub::Smoke(cli) => {
                check_bins_path()?;

                println!("starting local testnet using Diem swarm...");
                assert!(reference_db.is_some(), "no db");
                cli.run(move_release, reference_db).await?;
            }
        }
        Ok(())
    }
}

fn check_bins_path() -> Result<()> {
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
    Ok(())
}
