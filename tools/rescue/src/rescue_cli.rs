//! CLI tool for rescue operations in Diem, providing commands for transaction rescue,
//! database bootstrapping, and debugging twin states.
use crate::{
    diem_db_bootstrapper::BootstrapOpts,
    rescue_tx::{
        check_rescue_bootstraps, register_vals, run_script_tx, save_rescue_blob, upgrade_tx,
    },
};

use clap::{Parser, Subcommand};
use libra_types::exports::AccountAddress;
use std::{path::PathBuf, time::Duration};
#[derive(Parser)]
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
    /// Registers new validators, and replaces the validator set.
    RegisterVals {
        #[clap(long)]
        /// registers new validators not found on the db, and replaces the validator set. Must be in format of operator.yaml (use `libra config validator init``)
        operator_yaml: Vec<PathBuf>,
        #[clap(short, long)]
        /// optional, provide a path to .mrb release, if this write should publish new framework
        upgrade_mrb: Option<PathBuf>,
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
            Sub::RegisterVals { operator_yaml, upgrade_mrb } => {
                let tx = register_vals(&self.db_path, operator_yaml, upgrade_mrb)?;

                let out_dir = self.blob_path.clone().unwrap_or(self.db_path.clone());
                let p = save_rescue_blob(tx, &out_dir)?;
                check_rescue_bootstraps(&self.db_path, &p)?;
            }
            Sub::UpgradeFramework { upgrade_mrb, set_validators } => {
                let tx = upgrade_tx(&self.db_path, upgrade_mrb, set_validators.clone())?;
                let out_dir = self.blob_path.clone().unwrap_or(self.db_path.clone());
                let p = save_rescue_blob(tx, &out_dir)?;
                check_rescue_bootstraps(&self.db_path, &p)?;
            }
            Sub::RunScript { script_path } => {
                let tx = run_script_tx(script_path.as_ref().unwrap())?;
                let out_dir = self.blob_path.clone().unwrap_or(self.db_path.clone());
                let p = save_rescue_blob(tx, &out_dir)?;
                check_rescue_bootstraps(&self.db_path, &p)?;
            }
        }
        // hack. let the DB close before exiting
        // TODO: fix in Diem or place in thread
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    }
}
