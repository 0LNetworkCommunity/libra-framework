use clap::{Args, Parser, Subcommand};
use diem_logger::warn;

use crate::{
    genesis_builder, parse_json, testnet_setup,
    wizard::{GenesisWizard, GITHUB_TOKEN_FILENAME},
};
use libra_types::{exports::NamedChain, global_config_dir, legacy_types::fixtures::TestPersona};
use std::{env, fs, net::Ipv4Addr, path::PathBuf};
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct GenesisCli {
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

impl GenesisCli {
    pub async fn execute(&self) -> anyhow::Result<()> {
        let data_path = self.home_dir.clone().unwrap_or_else(global_config_dir);

        match &self.command {
            Some(Sub::Genesis { github }) => {
                let github_token = find_github_token(&github.token_github_dir)?;

                let mut recovery = if let Some(p) = github.json_legacy.clone() {
                    parse_json::recovery_file_parse(p)?
                } else {
                    vec![]
                };

                genesis_builder::build(
                    github.org_github.to_owned(),
                    github.name_github.to_owned(),
                    github_token,
                    data_path,
                    github.local_framework,
                    &mut recovery,
                    self.chain.unwrap_or(NamedChain::TESTING),
                    None,
                )?;
            }
            Some(Sub::Register { github }) => {
                GenesisWizard::new(
                    github.org_github.to_owned(),
                    github.name_github.to_owned(),
                    Some(data_path),
                    self.chain.unwrap_or(NamedChain::TESTING),
                )
                .start_wizard(github.local_framework, github.json_legacy.clone(), false)
                .await?;
            }
            Some(Sub::Wizard { github }) => {
                GenesisWizard::new(
                    github.org_github.to_owned(),
                    github.name_github.to_owned(),
                    Some(data_path),
                    self.chain.unwrap_or(NamedChain::TESTING),
                )
                .start_wizard(github.local_framework, github.json_legacy.clone(), true)
                .await?;
            }
            Some(Sub::Testnet {
                me,
                ip_list,
                json_legacy,
            }) => {
                testnet_setup::setup(
                    me,
                    ip_list,
                    self.chain.unwrap_or(NamedChain::TESTING),
                    data_path,
                    json_legacy.to_owned(),
                )
                .await?
            }
            _ => {
                println!("\nIf you're looking for trouble \nYou came to the right place");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Args)]
#[clap(arg_required_else_help = true)]
struct GithubArgs {
    /// optionally provide a github token, otherwise will search in home_dir/github_token.txt
    #[clap(long)]
    token_github_dir: Option<PathBuf>,
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

/// help the user locate a github_token.txt in $HOME/.libra or working directory.
fn find_github_token(data_path_opt: &Option<PathBuf>) -> anyhow::Result<String> {
    // try to find in specified path
    let mut p = data_path_opt
        .as_ref()
        // try to find in $HOME/.libra
        .unwrap_or(&global_config_dir())
        .join(GITHUB_TOKEN_FILENAME);
    // try to find in working dir
    if !p.exists() {
        p = env::current_dir()?.join(GITHUB_TOKEN_FILENAME);
        warn!(
            "github_token.txt not found in {}. Trying the working path",
            p.display()
        )
    };
    Ok(fs::read_to_string(p)?.trim().to_owned())
}
