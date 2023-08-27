use anyhow::Context;
use clap::{Parser, Subcommand};
use libra_genesis_tools::{
    genesis_builder, parse_json,
    supply::SupplySettings,
    wizard::{GenesisWizard, GITHUB_TOKEN_FILENAME},
};
use libra_types::{exports::NamedChain, global_config_dir};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct GenesisCliArgs {
    #[clap(subcommand)]
    command: Option<Sub>,
    /// name of the type of chain we are starting
    #[clap(short, long)]
    chain: Option<NamedChain>,

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
}

#[derive(Subcommand)]
enum Sub {
    Genesis {
        #[clap(flatten)]
        /// optional, settings for supply.
        supply_settings: SupplySettings,
    }, // just do genesis without wizard
    Register {}, // just do registration without wizard
    Wizard {
        #[clap(flatten)]
        /// optional, settings for supply.
        supply_settings: SupplySettings,
    },
    /// sensible defaults for testnet, does not need a genesis repo
    /// accounts are created from fixture mnemonics for alice, bob, carol, dave
    Testnet {}
}

fn main() -> anyhow::Result<()> {
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Genesis { supply_settings }) => {
            let data_path = cli.home_dir.unwrap_or_else(global_config_dir);

            let github_token = cli.token_github.unwrap_or(
                std::fs::read_to_string(data_path.join(GITHUB_TOKEN_FILENAME))
                    .context("cannot find github_token.txt in config path")?
                    .trim()
                    .to_string(),
            );

            let recovery = if let Some(p) = cli.json_legacy {
                parse_json::parse(p)?
            } else {
                vec![]
            };

            genesis_builder::build(
                cli.org_github,
                cli.name_github,
                github_token,
                data_path,
                cli.local_framework,
                Some(&recovery),
                Some(supply_settings),
                cli.chain.unwrap_or(NamedChain::TESTING),
            )?;
        }
        Some(Sub::Register {}) => {
            GenesisWizard::new(
                cli.org_github,
                cli.name_github,
                cli.home_dir,
                cli.chain.unwrap_or(NamedChain::TESTING),
            )
            .start_wizard(cli.local_framework, cli.json_legacy, false, None)?;
        }
        Some(Sub::Wizard { supply_settings }) => {
            GenesisWizard::new(
                cli.org_github,
                cli.name_github,
                cli.home_dir,
                cli.chain.unwrap_or(NamedChain::TESTING),
            )
            .start_wizard(
                cli.local_framework,
                cli.json_legacy,
                true,
                Some(supply_settings),
            )?;
        }
        _ => {
            println!("\nIf you're looking for trouble \nYou came to the right place");
        }
    }

    // Continued program logic goes here...
    Ok(())
}
