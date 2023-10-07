
mod rescue_cli;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use rescue_cli::MissionOpts;


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
struct RescueCli {
    #[clap(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    Mission(MissionOpts),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RescueCli::parse();
    match cli.command {
        Some(Sub::Mission(mission)) => {
            mission.run().await?;
        }
        _ => {
            println!("\nI'll be there")
        }
    }

    Ok(())
}
