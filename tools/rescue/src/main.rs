use std::time::Duration;

use clap::{Parser, Subcommand};
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts, twin::TwinOpts};

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
    Debug(TwinOpts),
}

fn main() -> anyhow::Result<()> {
    let cli = RescueCli::parse();
    match cli.command {
        Some(Sub::RescueTx(mission)) => {
            let blob_path = mission.run()?;

            let b = BootstrapOpts {
                db_dir: mission.data_path,
                genesis_txn_file: blob_path,
                waypoint_to_verify: None,
                commit: false,
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
