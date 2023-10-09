
use clap::{Parser, Subcommand};
use rescue::{rescue_tx::RescueTxOpts, diem_db_bootstrapper::BootstrapOpts};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct RescueCli {
    #[clap(subcommand)]
    command: Option<Sub>,
    #[clap(long)]
    /// apply to db in one step.
    apply_to_db: bool,
}

#[derive(Subcommand)]
enum Sub {
    RescueTx(RescueTxOpts),
    Bootstrap(BootstrapOpts)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RescueCli::parse();
    match cli.command {
        Some(Sub::RescueTx(mission)) => {
            let blob_path = mission.run().await?;

            if cli.apply_to_db {
              let b = BootstrapOpts {
                db_dir: mission.data_path,
                genesis_txn_file: blob_path,
                waypoint_to_verify: None,
                commit: true
              };
              b.run()?;
            };
            println!("SUCCESS: rescue mission complete.");
        },
        Some(Sub::Bootstrap(bootstrap)) => {
            bootstrap.run()?;
            println!("SUCCESS: db boostrapped with writeset (genesis tx)");
        }
        _ => {
            println!("\nI'll be there")
        }
    }

    Ok(())
}
