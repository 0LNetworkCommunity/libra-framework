use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libra_genesis_tools::{wizard::{GenesisWizard, DEFAULT_DATA_PATH, GITHUB_TOKEN_FILENAME}, genesis_builder};

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
    Testnet {
      /// what are the settings for the genesis repo configs
      #[arg(short, long)]
      genesis_repo_org: String,
      /// name of the repo
      #[arg(short, long)]
      repo_name: String,
      /// uses the local framework build
      #[arg(short, long)]
      use_local_framework: bool,
    },
    Wizard {
        /// choose a different home data folder for all node data.
        /// defaults to $HOME/.libra
        #[arg(long)]
        home_dir: Option<PathBuf>,

        /// if we should use a local mrb framework instead of the one from github. This is useful for testing.
        #[arg(short,long)]
        local_framework: bool,
    }
}

fn main() -> anyhow::Result<()>{
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Fork { test_mode }) => {
            dbg!(&test_mode);
            // make_recovery_genesis_from_vec_legacy_recovery();
        }
        Some(Sub::Testnet { genesis_repo_org, repo_name, use_local_framework }) => {
          let data_path = dirs::home_dir()
            .expect("no home dir found")
            .join(DEFAULT_DATA_PATH);
          
          let github_token = std::fs::read_to_string(&data_path.join(GITHUB_TOKEN_FILENAME))?;

          genesis_builder::build(
                genesis_repo_org,
                repo_name,
                github_token,
                data_path,
                use_local_framework,
                None,
            )?;
        }
        Some(Sub::Wizard { home_dir, local_framework }) => {
            GenesisWizard::default().start_wizard(home_dir, local_framework)?;
        }
        _ => {}
    }

    // Continued program logic goes here...
    Ok(())
}
