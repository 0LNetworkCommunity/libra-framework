use anyhow::{Context, bail};
use clap::{Parser, Subcommand, Args};
use libra_genesis_tools::{
    genesis_builder, parse_json,
    supply::SupplySettings,
    wizard::{GenesisWizard, GITHUB_TOKEN_FILENAME},
};
use libra_types::{exports::NamedChain, global_config_dir, legacy_types::fixtures::TestPersona};
use zapatos_genesis::config::{ValidatorConfiguration, HostAndPort};
use std::{path::PathBuf, net::Ipv4Addr};

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
        #[clap(flatten)]
        /// optional, settings for supply.
        supply_settings: SupplySettings,
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
        #[clap(flatten)]
        /// optional, settings for supply.
        supply_settings: SupplySettings,
    },
    /// sensible defaults for testnet, does not need a genesis repo
    /// accounts are created from fixture mnemonics for alice, bob, carol, dave
    Testnet {

      /// which persona is this machine going to register as
      #[clap(short, long)]
      me: TestPersona,

      /// list of IP addresses of each persona Alice, Bob, Carol, Dave
      #[clap(short, long)]
      ip_list: Vec<HostAndPort>,

    }
}

fn main() -> anyhow::Result<()> {
    let cli = GenesisCliArgs::parse();
    match cli.command {
        Some(Sub::Genesis {
          github,
          supply_settings
        }) => {

            let data_path = cli.home_dir.unwrap_or_else(global_config_dir);

            let github_token = github.token_github.unwrap_or(
                std::fs::read_to_string(data_path.join(GITHUB_TOKEN_FILENAME))
                    .context("cannot find github_token.txt in config path")?
                    .trim()
                    .to_string(),
            );

            let recovery = if let Some(p) = github.json_legacy {
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
                Some(&recovery),
                Some(supply_settings),
                cli.chain.unwrap_or(NamedChain::TESTING),
                None,
            )?;
        }
        Some(Sub::Register {
          github
        }) => {
            GenesisWizard::new(
                github.org_github,
                github.name_github,
                cli.home_dir,
                cli.chain.unwrap_or(NamedChain::TESTING),
            )
            .start_wizard(github.local_framework, github.json_legacy, false, None)?;
        }
        Some(Sub::Wizard {
          github,
          supply_settings
        }) => {
            GenesisWizard::new(
                github.org_github,
                github.name_github,
                cli.home_dir,
                cli.chain.unwrap_or(NamedChain::TESTING),
            )
            .start_wizard(
                github.local_framework,
                github.json_legacy,
                true,
                Some(supply_settings),
            )?;
        },
        Some(Sub::Testnet {
          me,
          ip_list
        }) => {
            let data_path = cli.home_dir.unwrap_or_else(global_config_dir);

            // TODO: make validator config here
            // testnet_validator_config
            let val_cfg: Vec<ValidatorConfiguration> = ip_list.iter()
              .enumerate()
              .filter_map(|(idx, host)| {
                let p = TestPersona::from(idx).ok()?;
                genesis_builder::testnet_validator_config(&p, host).ok()
              })
              .collect();

            // ip_list.iter()
            //   .enumerate()
            //   .for_each(|(idx, host)| {
            //     let p = TestPersona::from(idx).ok().unwrap();
            //     dbg!(&p);
            //     let c = genesis_builder::testnet_validator_config(&p, host).ok();
            //     dbg!(&c);
            //   });
            //   // .collect();

          let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sample_end_user_single.json");

          let recovery = parse_json::recovery_file_parse(p).unwrap();


            let recovery = vec![];
            genesis_builder::build(
                "none".to_string(), // when is testnet is ignored
                "none".to_string(),
                "none".to_string(),
                data_path,
                true,
                Some(&recovery),
                Some(SupplySettings::default()),
                cli.chain.unwrap_or(NamedChain::TESTING),
                Some(val_cfg)
            )?;
        },
        _ => {
            println!("\nIf you're looking for trouble \nYou came to the right place");
        }
    }
    Ok(())
}
