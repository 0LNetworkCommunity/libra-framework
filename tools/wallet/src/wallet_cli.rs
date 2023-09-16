use crate::account_keys::{self, get_keys_from_prompt};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
/// Generate keys, key schemes, and save key files.
pub struct WalletCli {
    #[clap(subcommand)]
    command: WalletSub,
}

#[derive(Subcommand)]
enum WalletSub {
    /// Generate keys and account address locally
    Keygen {
        /// Recover account from the given mnemonic
        #[clap(short, long)]
        mnemonic: Option<String>,

        /// Path of the directory to store yaml files
        #[clap(short, long)]
        output_dir: Option<String>,
    },
    /// Use the legacy key derivation scheme
    // TODO: call this 'WalletArgs'
    Legacy(LegArgs),
    // TODO: add WhoAmI to display the wallet info.
}

#[derive(Args, Debug)]
struct LegArgs {
    ///  display private keys and authentication keys
    #[clap(short, long)]
    display: bool,

    #[clap(short, long)]
    /// save legacy keyscheme private keys to file
    output_path: Option<PathBuf>,
    /// generate new keys and mnemonic in legacy format. It's not clear why you need this besides for testing. Note: these are not useful for creating a validator.
    #[clap(short, long)]
    keygen: bool,
}

impl WalletCli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            WalletSub::Legacy(args) => {
                if !args.display && args.output_path.is_none() {
                    println!("pass --display to show keys and/or --output-path to save keys");
                    return Ok(());
                }

                let l = if args.keygen {
                    account_keys::legacy_keygen(true)?
                } else {
                    get_keys_from_prompt()?
                };

                if let Some(dir) = &args.output_path {
                    l.save_keys(dir)?;
                }

                if args.display {
                    l.display();
                }
            }
            WalletSub::Keygen {
                mnemonic,
                output_dir,
            } => {
                println!(
                    "{}",
                    crate::key_gen::run(
                        mnemonic.to_owned(),
                        output_dir.as_ref().map(PathBuf::from)
                    )
                    .await?
                );
            }
        }
        Ok(())
    }
}
