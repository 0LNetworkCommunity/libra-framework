use anyhow::Result;
use diem_genesis::config::HostAndPort;
use diem_types::chain_id::NamedChain;
use glob::glob;
use libra_config::validator_registration::{registration_from_operator_yaml, ValCredentials};
use libra_genesis_tools::testnet_setup;
use libra_rescue::{
    one_step::one_step_apply_rescue_on_db, replace_validators::replace_validators_blob,
};
use libra_types::{core_types::fixtures::TestPersona, global_config_dir};
use std::path::{Path, PathBuf};

#[derive(clap::Parser)]
pub struct TestnetConfigOpts {
    #[clap(short, long)]
    /// location for the default data directory
    out_dir: Option<PathBuf>,
    /// name of the type of chain we are starting
    #[clap(short, long)]
    chain_name: Option<NamedChain>,
    /// sensible defaults for testnet, does not need a genesis repo
    /// accounts are created from fixture mnemonics for alice, bob, carol, dave
    /// which persona is this machine going to register as
    #[clap(short, long)]
    me: TestPersona,
    /// ordered list of dns/ip with port for alice..dave
    /// use 6180 for production validator port
    #[clap(short('i'), long)]
    host_list: Vec<HostAndPort>,
    /// path to file for legacy migration file
    #[clap(short, long)]
    json_legacy: Option<PathBuf>,
}

impl TestnetConfigOpts {
    pub async fn run(
        &self,
        framework_mrb_path: Option<PathBuf>,
        twin_db: Option<PathBuf>,
    ) -> Result<()> {
        let chain_name = self.chain_name.unwrap_or(NamedChain::TESTNET); // chain_id = 2
        let data_path = self.out_dir.clone().unwrap_or_else(global_config_dir);
        let out_path = testnet_setup::setup(
            &self.me,
            &self.host_list,
            chain_name,
            data_path,
            self.json_legacy.to_owned(),
            framework_mrb_path,
        )
        .await?;

        // if it's a twin case, then we need to do brain surgery
        // 1. collect all the operator.yaml files created
        // in first step.
        // 2. place a database in the default home path
        // 3. run the twin rescue mission
        // 4. use artifacts of #4 to update the config files
        if let Some(p) = twin_db {
            println!("configuring twin...");
            configure_twin(&out_path, &p).await?;
        }
        Ok(())
    }
}

/// Configure a twin network based on the specified options
async fn configure_twin(home_path: &Path, reference_db: &Path) -> anyhow::Result<()> {
    // don't do any operations on the reference db
    let destination_db = home_path.join("data/db");
    fs_extra::dir::copy(
        reference_db,
        &destination_db, // saving to standard db path
        &fs_extra::dir::CopyOptions::new(),
    )?;
    // Step 1: Collect all the operator.yaml files
    println!("Collecting operator configuration files...");
    // using glob read all the operator*.yaml files in <data_path>/operator_files
    let operator_path = home_path.join("operator_files");
    let pattern = operator_path
        .join("operator*.yaml")
        .to_string_lossy()
        .to_string();

    let mut operator_files: Vec<PathBuf> = Vec::new();
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("Found operator file: {:?}", path);
                operator_files.push(path);
            }
            Err(e) => println!("Error while processing operator file: {}", e),
        }
    }

    if operator_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No operator files found in {}",
            operator_path.display()
        ));
    }

    // Parse each operator file into ValCredentials
    let mut val_credentials: Vec<ValCredentials> = Vec::new();
    for path in operator_files {
        match registration_from_operator_yaml(Some(path)) {
            Ok(cred) => {
                println!(
                    "Successfully parsed credentials for account: {}",
                    cred.account
                );
                val_credentials.push(cred);
            }
            Err(e) => println!("Error parsing operator file: {}", e),
        }
    }

    if val_credentials.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid credentials could be parsed from operator files"
        ));
    }

    // Step 2 & 3: Run the twin rescue mission with the database path
    println!("Running twin rescue mission...");
    // Create and apply rescue blob
    println!("Creating rescue blob from the reference db");
    let rescue_blob_path = replace_validators_blob(&reference_db, val_credentials, false).await?;

    println!("Applying the rescue blob to the database & bootstrapping");
    let wp = one_step_apply_rescue_on_db(&reference_db, &rescue_blob_path)?;

    println!("Created rescue blob at: {}", rescue_blob_path.display());

    // Step 4: Update config files with artifacts
    println!("Updating configuration files...");
    // TODO: Add code to update configs with rescue blob

    println!("Twin configuration complete");
    Ok(())
}
