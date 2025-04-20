//! CLI tool for rescue operations in Diem, providing commands for transaction rescue,
//! database bootstrapping, and debugging twin states.
use crate::{
    cli_bootstrapper::{check_rescue_bootstraps, BootstrapOpts},
    node_config::post_rescue_node_file_updates,
    transaction_factory::{register_vals, run_script_tx, save_rescue_blob, upgrade_tx},
};

use clap::{Parser, Subcommand};
use diem_types::waypoint::Waypoint;
use libra_types::exports::AccountAddress;
use std::{path::PathBuf, time::Duration};

/// Constants for blob file names
pub const REPLACE_VALIDATORS_BLOB: &str = "replace_validators_rescue.blob";
pub const UPGRADE_FRAMEWORK_BLOB: &str = "upgrade_framework_rescue.blob";
pub const RUN_SCRIPT_BLOB: &str = "run_script_rescue.blob";

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
/// Apply writeset transactions to DB at rest
pub struct RescueCli {
    #[clap(short, long)]
    /// path to the reference db, often $HOME/.libra/data/db
    pub db_path: PathBuf,
    #[clap(short, long)]
    /// directory to read/write or the rescue.blob. Will default to db_path/rescue.blob
    pub blob_path: Option<PathBuf>,

    #[clap(subcommand)]
    pub command: Sub,
}

#[derive(Subcommand)]
pub enum Sub {
    Bootstrap(BootstrapOpts),
    /// once the node is started run this command to update safety rules
    PatchSafetyRules {
        #[clap(short, long)]
        /// path to validator.yaml
        config_path: PathBuf,
        /// rescue blob path
        #[clap(short, long)]
        blob_path: PathBuf,
        /// expected waypoint
        #[clap(short, long)]
        waypoint: Waypoint,
    },
    /// Registers new validators, and replaces the validator set.
    RegisterVals {
        #[clap(long)]
        /// registers new validators not found on the db, and replaces the validator set. Must be in format of operator.yaml (use `libra config validator init`)
        operator_yaml: Vec<PathBuf>,
        #[clap(short, long)]
        /// optional, provide a path to .mrb release, if this write should publish new framework
        upgrade_mrb: Option<PathBuf>,
        #[clap(short, long)]
        /// optional, chain_id to use, default is 2 (staging)
        chain_id: Option<u8>,
    },
    /// Upgrades the framework in the reference DB
    UpgradeFramework {
        #[clap(short, long)]
        /// provide a path to .mrb release, if this write should publish new framework
        upgrade_mrb: PathBuf,

        #[clap(short, long)]
        /// optional, update validator set (must be previously registered on db)
        set_validators: Option<Vec<AccountAddress>>,
    },
    // Run a Move script from a file. Must use code from reference DB's framework.
    RunScript {
        #[clap(short, long)]
        /// directory to read/write or the rescue.blob
        script_path: Option<PathBuf>,
    },
}

impl RescueCli {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Sub::Bootstrap(bootstrap) => {
                bootstrap.run()?;
            }
            Sub::RegisterVals {
                operator_yaml,
                upgrade_mrb,
                chain_id,
            } => {
                let tx = register_vals(&self.db_path, operator_yaml, upgrade_mrb, *chain_id)?;

                let out_file = self
                    .blob_path
                    .clone()
                    .unwrap_or(self.db_path.clone())
                    .join(REPLACE_VALIDATORS_BLOB);
                let p = save_rescue_blob(tx, &out_file)?;
                check_rescue_bootstraps(&self.db_path, &p)?;
            }
            Sub::UpgradeFramework {
                upgrade_mrb,
                set_validators,
            } => {
                let tx = upgrade_tx(&self.db_path, upgrade_mrb, set_validators.clone())?;
                let out_dir = self
                    .blob_path
                    .clone()
                    .unwrap_or(self.db_path.clone())
                    .join(UPGRADE_FRAMEWORK_BLOB);

                let p = save_rescue_blob(tx, &out_dir)?;
                check_rescue_bootstraps(&self.db_path, &p)?;
            }
            Sub::RunScript { script_path } => {
                let tx = run_script_tx(script_path.as_ref().unwrap())?;
                let out_dir = self
                    .blob_path
                    .clone()
                    .unwrap_or(self.db_path.clone())
                    .join(RUN_SCRIPT_BLOB);
                let p = save_rescue_blob(tx, &out_dir)?;
                check_rescue_bootstraps(&self.db_path, &p)?;
            }
            Sub::PatchSafetyRules {
                config_path,
                blob_path,
                waypoint,
            } => {
                post_rescue_node_file_updates(config_path, *waypoint, blob_path)?;
            }
        }
        // hack. let the DB close before exiting
        // TODO: fix in Diem or place in thread
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    }
}
