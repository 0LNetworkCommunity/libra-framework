use crate::core::{backlog, proof};
use clap::{Parser, Subcommand};
use libra_types::exports::Client;
use libra_types::exports::{Ed25519PrivateKey, ValidCryptoMaterialStringExt};
use libra_types::legacy_types::app_cfg::AppCfg;
use libra_types::type_extensions::client_ext::ClientExt;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]

/// clap struct entry point for the tower cli
pub struct TowerCli {
    #[clap(subcommand)]
    /// the subcommand
    pub command: TowerSub,
    /// If the node is offline and tower needs to run in local mode
    /// without querying chain
    #[clap(short, long)]
    pub local_mode: bool,
    /// The optional path to an alternate path besides $HOME/.0L
    #[clap(short, long)]
    pub config_file: Option<PathBuf>,
    /// nickname of the profile to use, if there is more than one. Defaults to first.
    #[clap(short, long)]
    pub profile: Option<String>,

    /// optional, private key of the account. Otherwise this will prompt for mnemonic. Warning: intended for testing.
    #[clap(short, long)]
    pub test_private_key: Option<String>,
}

#[derive(Subcommand)]
pub enum TowerSub {
    Backlog {
        /// Just show the backlog of proofs not submitted
        #[clap(short, long)]
        show: bool,
    },
    Start,
    Once,
    Zero,
}

impl TowerCli {
    pub async fn run(&self) -> anyhow::Result<()> {
        // let cli = TowerCli::parse();

        let mut app_cfg = AppCfg::load(self.config_file.clone())?;

        if let Some(pk) = &self.test_private_key {
            let profile = app_cfg.get_profile_mut(self.profile.clone())?;
            profile.test_private_key = Some(Ed25519PrivateKey::from_encoded_string(pk)?);
        };

        match self.command {
            TowerSub::Backlog { show } => {
                println!("backlog");
                if show {
                    backlog::show_backlog(&app_cfg).await?;
                } else {
                    backlog::process_backlog(&app_cfg).await?;
                }
            }
            TowerSub::Start => {
                proof::mine_and_submit(&mut app_cfg, self.local_mode).await?;
            }
            TowerSub::Once => {
                let (client, _) = Client::from_libra_config(&app_cfg, None).await?;
                proof::get_next_and_mine(&app_cfg, &client, self.local_mode).await?;
            }
            TowerSub::Zero => {
                proof::write_genesis(&app_cfg)?;
            }
        }

        Ok(())
    }
}
