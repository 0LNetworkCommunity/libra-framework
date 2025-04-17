use glob::glob;
use libra_config::validator_registration::{registration_from_operator_yaml, ValCredentials};
use std::path::{Path, PathBuf};

pub fn find_operator_configs(start_path: &Path) -> anyhow::Result<Vec<ValCredentials>> {
    assert!(start_path.exists(), "start_path dir should exist");

    let pattern = format!("{}/**/operator*.yaml", start_path.display());

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
    Ok(val_credentials)
}
