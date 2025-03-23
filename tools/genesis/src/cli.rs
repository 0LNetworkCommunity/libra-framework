use clap::{Args, Parser, Subcommand};
use diem_genesis::config::HostAndPort;

use crate::{
    genesis_builder, parse_json, testnet_setup,
    wizard::{GenesisWizard, GITHUB_TOKEN_FILENAME},
};
use libra_types::{core_types::fixtures::TestPersona, exports::NamedChain, global_config_dir};
use std::{fs, path::PathBuf};
#[derive(Parser)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
/// Generate genesis transactions for testnet and upgrades
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
    pub async fn run(&self) -> anyhow::Result<()> {
        let data_path = self.home_dir.clone().unwrap_or_else(global_config_dir);
        let chain_name = self.chain.unwrap_or(NamedChain::TESTNET); // chain_id = 2

        match &self.command {
            Some(Sub::Build { github, drop_list }) => {
                let mut recovery = if let Some(p) = github.json_legacy.clone() {
                    parse_json::recovery_file_parse(p)?
                } else {
                    vec![]
                };

                if let Some(dp) = drop_list {
                    parse_json::drop_accounts(&mut recovery, dp)?;
                };

                // TODO: there's no reason a github token should be needed to
                // read the genesis
                let github_token = fs::read_to_string(data_path.join(GITHUB_TOKEN_FILENAME))?;

                genesis_builder::build(
                    github.org_github.to_owned(),
                    github.name_github.to_owned(),
                    github_token,
                    data_path,
                    github.local_framework.to_owned(),
                    &mut recovery,
                    chain_name,
                    None,
                )?;
            }
            Some(Sub::Register { github }) => {
                GenesisWizard::new(
                    github.org_github.to_owned(),
                    github.name_github.to_owned(),
                    Some(data_path),
                    chain_name,
                )
                .start_wizard(
                    github.local_framework.to_owned(),
                    github.json_legacy.to_owned(),
                    github.token_github_file.to_owned(),
                    false,
                )
                .await?;
            }

            Some(Sub::Testnet {
                me,
                host_list,
                framework_mrb_path,
                json_legacy,
            }) => {
                testnet_setup::setup(
                    me,
                    host_list,
                    chain_name,
                    data_path,
                    json_legacy.to_owned(),
                    Some(framework_mrb_path.to_owned()),
                )
                .await?;
            }
            _ => {}
        }
        println!("\nIf you're looking for trouble \nYou came to the right place\n");
        Ok(())
    }
}

#[derive(Debug, Clone, Args)]
#[clap(arg_required_else_help = true)]
struct GithubArgs {
    /// optionally provide a github token, otherwise will search in home_dir/github_token.txt
    #[clap(long)]
    token_github_file: Option<PathBuf>,
    /// what are the settings for the genesis repo configs
    #[clap(short, long)]
    org_github: String,
    /// name of the repo
    #[clap(short, long)]
    name_github: String,
    /// uses the local framework build
    #[clap(short, long)]
    local_framework: Option<PathBuf>,
    /// path to file for legacy migration file
    #[clap(short, long)]
    json_legacy: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Sub {
    /// build a genesis file from the coordination git repo
    Build {
        /// github args
        #[clap(flatten)]
        github: GithubArgs,

        /// Ark B
        #[clap(long)]
        drop_list: Option<PathBuf>,
    }, // just do genesis without wizard
    /// register to the genesis coordination git repository
    Register {
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
        /// ordered list of dns/ip with port for alice..dave
        /// use 6180 for production validator port
        #[clap(long)]
        host_list: Vec<HostAndPort>,
        /// path to the Move framework file, usually ./framework/releases/head.mrb
        #[clap(short, long)]
        framework_mrb_path: PathBuf,
        /// path to file for legacy migration file
        #[clap(short, long)]
        json_legacy: Option<PathBuf>,
    },
}
