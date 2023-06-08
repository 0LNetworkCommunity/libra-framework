use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libra_genesis_tools::{wizard::{GenesisWizard, DEFAULT_DATA_PATH, GITHUB_TOKEN_FILENAME}, genesis_builder, parse_json};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct GenesisCliArgs {
    #[command(subcommand)]
    command: Option<Sub>,

    /// choose a different home data folder for all node data.
    /// defaults to $HOME/.libra
    #[arg(long)]
    home_dir: Option<PathBuf>,
    /// optionally provide a github token, otherwise will search in home_dir/github_token.txt
    #[arg(long)]
    token_github: Option<String>,
    /// what are the settings for the genesis repo configs
    #[arg(short, long)]
    org_github: String,
    /// name of the repo
    #[arg(short, long)]
    name_github: String,
    /// uses the local framework build
    #[arg(short, long)]
    local_framework: bool,
    /// path to file for legacy migration file
    #[arg(short, long)]
    json_legacy: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Sub {
    Genesis {}, // just do genesis without wizard
    Wizard {}
}

fn main() -> anyhow::Result<()>{
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Genesis {}) => {
          let data_path = cli.home_dir.unwrap_or_else(|| {
            dirs::home_dir()
            .expect("no home dir found")
            .join(DEFAULT_DATA_PATH)
          });
       
          
          let github_token = cli.token_github
          .unwrap_or(
            std::fs::read_to_string(
              &data_path
              .join(GITHUB_TOKEN_FILENAME)
            )?.trim().to_string()
          );
          

          let recovery = if let Some(p) = cli.json_legacy {
            parse_json::parse(p)?
          } else { vec![] };

          genesis_builder::build(
                cli.org_github,
                cli.name_github,
                github_token,
                data_path,
                cli.local_framework,
                Some(&recovery),
            )?;
        }
        Some(Sub::Wizard { }) => {
            GenesisWizard::default().start_wizard(cli.home_dir, cli.local_framework)?;
        }
        _ => {}
    }

    // Continued program logic goes here...
    Ok(())
}
