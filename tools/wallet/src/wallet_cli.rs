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
    Legacy,
    Whoami(WhoamiOpts)
}

#[derive(Args, Debug)]
struct WhoamiOpts {
    ///  display private keys
    #[clap(short('p'), long)]
    display_private: bool,

    #[clap(short, long)]
    /// save keyscheme private keys to file
    output_path: Option<PathBuf>,
}

impl WalletCli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            WalletSub::Whoami(args) => {

                let l = get_keys_from_prompt()?;

                if let Some(dir) = &args.output_path {
                    l.save_keys(dir)?;
                }

                l.display(args.display_private);
            }
            WalletSub::Legacy => {
              println!("this command will generate legacy keys and addresses from v5 addresses. You should only be using this for testing or debugging purposes");

              account_keys::legacy_keygen(true)?;

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
