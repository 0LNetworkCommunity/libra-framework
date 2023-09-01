use anyhow::Result;
use libra_wallet::{
    utils::to_yaml,
    validator_files::SetValidatorConfiguration,
    validator_files::{OPERATOR_FILE, OWNER_FILE},
};
use std::path::PathBuf;
use zapatos_github_client::Client;

// TODO: duplicate with libra-wallet and crate/aptos/src/genesis/keys
pub const PUBLIC_KEYS_FILE: &str = "public-keys.yaml";

/// Function to publish the validator configuration files to github
pub fn register(
    genesis_username: String,
    github_owner: String,
    github_repository: String,
    github_token: String,
    home_path: PathBuf,
) -> Result<()> {
    let directory = PathBuf::from(genesis_username);
    let operator_file = directory.join(OPERATOR_FILE);
    let owner_file = directory.join(OWNER_FILE);

    let (operator_config, owner_config) =
        SetValidatorConfiguration::read_configs_from_file(Some(home_path))?;

    let git_client = Client::new(
        github_owner,
        github_repository,
        "main".to_owned(),
        github_token,
    );

    git_client.put(
        &operator_file.display().to_string(),
        &base64::encode(to_yaml(&operator_config)?),
    )?;

    git_client.put(
        &owner_file.display().to_string(),
        &base64::encode(to_yaml(&owner_config)?),
    )?;

    Ok(())
}
