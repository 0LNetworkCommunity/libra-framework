//!  A simple workflow tool to organize all genesis
//! instead of using many CLI tools.
//! genesis wizard

use crate::{genesis_builder, node_yaml};
///////
// TODO: import from libra
use crate::{genesis_registration, hack_cli_progress::OLProgress};
//////
use crate::github_extensions::LibraGithubClient;
use crate::helpers::MODE_0L;

use anyhow::bail;
use dialoguer::{Confirm, Input};
use dirs;
use indicatif::{ProgressBar, ProgressIterator};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use libra_wallet::{keys::VALIDATOR_FILE, validator_files::SetValidatorConfiguration};
use ol_types::config::AppCfg;
use zapatos_types::chain_id::NamedChain;
use zapatos_config::config::IdentityBlob;
use zapatos_genesis::config::HostAndPort;
use zapatos_github_client::Client;
use std::str::FromStr;


pub const DEFAULT_DATA_PATH: &str = ".libra";
pub const DEFAULT_GIT_BRANCH: &str = "main";
const GITHUB_TOKEN_FILENAME: &str = "github_token.txt";
/// Wizard for genesis
pub struct GenesisWizard {
    /// a name to use only for genesis purposes
    pub username: String,
    /// the github org hosting the genesis repo
    pub genesis_repo_org: String,
    /// name of the repo
    pub repo_name: String,
    /// the registrant's github username
    github_username: String,
    /// the registrant's github api token.
    github_token: String,
    /// the home path of the user
    data_path: PathBuf,
    /// what epoch is the fork happening from
    pub epoch: Option<u64>,
}

impl Default for GenesisWizard {
    /// testnet values for genesis wizard
    fn default() -> Self {
        let data_path = dirs::home_dir()
            .expect("no home dir found")
            .join(DEFAULT_DATA_PATH);

        Self {
            username: "alice".to_string(),
            genesis_repo_org: "0o-de-lally".to_string(),
            // genesis_repo_org: "alice".to_string(),
            repo_name: "a-genesis".to_string(),
            github_username: "".to_string(),
            github_token: "".to_string(),
            data_path,
            epoch: None,
        }
    }
}
impl GenesisWizard {
    /// start wizard for end-to-end genesis
    pub fn start_wizard(&mut self, home_dir: Option<PathBuf>, use_local_framework: bool) -> anyhow::Result<()> {
        if let Some(d) = home_dir {
            self.data_path = d;
        }

        if !Path::exists(&self.data_path) {
            println!(
                "\nIt seems you have no files at {}, creating directory now",
                &self.data_path.display()
            );
            std::fs::create_dir_all(&self.data_path)?;
        }
        // check the git token is as expected, and set it.
        self.git_token_check()?;

        let to_init = Confirm::new()
            .with_prompt(format!(
                "Want to freshen configs at {:?} now?",
                &self.data_path
            ))
            .interact()?;
        if to_init {
            let host = what_host()?;
            initialize_host(Some(self.data_path.clone()), &self.github_username, host, None)?;
        }

        let to_register = Confirm::new()
            .with_prompt("Do you need to register for genesis?")
            .interact()
            .unwrap();

        // check if .0L folder is clean
        if to_register {
            let id = IdentityBlob::from_file(&self.data_path.clone().join(VALIDATOR_FILE))?;

            self.username = id
                .account_address
                .expect(&format!(
                    "cannot find an account address in {}",
                    VALIDATOR_FILE
                ))
                .to_hex_literal();
            // check if the user has the github auth token, and that
            // there is a forked repo on their account.
            // Fork the repo, if it doesn't exist
            self.git_setup()?;

            self.genesis_registration_github()?;

            self.make_pull_request()?
        }

        let ready = Confirm::new()
            .with_prompt("\nNOW WAIT for everyone to do genesis. Is everyone ready?")
            .interact()
            .unwrap();

        if ready {

            // Get Legacy Recovery from file
            let legacy_recovery = vec![];

            // TODO: progress bar is odd when we  ask "already exists, are you sure you want to overwrite"

            // let pb = ProgressBar::new(1000).with_style(OLProgress::spinner());

            // pb.enable_steady_tick(Duration::from_millis(100));

            genesis_builder::build(
                self.genesis_repo_org.clone(),
                self.repo_name.clone(),
                self.github_token.clone(),
                self.data_path.clone(),
                use_local_framework,
                &legacy_recovery,
            )?;
            // pb.finish_and_clear();

            OLProgress::complete("Genesis files built");

            for _ in (0..10)
                .progress_with_style(OLProgress::fun())
                .with_message("Initializing 0L")
            {
                thread::sleep(Duration::from_millis(100));
            }
        } else {
            println!("Please wait for everyone to finish genesis and come back");
        }

        Ok(())
    }

    fn git_token_check(&mut self) -> anyhow::Result<()> {
        let gh_token_path = self.data_path.join(GITHUB_TOKEN_FILENAME);
        if !Path::exists(&gh_token_path) {
            match Input::<String>::new()
                .with_prompt(&format!(
                    "No github token found, enter one now, or save to {}",
                    self.data_path.join(GITHUB_TOKEN_FILENAME).display()
                ))
                .interact_text()
            {
                Ok(s) => {
                    // creates the folders if necessary (this check is called before host init)
                    std::fs::create_dir_all(&self.data_path)?;
                    std::fs::write(&gh_token_path, s)?;
                }
                _ => println!("somehow couldn't read what you typed"),
            }
        }

        self.github_token = std::fs::read_to_string(&gh_token_path)?;
        OLProgress::complete("github token found");

        let temp_gh_client = Client::new(
            self.genesis_repo_org.clone(), // doesn't matter
            self.repo_name.clone(),
            DEFAULT_GIT_BRANCH.to_string(),
            self.github_token.clone(),
        );

        self.github_username = temp_gh_client.get_authenticated_user()?;

        if !Confirm::new()
            .with_prompt(format!(
                "Is this your github user? {} ",
                &self.github_username
            ))
            .interact()?
        {
            println!("Please update your github token");
            return Ok(());
        }

        Ok(())
    }

    fn git_setup(&mut self) -> anyhow::Result<()> {
        let gh_client = Client::new(
            self.genesis_repo_org.clone(),
            self.repo_name.clone(),
            DEFAULT_GIT_BRANCH.to_string(),
            self.github_token.clone(),
        );

        // Use the github token to find out who is the user behind it
        // check if a gitbhub repo was already created.
        let user_gh_client = Client::new(
            self.github_username.clone(),
            self.repo_name.clone(),
            DEFAULT_GIT_BRANCH.to_string(),
            self.github_token.clone(),
        );

        if user_gh_client.get_branches().is_err() {
            match Confirm::new()
                .with_prompt(format!(
                    "Fork the genesis repo to your account? {} ",
                    &self.github_username
                ))
                .interact()
            {
                Ok(true) => gh_client.fork_genesis_repo(&self.genesis_repo_org, &self.repo_name)?,
                _ => bail!("no forked repo on your account, we need it to continue"),
            }
        } else {
            println!("Found a genesis repo on your account, we'll use that for registration.\n");
        }
        // Remeber to clear out the /owner key from the key_store.json for safety.
        Ok(())
    }

    // fn maybe_remove_money_keys(&self, app_cfg: &AppCfg) {
    //     if Confirm::new()
    //         .with_prompt("Remove the money keys from the key store?")
    //         .interact().unwrap()
    //     {
    //         let storage_helper =
    //             StorageHelper::get_with_path(self.data_path.clone());

    //         let mut owner_storage = storage_helper.storage(app_cfg.format_oper_namespace().clone());
    //         owner_storage.set(OWNER_KEY, "").unwrap();
    //         owner_storage.set(OPERATOR_KEY, "").unwrap();

    //         let mut oper_storage = storage_helper.storage(app_cfg.format_oper_namespace().clone());

    //         oper_storage.set(OWNER_KEY, "").unwrap();
    //         oper_storage.set(OPERATOR_KEY, "").unwrap();
    //     }
    // }

    fn genesis_registration_github(&self) -> anyhow::Result<()> {
        let pb = ProgressBar::new(1000).with_style(OLProgress::spinner());
        pb.enable_steady_tick(Duration::from_millis(100));

        genesis_registration::register(
            self.username.clone(),
            self.github_username.clone(), // Do the registration on the fork.
            self.repo_name.clone(),
            self.github_token.clone(),
            self.data_path.clone(),
        )?;
        pb.finish_and_clear();

        OLProgress::complete("Registered to genesis on github.");

        Ok(())
    }

    fn _download_snapshot(&mut self, _app_cfg: &AppCfg) -> anyhow::Result<PathBuf> {
        if let Some(e) = self.epoch {
            if !Confirm::new()
                .with_prompt(&format!("So are we migrating data from epoch {}?", e))
                .interact()
                .unwrap()
            {
                bail!("Please specify the epoch you want to migrate from.")
            }
        } else {
            self.epoch = Input::new()
                .with_prompt("What epoch are we migrating from? ")
                .interact_text()
                .ok();
            // .map(|epoch| epoch.parse().unwrap()).ok();
        }

        let pb = ProgressBar::new(1000).with_style(OLProgress::spinner());

        pb.enable_steady_tick(Duration::from_millis(100));

        // // All we are doing is download the snapshot from github.
        // let backup = Backup::new(self.epoch, app_cfg);

        // if backup.manifest_path().is_err() {
        //     backup.fetch_backup(true)?;
        // } else {
        //     println!("Already have snapshot for epoch {}", self.epoch.unwrap());
        // }

        // I changed the manifest file name to state.manifest instead of epoch_ending.manifest
        // let snapshot_manifest_file = backup.manifest_path()?;

        // let snapshot_dir = snapshot_manifest_file.parent().unwrap().to_path_buf();

        // hack
        let snapshot_dir = PathBuf::new();

        pb.finish_and_clear();
        Ok(snapshot_dir)
    }

    fn make_pull_request(&self) -> anyhow::Result<()> {
        let gh_token_path = self.data_path.join(GITHUB_TOKEN_FILENAME);
        let api_token = std::fs::read_to_string(&gh_token_path)?;

        let pb = ProgressBar::new(1).with_style(OLProgress::bar());
        let gh_client = Client::new(
            self.genesis_repo_org.clone(),
            self.repo_name.clone(),
            DEFAULT_GIT_BRANCH.to_string(),
            api_token.clone(),
        );
        // repository_owner, genesis_repo_name, username
        // This will also fail if there already is a pull request!
        match gh_client.make_genesis_pull_request(
            &*self.genesis_repo_org,
            &*self.repo_name,
            &*self.github_username,
        ) {
            Ok(_) => println!("created pull request to genesis repo"),
            Err(_) => println!("failed to create pull request to genesis repo: do you already have an open PR? If so, you don't need to do anything else."),
        };
        pb.inc(1);
        pb.finish_and_clear();
        Ok(())
    }

    fn _maybe_backup_db(&self) {
        // ask to empty the DB
        if self.data_path.join("db").exists() {
            println!("We found a /db directory. Can't do genesis with a non-empty db.");
            if Confirm::new()
                .with_prompt("Let's move the old /db to /db_bak_<date>?")
                .interact()
                .unwrap()
            {
                let date_str = chrono::Utc::now().format("%Y-%m-%d-%H-%M").to_string();
                fs::rename(
                    self.data_path.join("db"),
                    self.data_path.join(format!("db_bak_{}", date_str)),
                )
                .expect("failed to move db to db_bak");
            }
        }
    }
}

fn initialize_host(
    home_path: Option<PathBuf>,
    username: &str,
    host: HostAndPort,
    mnem: Option<String>,
) -> anyhow::Result<()> {
    libra_wallet::keys::refresh_validator_files(mnem, home_path.clone())?;
    OLProgress::complete("Initialized validator key files");
    // TODO: set validator fullnode configs. Not NONE
    SetValidatorConfiguration::new(home_path.clone(), username.to_owned(), host, None)
        .set_config_files()?;
    OLProgress::complete("Saved genesis registration files locally");

    node_yaml::save_validator_yaml(home_path)?;
    OLProgress::complete("Saved validator node yaml file locally");
    Ok(())
}


/// interact with user to get ip address
pub fn what_host() -> Result<HostAndPort, anyhow::Error> {
    // get from external source since many cloud providers show different interfaces for `machine_ip`
    let resp = reqwest::blocking::get("https://ifconfig.me")?;
    // let ip_str = resp.text()?;

     let host = match resp.text() {
       Ok(ip_str) => { 
          let h = HostAndPort::from_str(&format!("{}:6180", ip_str))?;
          if *MODE_0L == NamedChain::DEVNET { return Ok(h) }
          Some(h)
        }
        _ => None
     };


    if let Some(h) = host {
          let txt = &format!(
        "Will you use this host, and this IP address {:?}, for your node?",
        h
      );
      if Confirm::new().with_prompt(txt).interact().unwrap() {
          return Ok(h)
      }
    };


    let input: String = Input::new()
                .with_prompt("Enter the DNS or IP address, with port 6180")
                .interact_text()
                .unwrap();
    let ip = input
      .parse::<HostAndPort>()
      .expect("Could not parse IP or DNS address");

    Ok(ip)
}

#[test]
#[ignore]

fn test_wizard() {
    let mut wizard = GenesisWizard::default();
    wizard.start_wizard(None, false).unwrap();
}

#[test]
fn test_validator_files_config() {
    let alice_mnem = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse".to_string();
    let h = HostAndPort::local(6180).unwrap();
    let test_path = dirs::home_dir()
        .unwrap()
        .join(DEFAULT_DATA_PATH)
        .join("test_genesis");
    if test_path.exists() {
        fs::remove_dir_all(&test_path).unwrap();
    }

    initialize_host(Some(test_path.clone()), "validator", h, Some(alice_mnem)).unwrap();

    fs::remove_dir_all(&test_path).unwrap();
}

#[test]
fn test_register() {
    let mut g = GenesisWizard::default();
    g.username = "0xTEST".to_string();
    g.git_token_check().unwrap();
    g.genesis_registration_github().unwrap();
}


