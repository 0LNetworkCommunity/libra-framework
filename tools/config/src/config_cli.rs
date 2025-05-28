use crate::{
    interactive, config_wizard,
    get_genesis_artifacts::{download_genesis, get_genesis_waypoint},
    make_yaml_public_fullnode::init_fullnode_yaml,
    validator_config::{validator_dialogue, vfn_dialogue},
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use libra_types::{
    core_types::network_playlist::NetworkPlaylist,
    exports::{AccountAddress, AuthenticationKey, NamedChain},
    global_config_dir, ol_progress,
};
use libra_wallet::{utils::read_operator_file, validator_files::OPERATOR_FILE};
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]
/// Generate a libra config file in the home .libra directory
pub struct ConfigCli {
    #[clap(subcommand)]
    subcommand: Option<ConfigSub>,
    /// Path for configs if not the default $HOME/.libra
    #[clap(short('f'), long)]
    path: Option<PathBuf>,
    /// optional. Which network to use as the default. Defaults to MAINNET other options: TESTNET, TESTING, DEVNET
    #[clap(short, long)]
    chain_name: Option<NamedChain>,
    /// optional. Name of the profile
    #[clap(short, long)]
    profile: Option<String>,
}

#[derive(clap::Subcommand)]
enum ConfigSub {
    /// Generates a libra-cli-config.yaml for cli tools like `txs`, `query`, etc.  Note: the file can also be used for Carpe, though that app uses a different default directory than these cli tools.
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

        #[clap(long)]
        /// override the default fullnodes URLs
        fullnode_url: Option<Url>,
        /// a URL for a network playlist to load default nodes from
        #[clap(long, conflicts_with("fullnode_url"))]
        playlist_url: Option<Url>,
    },
    /// replace API URL, reset an address, remove a profile.
    Fix {
        /// optional, reset the address from mnemonic. Will also lookup on the chain for the actual address if you forgot it, or rotated your authkey.
        #[clap(short('a'), long)]
        reset_address: bool,

        #[clap(short, long)]
        remove_profile: Option<String>,

        /// optional, force overwrite of all urls in current network profile to this url
        #[clap(short('u'), long)]
        fullnode_url: Option<Url>,
    },
    /// Show the addresses and configs on this device
    View {},

    // COMMIT NOTE: we haven't used vendor tooling configs for anything.
    /// Generate validators' config file
    ValidatorInit {
        // just make the VFN file
        #[clap(short, long)]
        vfn: bool,
    },

    /// Generate a fullnode dir and add fullnode.yaml from template
    FullnodeInit {
        /// path to libra config and data files defaults to $HOME/.libra
        #[clap(long)]
        home_path: Option<PathBuf>,
    },
}

impl ConfigCli {
    /// Executes the appropriate subcommand based on user input.
    pub async fn run(&self) -> Result<()> {
        match &self.subcommand {
            Some(ConfigSub::Fix {
                reset_address,
                remove_profile,
                fullnode_url: force_url,
            }) => {
                interactive::fix_config(interactive::FixOptions {
                    reset_address: *reset_address,
                    remove_profile: remove_profile.clone(),
                    fullnode_url: force_url.clone(),
                    config_path: self.path.clone(),
                    chain_name: self.chain_name,
                })
                .await
            }

            // Initialize configuration wizard
            Some(ConfigSub::Init {
                force_address,
                force_authkey,
                test_private_key,
                fullnode_url: url,
                playlist_url,
            }) => {
                let mut playlist = None;

                if url.is_some() {
                    if playlist_url.is_some() {
                        bail!("cannot use both --force-url and --playlist-url");
                    }
                    if self.chain_name.is_none() {
                        bail!("--force-url requires --chain-name");
                    }
                    playlist = Some(NetworkPlaylist::new(url.clone(), self.chain_name));
                }

                config_wizard::wizard(
                    force_authkey.to_owned(),
                    force_address.to_owned(),
                    self.path.to_owned(),
                    self.chain_name.to_owned(),
                    test_private_key.to_owned(),
                    playlist_url.to_owned(),
                    playlist,
                )
                .await?;

                Ok(())
            }

            // Initialize validator configuration
            Some(ConfigSub::ValidatorInit { vfn }) => {
                let home_dir = self.path.clone().unwrap_or_else(global_config_dir);
                if *vfn {
                    vfn_dialogue(&home_dir, None, None).await?;
                    return Ok(());
                }

                let data_path = global_config_dir();
                if !&data_path.exists() {
                    println!(
                        "\nIt seems you have no files at {}, creating directory now",
                        data_path.display()
                    );
                    std::fs::create_dir_all(&data_path)?;
                }
                // Download genesis block and initialize validators' configuration
                download_genesis(Some(data_path.clone())).await?;
                let _ = get_genesis_waypoint(Some(data_path.clone())).await?;
                validator_dialogue(&data_path, None, self.chain_name).await?;
                println!("Validators' config initialized.");
                Ok(())
            }

            // View validator and network configurations
            Some(ConfigSub::View {}) => {
                let home_dir = self.path.clone().unwrap_or_else(global_config_dir);

                let public_keys_file = home_dir.join(OPERATOR_FILE);

                let public_identity = read_operator_file(public_keys_file.as_path())?;
                println!("validator public credentials:");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&public_identity).unwrap()
                );

                println!("network addresses:");
                let validator_net = public_identity.validator_host;
                let net_addr = validator_net
                    .as_network_address(public_identity.validator_network_public_key)?;
                println!(
                    "validator: {}",
                    serde_json::to_string_pretty(&net_addr).unwrap()
                );

                if let Some(fn_host) = public_identity.full_node_host {
                    let net_addr_fn = fn_host.as_network_address(
                        public_identity
                            .full_node_network_public_key
                            .context("expected a full_node_network_public_key in operator.yaml")?,
                    )?;

                    println!(
                        "vfn: {}",
                        serde_json::to_string_pretty(&net_addr_fn).unwrap()
                    );
                } else {
                    println!("WARN: no config information found for Validator Full Node (VFN)")
                }

                println!(
                    "\nNOTE: to check if this matches your mnemonic try `libra wallet whoami`"
                );

                Ok(())
            }

            // Initialize fullnode configuration
            Some(ConfigSub::FullnodeInit { home_path }) => {
                download_genesis(home_path.to_owned()).await?;
                println!("downloaded genesis block");

                let p = init_fullnode_yaml(home_path.to_owned(), true).await?;

                println!("config created at {}", p.display());

                ol_progress::OLProgress::complete("fullnode configs initialized");

                Ok(())
            }
            _ => {
                println!("Sometimes I'm right and I can be wrong. My own beliefs are in my song. The butcher, the banker, the drummer and then. Makes no difference what group I'm in.");

                Ok(())
            }
        }
    }
}
