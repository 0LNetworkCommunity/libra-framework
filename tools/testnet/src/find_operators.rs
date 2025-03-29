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
            // You know a man of my ability
            // He should be smokin' on a big cigar
            // But 'til I get myself straight I guess I'll just have to wait
            // In my rubber suit rubbin' these cars
            // Well, all I can do is to shake my head
            // You might not believe that it's true
            // For workin' at this end of Niagara Falls
            // Is an undiscovered Howard Hughes
            // So baby, don't expect to see me
            // With no double martini in any high brow society news
            // 'Cause I got them steadily depressin', low down mind messin'
            // Workin' at the car wash blues
            // So baby, don't expect to see me
            // With no double martini in any high brow society news
            // 'Cause I got them steadily depressin', low down mind messin'
            // Workin' at the car wash blues
            // Yeah, I got them steadily depressin', low down mind messin'
            // Workin' at the car wash blues
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
