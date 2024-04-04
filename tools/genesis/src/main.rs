use anyhow::Context;
use clap::{Args, Parser, Subcommand};

use libra_genesis_tools::{
    genesis_builder, parse_json, testnet_setup,
    wizard::{GenesisWizard, GITHUB_TOKEN_FILENAME},
};
use libra_types::{exports::NamedChain, global_config_dir, legacy_types::fixtures::TestPersona};
use std::{net::Ipv4Addr, path::PathBuf};
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
}

#[derive(Debug, Clone, Args)]
#[clap(arg_required_else_help = true)]
struct GithubArgs {
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
        /// github args
        #[clap(flatten)]
        github: GithubArgs,
    }, // just do genesis without wizard
    Register {
        /// github args
        #[clap(flatten)]
        github: GithubArgs,
    },
    // full wizard
    Wizard {
        /// github args
        #[clap(flatten)]
        github: GithubArgs,
    },
    /// sensible defaults for testnet, does not need a genesis repo
    /// accounts are created from fixture mnemonics for alice, bob, carol, dave
    Testnet {
        /// which persona is this machine going to register as
        #[clap(short, long)]
        me: TestPersona,
        /// list of IP addresses of each persona Alice, Bob, Carol, Dave
        #[clap(short, long)]
        ip_list: Vec<Ipv4Addr>,
        /// path to file for legacy migration file
        #[clap(short, long)]
        json_legacy: Option<PathBuf>,
    },
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Genesis { github }) => {
            let data_path = cli.home_dir.unwrap_or_else(global_config_dir);

            let github_token = github.token_github.unwrap_or(
                std::fs::read_to_string(data_path.join(GITHUB_TOKEN_FILENAME))
                    .context("cannot find github_token.txt in config path")?
                    .trim()
                    .to_string(),
            );

            let mut recovery = if let Some(p) = github.json_legacy {
                parse_json::recovery_file_parse(p)?
            } else {
                vec![]
            };

            genesis_builder::build(
                github.org_github,
                github.name_github,
                github_token,
                data_path,
                github.local_framework,
                &mut recovery,
                cli.chain.unwrap_or(NamedChain::TESTING),
                None,
            )?;
        }
        Some(Sub::Register { github }) => {
            GenesisWizard::new(
                github.org_github,
                github.name_github,
                cli.home_dir,
                cli.chain.unwrap_or(NamedChain::TESTING),
            )
            .start_wizard(github.local_framework, github.json_legacy, false)
            .await?;
        }
        Some(Sub::Wizard { github }) => {
            GenesisWizard::new(
                github.org_github,
                github.name_github,
                cli.home_dir,
                cli.chain.unwrap_or(NamedChain::TESTING),
            )
            .start_wizard(github.local_framework, github.json_legacy, true)
            .await?;
        }
        Some(Sub::Testnet {
            me,
            ip_list,
            json_legacy,
        }) => {
            testnet_setup::setup(
                &me,
                &ip_list,
                cli.chain.unwrap_or(NamedChain::TESTING),
                cli.home_dir.unwrap_or_else(global_config_dir),
                json_legacy,
            )
            .await?
        }
        _ => {
            println!("\nIf you're looking for trouble \nYou came to the right place");
        }
    }
    Ok(())
}
