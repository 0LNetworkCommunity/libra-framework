use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libra_genesis_tools::wizard::GenesisWizard;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct GenesisCliArgs {
    #[command(subcommand)]
    command: Option<Sub>,
}

#[derive(Subcommand)]
enum Sub {
    /// does testing things
    Fork {
        /// lists test values
        #[arg(short, long)]
        test_mode: bool,
    },
    Wizard {
        /// choose a different home data folder for all node data.
        /// defaults to $HOME/.libra
        #[arg(long)]
        home_dir: Option<PathBuf>,
    }
}

fn main() -> anyhow::Result<()>{
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Fork { test_mode }) => {
            dbg!(&test_mode);
            // make_recovery_genesis_from_vec_legacy_recovery();
        }
        Some(Sub::Wizard { home_dir }) => {
            GenesisWizard::default().start_wizard(home_dir)?;
        }
        _ => {}
    }

    // Continued program logic goes here...
    Ok(())
}
