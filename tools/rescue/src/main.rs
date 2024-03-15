use std::time::Duration;

use clap::{Parser, Subcommand};
use rescue::{
    diem_db_bootstrapper::BootstrapOpts, fork_user::ForkOpts, rescue_tx::RescueTxOpts,
    twin::TwinOpts,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct RescueCli {
    #[clap(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    RescueTx(RescueTxOpts),
    Bootstrap(BootstrapOpts),
    HardFork(ForkOpts),
    Debug(TwinOpts),
}

fn main() -> anyhow::Result<()> {
    let cli = RescueCli::parse();
    match cli.command {
        Some(Sub::RescueTx(mission)) => {
            let blob_path = mission.run()?;

            // check it can bootstrap and print waypoiny
            let b = BootstrapOpts {
                db_dir: mission.db_dir,
                genesis_txn_file: blob_path,
                waypoint_to_verify: None,
                commit: false,
                info: false,
            };
            b.run()?;
        }
        Some(Sub::Bootstrap(bootstrap)) => {
            bootstrap.run()?;
        }
        Some(Sub::Debug(twin)) => {
            twin.run()?;
        }
        Some(Sub::HardFork(fork)) => {
            let blob_path = fork.run_ark_b()?;
            // check it can bootstrap and print waypoiny
            let b = BootstrapOpts {
                db_dir: fork.db_dir,
                genesis_txn_file: blob_path,
                waypoint_to_verify: None,
                commit: false,
                info: false,
            };
            b.run()?;
        }
        _ => {} // prints help
    }
    println!("done");
    // hack. let the DB close before exiting
    // TODO: fix in Diem or place in thread
    std::thread::sleep(Duration::from_millis(10));
    Ok(())
}
