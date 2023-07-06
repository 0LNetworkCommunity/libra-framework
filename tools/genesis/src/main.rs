use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libra_genesis_tools::{wizard::{GenesisWizard, GITHUB_TOKEN_FILENAME}, genesis_builder, parse_json, ancestry};
use libra_types::global_config_dir;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct GenesisCliArgs {
    #[clap(subcommand)]
    command: Option<Sub>,

    /// choose a different home data folder for all node data.
    /// defaults to $HOME/.libra
    #[clap(long)]
    home_dir: Option<PathBuf>,
    /// optionally provide a github token, otherwise will search in home_dir/github_token.txt
    #[clap(long)]
    token_github: Option<String>,
    /// what are the settings for the genesis repo configs
    #[clap(short, long)]
    org_github: String,
    /// name of the repo
    #[clap(short, long)]
    name_github: String,
    /// uses the local framework build
    #[clap(short, long)]
    local_framework: bool,
    /// path to file for legacy migration file
    #[clap(short, long)]
    json_legacy: Option<PathBuf>,
    // /// path to file for ancestry file to patch data
    // #[clap(short, long)]
    // json_ancestry: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Sub {
    Genesis {}, // just do genesis without wizard
    Register {}, // just do registration without wizard
    Wizard {}
}

fn main() -> anyhow::Result<()>{
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Genesis {}) => {
          let data_path = cli.home_dir.unwrap_or_else(|| {
            global_config_dir()
          });
       
          
          let github_token = cli.token_github
          .unwrap_or(
            std::fs::read_to_string(
              &data_path
              .join(GITHUB_TOKEN_FILENAME)
            )?.trim().to_string()
          );
          
          // // if we have an ancestry file we need it before loading recovery and possinly fixing ancestry fields there
          // let ancestry = if let Some(p) = cli.json_ancestry {
          //   let json = ancestry::parse_ancestry_json(p)?;
          //   Some(&ancestry::map_ancestry(&json)?)
          // } else{ None };

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
        Some(Sub::Register { }) => {
            GenesisWizard::new(
              cli.org_github, 
              cli.name_github, 
              cli.home_dir
            )
              .start_wizard(
                cli.local_framework, 
                cli.json_legacy, 
                false
            )?;
        }
        Some(Sub::Wizard { }) => {
            GenesisWizard::new(
              cli.org_github, 
              cli.name_github, 
              cli.home_dir
            )
              .start_wizard(
                cli.local_framework, 
                cli.json_legacy, 
                false
            )?;
        }
        _ => {}
    }

    // Continued program logic goes here...
    Ok(())
}
