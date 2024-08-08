//! CLI tool for rescue operations in Diem, providing commands for transaction rescue,
//! database bootstrapping, and debugging twin states.
use crate::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts, twin::TwinOpts};

use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
/// Apply writeset transactions to DB at rest
pub struct RescueCli {
    #[clap(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    RescueTx(RescueTxOpts),
    Bootstrap(BootstrapOpts),
    Debug(TwinOpts),
}

impl RescueCli {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Some(Sub::RescueTx(mission)) => {
                let blob_path = mission.run()?;

                let b = BootstrapOpts {
                    db_dir: mission.data_path.clone(),
                    genesis_txn_file: blob_path,
                    waypoint_to_verify: None,
                    commit: false,
                    info: false,
                };
                let _ = b.run()?;
            }
            Some(Sub::Bootstrap(bootstrap)) => {
                bootstrap.run()?;
            }
            Some(Sub::Debug(twin)) => {
                twin.run()?;
            }
            _ => {} // prints help
        }
        println!("done");
        // hack. let the DB close before exiting
        // TODO: fix in Diem or place in thread
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    }
}
