use crate::make_yaml_public_fullnode::{download_genesis, init_fullnode_yaml};
use crate::validator_config::{validator_dialogue, vfn_dialogue};
use crate::{legacy_config, make_profile};
use anyhow::{Context, Result};
use clap::Parser;
use libra_types::exports::AccountAddress;
use libra_types::exports::AuthenticationKey;
use libra_types::exports::Client;
use libra_types::exports::NamedChain;
use libra_types::legacy_types::app_cfg::{self, AppCfg};
use libra_types::type_extensions::client_ext::ClientExt;
use libra_types::{global_config_dir, ol_progress};
use libra_wallet::utils::read_operator_file;
use libra_wallet::validator_files::OPERATOR_FILE;
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
    // TODO: add WhoAmI to show libra.yaml profile info.
    /// try to add for fix the libra.yaml file
    #[clap(arg_required_else_help(true))]
    Fix {
        /// optional, reset the address from mnemonic. Will also lookup on the chain for the actual address if you forgot it, or rotated your authkey.
        #[clap(short, long)]
        address: bool,

        #[clap(short, long)]
        remove_profile: Option<String>,

        /// optional, force overwrite of all urls in current network profile to this url
        #[clap(short('u'), long)]
        force_url: Option<Url>,
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
    ValidatorInit {
        /// check the files generated
        #[clap(short, long)]
        check: bool,
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
    pub async fn run(&self) -> Result<()> {
        match &self.subcommand {
            Some(ConfigSub::VendorInit {
                public_key,
                profile,
                workspace,
            }) => make_profile::run(public_key, profile.as_deref().to_owned(), *workspace).await,
            Some(ConfigSub::Fix {
                address,
                remove_profile,
                force_url,
            }) => {
                let mut cfg = AppCfg::load(self.path.clone())?;

                if *address {
                    let mut account_keys = legacy_config::prompt_for_account()?;

                    let client = Client::new(cfg.pick_url(self.chain_name)?);

                    if client.get_index().await.is_ok() {
                        account_keys.account = match client
                            .lookup_originating_address(account_keys.auth_key)
                            .await
                        {
                            Ok(r) => r,
                            _ => {
                                println!("This looks like a new account, and it's not yet on chain. If this is not what you expected, are you sure you are using the correct recovery mnemonic?");
                                // do nothing
                                account_keys.account
                            }
                        };
                    };

                    let profile =
                        app_cfg::Profile::new(account_keys.auth_key, account_keys.account);

                    if dialoguer::Confirm::new()
                        .with_prompt("set as default profile?")
                        .interact()?
                    {
                        cfg.workspace
                            .set_default(account_keys.account.to_hex_literal());
                    }

                    cfg.maybe_add_profile(profile)?;
                }

                if let Some(p) = remove_profile {
                    let r = cfg.try_remove_profile(p);
                    if r.is_err() {
                        println!("no profile found matching {}", &p)
                    }
                }

                if let Some(u) = force_url {
                    let np = cfg.get_network_profile_mut(self.chain_name)?;
                    np.nodes = vec![];
                    np.add_url(u.to_owned());
                }
                cfg.save_file()?;
                Ok(())
            }
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
                    None,
                )
                .await?;

                Ok(())
            }
            Some(ConfigSub::ValidatorInit { check, vfn }) => {
                let home_dir = self.path.clone().unwrap_or_else(global_config_dir);
                if *vfn {
                    vfn_dialogue(&home_dir, None, None).await?;
                    return Ok(());
                }
                if *check {
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
                            public_identity.full_node_network_public_key.context(
                                "expected a full_node_network_public_key in operator.yaml",
                            )?,
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
                validator_dialogue(&data_path, None).await?;
                println!("Validators' config initialized.");
                Ok(())
            }
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
