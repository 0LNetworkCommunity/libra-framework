use crate::{account_keys, whoami::who_am_i};

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
    /// use mnemonic to see what account keys are generated
    Whoami(WhoamiOpts),
}

#[derive(Args, Debug)]
struct WhoamiOpts {
    ///  show the validator configurations
    #[clap(short('v'), long, default_value = "false")]
    show_validator: bool,

    ///  show the validator configurations
    #[clap(short('l'), long, default_value = "false")]
    legacy_address: bool,

    #[clap(short('m'), long)]
    mnemonic: Option<String>,
}

impl WalletCli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            WalletSub::Whoami(args) => {
                who_am_i(args.legacy_address, args.mnemonic.clone(), args.show_validator)?;
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
