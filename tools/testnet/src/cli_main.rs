use crate::cli_output::TestnetCliOut;
use crate::restore_helper::one_step_restore_db;
use crate::{cli_config::TestnetConfigOpts, cli_swarm::SwarmCliOpts};
use anyhow::Result;
use async_trait::async_trait;
use clap::Subcommand;
use clap::{self, Parser};
use diem::common::types::{CliCommand, CliError, CliTypedResult};
use diem_framework::ReleaseBundle;
use libra_types::global_config_dir;
use std::path::PathBuf;
#[derive(Parser)]

/// Setup testnet files, or start a LibraSmoke.
/// Can be a virgin network with alice & co., or a twin network
/// with a reference db (from mainnet for example)
pub struct TestnetCli {
    #[clap(short, long)]
    /// Path to a framework mrb file
    /// If not provided, will try to search in this path
    /// at ./framework/releases/head.mrb
    pub framework_mrb_path: Option<PathBuf>,

    #[clap(long, conflicts_with = "twin_db")]
    /// Run a twin of mainnet, instead of a virgin network
    pub twin_epoch: Option<u64>,

    #[clap(long, requires = "twin_epoch")]
    /// Data path to use for the restore files and twin db, defaults to $HOME/.libra/
    pub data_path: Option<PathBuf>,

    #[clap(long, conflicts_with = "twin_epoch")]
    /// You already have a reference db for twin
    pub twin_db: Option<PathBuf>,

    #[clap(long, short)]
    /// print json output
    pub json: bool,

    #[clap(subcommand)]
    pub command: Sub,
}

#[derive(Subcommand)]
pub enum Sub {
    /// configs for genesis
    Configure(TestnetConfigOpts),
    /// start using Diem swarm
    Smoke(SwarmCliOpts),
}

#[async_trait]
impl CliCommand<TestnetCliOut> for TestnetCli {
    fn command_name(&self) -> &'static str {
        match self.command {
            Sub::Configure(_) => "testnet-configure",
            Sub::Smoke(_) => "testnet-smoke",
        }
    }

    async fn execute(self) -> CliTypedResult<TestnetCliOut> {
        let move_release = if let Some(p) = self.framework_mrb_path.clone() {
            ReleaseBundle::read(p).map_err(CliError::from)?
        } else {
            println!("assuming you are running this in the source repo. Will try to search in this path at ./framework/releases/head.mrb");
            libra_framework::testing_local_release_bundle()
        };

        // we have a reference db we'd like to use
        let reference_db = if self.twin_db.is_some() {
            println!("using reference database: {:?}", self.twin_db);
            self.twin_db.clone()
        } else if let Some(e) = self.twin_epoch {
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
            Sub::Configure(config) => Ok(config.run(self.framework_mrb_path, reference_db).await?),
            Sub::Smoke(smoke) => {
                check_bins_path()?;
                println!("starting local testnet using Diem swarm...");
                assert!(reference_db.is_some(), "no db");
                Ok(smoke.run(move_release, reference_db).await?)
            }
        }
    }
}

// Keep the run method for backwards compatibility
impl TestnetCli {
    pub async fn run(self) -> anyhow::Result<()> {
        let is_json = self.json.clone();
        match &self.execute_serialized().await {
            Ok(res) => {
                if is_json {
                    println!("{}", res);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
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
