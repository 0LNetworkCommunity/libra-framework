use anyhow::{bail, Result};
use libra_config::extension::cli_config_ext::CliConfigExt;
use std::{collections::BTreeMap, str::FromStr};
use url::Url;
use zapatos::{
    account::key_rotation::lookup_address,
    common::{
        init::Network,
        types::{
            account_address_from_public_key, CliConfig, CliError, CliTypedResult, ConfigSearchMode,
            ProfileConfig, PromptOptions, DEFAULT_PROFILE,
        },
        utils::{prompt_yes_with_override, read_line},
    },
};
use zapatos_crypto::{ed25519::Ed25519PublicKey, ValidCryptoMaterialStringExt};
use zapatos_rest_client::{
    diem_api_types::{DiemError, DiemErrorCode},
    error::{DiemErrorResponse, RestError},
    Client,
};

pub async fn run(public_key: &str, profile: Option<&str>) -> Result<()> {
    let mut config = if CliConfig::config_exists_ext(ConfigSearchMode::CurrentDir) {
        CliConfig::load_ext(ConfigSearchMode::CurrentDir)?
    } else {
        CliConfig::default()
    };
    let profile_name = profile.unwrap_or(DEFAULT_PROFILE);
    let prompt_options = PromptOptions::default();
    let public_key = Ed25519PublicKey::from_encoded_string(public_key)?;

    // Select profile we're using
    let mut profile_config = if let Some(profile_config) = config.remove_profile(profile_name) {
        prompt_yes_with_override(&format!("0L already initialized for profile {}, do you want to overwrite the existing config?", profile_name), prompt_options)?;
        profile_config
    } else {
        ProfileConfig::default()
    };

    eprintln!("Configuring for profile {}", profile_name);

    // Choose a network
    eprintln!("Choose network from [devnet, testnet, mainnet, local, custom | defaults to local]");
    let input = read_line("network")?;
    let input = input.trim();
    let network = if input.is_empty() {
        eprintln!("No network given, using local...");
        Network::Local
    } else {
        Network::from_str(input)?
    };

    // Ensure that there is at least a REST URL set for the network
    match network {
        Network::Local => {
            profile_config.rest_url = Some("http://localhost:8080".to_string());
            profile_config.faucet_url = None;
        }
        Network::Custom => custom_network(&mut profile_config)?,
        _ => bail!("0L only supports Local and Custom networks (for now)"),
    }

    let client = Client::new(
        Url::parse(
            profile_config
                .rest_url
                .as_ref()
                .expect("Must have rest client as created above"),
        )
        .map_err(|err| CliError::UnableToParse("rest_url", err.to_string()))?,
    );

    // lookup the address from onchain instead of deriving it
    // if this is the rotated key, deriving it will outputs an incorrect address
    let derived_address = account_address_from_public_key(&public_key);
    let address = lookup_address(&client, derived_address, false).await?;

    profile_config.private_key = None;
    profile_config.public_key = Some(public_key);
    profile_config.account = Some(address);

    // Create account if it doesn't exist
    // Check if account exists
    let account_exists = match client.get_account(address).await {
        Ok(_) => true,
        Err(err) => {
            if let RestError::Api(DiemErrorResponse {
                error:
                    DiemError {
                        error_code: DiemErrorCode::ResourceNotFound,
                        ..
                    },
                ..
            })
            | RestError::Api(DiemErrorResponse {
                error:
                    DiemError {
                        error_code: DiemErrorCode::AccountNotFound,
                        ..
                    },
                ..
            }) = err
            {
                false
            } else {
                bail!("Failed to check if account exists: {:?}", err);
            }
        }
    };

    if account_exists {
        eprintln!("Account {} has been already found onchain", address);
    } else if network == Network::Mainnet {
        eprintln!("Account {} does not exist, you will need to create and fund the account by transferring funds from another account", address);
    } else {
        eprintln!("Account {} has been initialized locally, but you must transfer coins to it to create the account onchain", address);
    }

    // Ensure the loaded config has profiles setup for a possible empty file
    if config.profiles.is_none() {
        config.profiles = Some(BTreeMap::new());
    }
    config
        .profiles
        .as_mut()
        .expect("Must have profiles, as created above")
        .insert(profile_name.to_string(), profile_config);
    config.save_ext()?;
    eprintln!(
        "\n0L CLI is now set up for account {} as profile {}!",
        address,
        profile.unwrap_or(DEFAULT_PROFILE)
    );
    Ok(())
}

fn custom_network(profile_config: &mut ProfileConfig) -> CliTypedResult<()> {
    // Rest Endpoint
    let rest_url = {
        let current = profile_config.rest_url.as_deref();
        eprintln!(
            "Enter your rest endpoint [Current: {} | No input: Exit (or keep the existing if present)]",
            current.unwrap_or("None"),
        );
        let input = read_line("Rest endpoint")?;
        let input = input.trim();
        if input.is_empty() {
            if let Some(current) = current {
                eprintln!("No rest url given, keeping the existing url...");
                Some(current.to_string())
            } else {
                eprintln!("No rest url given, exiting...");
                return Err(CliError::AbortedError);
            }
        } else {
            Some(
                Url::parse(input)
                    .map_err(|err| CliError::UnableToParse("Rest Endpoint", err.to_string()))?
                    .to_string(),
            )
        }
    };
    profile_config.rest_url = rest_url;
    profile_config.faucet_url = None;
    Ok(())
}
