
use clap::{Parser, Subcommand};
use rescue::{rescue_tx::RescueTxOpts, diem_db_bootstrapper::BootstrapOpts};

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
    Bootstrap(BootstrapOpts)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RescueCli::parse();
    match cli.command {
        Some(Sub::RescueTx(mission)) => {
            mission.run().await?;
            println!("SUCCESS: rescue mission complete.")

        },
        Some(Sub::Bootstrap(bootstrap)) => {
            bootstrap.run()?;
            println!("SUCCESS: db boostrapped with writeset (genesis tx)")
        }
        _ => {
            println!("\nI'll be there")
        }
    }

    Ok(())
}
