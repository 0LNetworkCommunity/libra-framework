use clap::{Parser, Subcommand};
use libra_types::legacy_types::app_cfg::AppCfg;
use libra_types::exports::Client;
use libra_types::type_extensions::client_ext::ClientExt;
use std::path::PathBuf;
use crate::core::{proof, backlog};

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]

/// clap struct entry point for the tower cli
pub struct TowerCli {
    #[clap(subcommand)]
    command: TowerSub,
    /// If the node is offline and tower needs to run in local mode
    /// without querying chain
    #[clap(short,long)]
    local_mode: bool,
    /// The optional path to an alternate path besides $HOME/.0L
    #[clap(short,long)]
    config_file: Option<PathBuf>,
    /// nickname of the profile to use, if there is more than one. Defaults to first.
    #[clap(short,long)]
    profile: Option<String>,
}

#[derive(Subcommand)]
enum TowerSub {
  Backlog { 
    /// Just show the backlog of proofs not submitted
    #[clap(short,long)]
    show: bool 
  },
  Start,
  Once,
  Zero,
}

impl TowerCli {
    pub async fn run(&self) -> anyhow::Result<()>{
      let cli = TowerCli::parse();

      let mut app_cfg = AppCfg::load(cli.config_file)?;

      match cli.command {
        TowerSub::Backlog { show } => {
          println!("backlog");
          if show {
            backlog::show_backlog(&app_cfg).await?;
          } else {
            backlog::process_backlog(&app_cfg).await?;
          }
        },
        TowerSub::Start => {
          proof::mine_and_submit(&mut app_cfg, cli.local_mode).await?;
        },
        TowerSub::Once => {
          let (client, _) = Client::from_libra_config(&app_cfg, None).await?;
          proof::get_next_and_mine(&app_cfg, &client, cli.local_mode).await?;
        },
        TowerSub::Zero => {
          proof::write_genesis(&app_cfg)?;
        },
      }
      
      Ok(())
    }
}