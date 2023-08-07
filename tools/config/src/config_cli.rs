use crate::host::{initialize_validator_configs};
use crate::{legacy_config, make_profile};
use anyhow::Result;
use clap::Parser;
use libra_types::exports::AccountAddress;
use libra_types::exports::AuthenticationKey;
use libra_types::exports::NamedChain;
use libra_types::global_config_dir;
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
/// Generate a libra config file in the home .libra directory
pub struct ConfigCli {
    #[clap(subcommand)]
    subcommand: Option<ConfigSub>,
    /// Path for configs if not the default $HOME/.libra
    #[clap(short, long)]
    path: Option<PathBuf>,
    /// optional. Which network to use as the default. Defaults to MAINNET other options: TESTNET, TESTING, DEVNET
    #[clap(short, long)]
    chain_name: Option<NamedChain>,
}

#[derive(clap::Subcommand)]
enum ConfigSub {
    /// Generates a libra.yaml for cli tools like txs, tower, etc.  Note: the file can also be used for Carpe, though that app uses a different default directory than these cli tools.
    Init {
        /// force an account address instead of reading from mnemonic, requires --force_authkey
        #[clap(long)]
        force_address: Option<AccountAddress>,
        /// force an authkey instead of reading from mnemonic, requires --force_address
        #[clap(long)]
        force_authkey: Option<AuthenticationKey>,
        /// use a private key to initialize. Warning: intended for testing only.
        #[clap(long)]
        test_private_key: Option<String>,
        /// optional. A URL for a network playlist to load default nodes from
        #[clap(long)]
        playlist_url: Option<Url>,
    },
    /// For core developers. Generates a config.yaml in the vendor format. This is a hidden command in the CLI.
    #[clap(hide(true))]
    VendorInit {
        /// Ed25519 public key
        #[clap(short, long)]
        public_key: String,

        /// Profile name to use when saving the config. Defaults to "default"
        ///
        /// This will be used to override associated settings such as
        /// the REST URL, and the private key arguments.
        #[clap(long)]
        profile: Option<String>,

        /// In libra we default to the configs being global in $HOME/.libra
        /// Otherwise you should pass -w to use the workspace configuration.
        /// Uses this directory as the workspace, instead of using the global
        /// parameters in $HOME/.libra
        #[clap(short, long)]
        workspace: bool,
    },
    /// Generate validators' config file
    ValidatorInit {},
}

impl ConfigCli {
    pub async fn run(&self) -> Result<()> {
        match &self.subcommand {
            Some(ConfigSub::VendorInit {
                public_key,
                profile,
                workspace,
            }) => make_profile::run(public_key, profile.as_deref().to_owned(), *workspace).await,
            Some(ConfigSub::Init {
                force_address,
                force_authkey,
                test_private_key,
                playlist_url,
            }) => {
                legacy_config::wizard(
                    force_authkey.to_owned(),
                    force_address.to_owned(),
                    self.path.to_owned(),
                    self.chain_name.to_owned(),
                    test_private_key.to_owned(),
                    playlist_url.to_owned(),
                )
                .await?;

                Ok(())
            }
            Some(ConfigSub::ValidatorInit {}) => {
                let data_path = global_config_dir();
                if !&data_path.exists() {
                    println!(
                        "\nIt seems you have no files at {}, creating directory now",
                        data_path.display()
                    );
                    std::fs::create_dir_all(&data_path)?;
                }
                initialize_validator_configs(&data_path, None)?;
                println!("Validators' config initialized.");
                Ok(())
            }
            _ => {
                println!("Sometimes I'm right and I can be wrong. My own beliefs are in my song. The butcher, the banker, the drummer and then. Makes no difference what group I'm in.");

                Ok(())
            }
        }
    }
}
