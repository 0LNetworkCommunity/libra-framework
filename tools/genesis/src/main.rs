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
    /// uses the local framework build
    #[arg(short, long)]
    github_token: Option<String>,
    /// what are the settings for the genesis repo configs
    #[arg(short, long)]
    genesis_repo_org: String,
    /// name of the repo
    #[arg(short, long)]
    repo_name: String,
    /// uses the local framework build
    #[arg(short, long)]
    use_local_framework: bool,
    /// uses the local framework build
    #[arg(short, long)]
    legacy_json: Option<PathBuf>,
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
       
          
          let github_token = cli.github_token
          .unwrap_or(
            std::fs::read_to_string(
              &data_path
              .join(GITHUB_TOKEN_FILENAME)
            )?.trim().to_string()
          );
          

          let recovery = if let Some(p) = cli.legacy_json {
            parse_json::parse(p)?
          } else { vec![] };

          genesis_builder::build(
                cli.genesis_repo_org,
                cli.repo_name,
                github_token,
                data_path,
                cli.use_local_framework,
                Some(&recovery),
            )?;
        }
        Some(Sub::Wizard { }) => {
            GenesisWizard::default().start_wizard(cli.home_dir, cli.use_local_framework)?;
        }
        _ => {}
    }

    // Continued program logic goes here...
    Ok(())
}
