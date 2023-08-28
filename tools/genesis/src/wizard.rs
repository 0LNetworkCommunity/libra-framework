//!  A simple workflow tool to organize all genesis
//! instead of using many CLI tools.
//! genesis wizard

use crate::{genesis_builder, parse_json, supply::SupplySettings};
///////
// TODO: import from libra
use crate::genesis_registration;
use libra_types::ol_progress::OLProgress;
use zapatos_types::chain_id::NamedChain;
//////
use crate::github_extensions::LibraGithubClient;
use anyhow::{bail, Context};
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressIterator};
use libra_types::global_config_dir;

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use libra_types::legacy_types::app_cfg::AppCfg;
use libra_wallet::keys::VALIDATOR_FILE;
use zapatos_config::config::IdentityBlob;
use zapatos_github_client::Client;

use libra_config::host::initialize_validator_configs;

pub const DEFAULT_GIT_BRANCH: &str = "main";
pub const GITHUB_TOKEN_FILENAME: &str = "github_token.txt";
/// Wizard for genesis
#[derive(Debug, Clone)]
pub struct GenesisWizard {
    /// the validator address only for genesis purposes
    pub validator_address: String,
    /// the github org hosting the genesis repo
    pub genesis_repo_org: String,
    /// name of the repo
    pub repo_name: String,
    /// the registrant's github username
    pub github_username: String,
    /// the registrant's github api token.
    pub github_token: String,
    /// the home path of the user
    pub data_path: PathBuf,
    // TODO: remove
    /// what epoch is the fork happening from
    pub epoch: Option<u64>,
    /// what epoch is the fork happening from
    pub chain: NamedChain,
}

impl GenesisWizard {
    /// constructor
    pub fn new(
        genesis_repo_org: String,
        repo_name: String,
        data_path: Option<PathBuf>,
        chain: NamedChain,
    ) -> Self {
        let data_path = data_path.unwrap_or_else(global_config_dir);

        Self {
            validator_address: "tbd".to_string(),
            genesis_repo_org,
            repo_name,
            github_username: "".to_string(),
            github_token: "".to_string(),
            data_path,
            epoch: None,
            chain, // defaults to testing.
        }
    }

    /// start wizard for end-to-end genesis
    pub fn start_wizard(
        &mut self,
        use_local_framework: bool,
        legacy_recovery_path: Option<PathBuf>,
        do_genesis: bool,
        supply_settings: Option<SupplySettings>,
    ) -> anyhow::Result<()> {
        if !Path::exists(&self.data_path) {
            println!(
                "\nIt seems you have no files at {}, creating directory now",
                &self.data_path.display()
            );
            std::fs::create_dir_all(&self.data_path)?;
        }
        // check the git token is as expected, and set it.
        self.git_token_check()?;

        match initialize_validator_configs(&self.data_path, Some(&self.github_username)) {
            Ok(_) => {
                println!("Validators' config initialized!");
            }
            Err(error) => {
                eprintln!("Error in initializing validators' config: {}", error);
            }
        }

        let to_register = Confirm::new()
            .with_prompt("Do you need to register for genesis?")
            .interact()
            .unwrap();

        // check if .0L folder is clean
        if to_register {
            let id = IdentityBlob::from_file(&self.data_path.clone().join(VALIDATOR_FILE))?;

            self.validator_address = id
                .account_address
                .context(format!(
                    "cannot find an account address in {}",
                    VALIDATOR_FILE
                ))?
                .to_hex_literal();
            // check if the user has the github auth token, and that
            // there is a forked repo on their account.
            // Fork the repo, if it doesn't exist
            self.git_setup()?;

            self.genesis_registration_github()?;

            self.make_pull_request()?;
        }

        let ready = if do_genesis {
            Confirm::new()
                .with_prompt("\nNOW WAIT for everyone to do genesis. Is everyone ready?")
                .interact()
                .unwrap()
        } else {
            false
        };

        if ready {
            // Get Legacy Recovery from file
            let legacy_recovery = if let Some(p) = legacy_recovery_path {
                parse_json::parse(p)?
            } else {
                vec![]
            };

            genesis_builder::build(
                self.genesis_repo_org.clone(),
                self.repo_name.clone(),
                self.github_token.clone(),
                self.data_path.clone(),
                use_local_framework,
                Some(&legacy_recovery),
                supply_settings,
                self.chain.clone(),
                None,
            )?;

            for _ in (0..10)
                .progress_with_style(OLProgress::fun())
                .with_message("Initializing 0L")
            {
                thread::sleep(Duration::from_millis(100));
            }
        } else {
            println!("Please wait for everyone to finish genesis registration and come back");
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

        self.github_username = temp_gh_client
            .get_authenticated_user()
            .context("could not get authenticated user on github api")?;

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
        let pb = ProgressBar::new(1000).with_style(OLProgress::spinner());
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
                Ok(true) => {
                    gh_client.fork_genesis_repo(&self.genesis_repo_org, &self.repo_name)?;
                    // give it a few seconds after submitting. Otherwise will get a 500 error while the repo is being created
                    thread::sleep(Duration::from_secs(5));
                }
                _ => bail!("no forked repo on your account, we need it to continue"),
            }
        } else {
            println!("Found a genesis repo on your account, we'll use that for registration.\n");
        }

        pb.finish_and_clear();
        OLProgress::complete(&format!(
            "Forked the genesis repo from {}/{}",
            self.genesis_repo_org.clone(),
            self.repo_name.clone()
        ));
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
            self.validator_address.clone(),
            self.github_username.clone(), // Do the registration on the fork.
            self.repo_name.clone(),
            self.github_token.clone(),
            self.data_path.clone(),
        )?;

        pb.finish_and_clear();

        OLProgress::complete(&format!(
            "Configs written to {}/{}",
            self.github_username, self.repo_name
        ));

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
        let api_token = std::fs::read_to_string(gh_token_path)?;

        let pb = ProgressBar::new(1).with_style(OLProgress::bar());
        let gh_client = Client::new(
            self.genesis_repo_org.clone(),
            self.repo_name.clone(),
            DEFAULT_GIT_BRANCH.to_string(),
            api_token,
        );
        // repository_owner, genesis_repo_name, username
        // This will also fail if there already is a pull request!
        match gh_client.make_genesis_pull_request(
            &self.genesis_repo_org,
            &self.repo_name,
            &self.github_username,
            None, // default to "main"
        ) {
            Ok(_) => {}
            Err(e) => {
                if e.to_string().contains("A pull request already exists") {
                    println!(
                        "INFO: A pull request already exists, you don't need to do anything else."
                    );
                    // return Ok(())
                } else if e.to_string().contains("No commits between main and main") {
                    println!(
                        "INFO: A pull request already exists, and there are no changes with main"
                    );
                } else {
                    bail!("failed to create pull, message: {}", e.to_string())
                }
            }
        };
        pb.inc(1);
        pb.finish_and_clear();
        OLProgress::complete("Pull request to genesis repo complete");
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

#[test]
#[ignore]

fn test_wizard() {
    let mut wizard = GenesisWizard::new(
        "0LNetworkCommunity".to_string(),
        "test_genesis".to_string(),
        None,
    );
    wizard.start_wizard(false, None, false, None).unwrap();
}

#[test]
#[ignore] // dev helper
fn test_register() {
    let mut g = GenesisWizard::new(
        "0LNetworkCommunity".to_string(),
        "test_genesis".to_string(),
        None,
    );
    g.validator_address = "0xTEST".to_string();
    g.git_token_check().unwrap();
    g.genesis_registration_github().unwrap();
}
