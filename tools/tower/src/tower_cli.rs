use crate::core::delay::verify;
use crate::core::next_proof::NextProof;
use crate::core::{backlog, proof};
use clap::{Parser, Subcommand};
use libra_types::exports::Client;
use libra_types::exports::{Ed25519PrivateKey, ValidCryptoMaterialStringExt};
use libra_types::legacy_types::app_cfg::AppCfg;
use libra_types::legacy_types::app_cfg::Profile;
use libra_types::legacy_types::block::VDFProof;
use libra_types::legacy_types::vdf_difficulty::VDFDifficulty;
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
    Verify {
      file: PathBuf,
    },
    #[clap(hide(true))]
    Debug {
        #[clap(long)]
        height: u64,
        #[clap(long)]
        preimage: String,
        #[clap(long)]
        difficulty: u64,
        #[clap(long)]
        security: u64,
        #[clap(long)]
        dir: PathBuf,
    },
}

impl TowerCli {
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut app_cfg = AppCfg::load(self.config_file.clone())?;

        let profile = app_cfg.get_profile_mut(self.profile.clone())?;

        if let Some(pk) = &self.test_private_key {
            profile.set_private_key(&Ed25519PrivateKey::from_encoded_string(pk)?);
        };

        match &self.command {
            TowerSub::Backlog { show } => {
                println!("backlog");
                if *show {
                    backlog::show_backlog(&app_cfg).await?;
                } else {
                    if profile.borrow_private_key().is_err() {
                        prompt_private_key(profile)?;
                    }

                    backlog::process_backlog(&app_cfg).await?;
                }
            }
            TowerSub::Start => {
                if profile.borrow_private_key().is_err() {
                    prompt_private_key(profile)?;
                }
                proof::mine_and_submit(&mut app_cfg, self.local_mode).await?;
            }
            TowerSub::Once => {
                let (client, _) = Client::from_libra_config(&app_cfg, None).await?;
                proof::get_next_and_mine(&app_cfg, &client, self.local_mode).await?;
            }
            TowerSub::Zero => {
                proof::write_genesis(&app_cfg)?;
            }
            TowerSub::Verify { file } => {
              let block = VDFProof::parse_block_file(&file, false)?;
              match verify(&block.preimage, &block.proof, block.difficulty(), block.security() as u16, true) {
                true => println!("SUCCESS: valid proof"),
                false => println!("FAIL: proof is invalid"),
              };
            }
            TowerSub::Debug {
                height,
                difficulty,
                security,
                preimage,
                dir,
            } => {
                let params = NextProof {
                    preimage: hex::decode(preimage)?,
                    next_height: *height,
                    diff: VDFDifficulty {
                        difficulty: *difficulty,
                        security: *security,
                        prev_diff: 0,
                        prev_sec: 0,
                    },
                };
                proof::mine_once(dir, &params)?;
            }
        }

        Ok(())
    }
}

// for any long running operations requiring the private key in memory.
fn prompt_private_key(cfg: &mut Profile) -> anyhow::Result<()> {
    let leg_keys = libra_wallet::account_keys::get_keys_from_prompt()?;
    cfg.set_private_key(&leg_keys.child_0_owner.pri_key);
    Ok(())
}
