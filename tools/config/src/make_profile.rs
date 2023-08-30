use anyhow::{bail, Result};
use libra_types::type_extensions::cli_config_ext::CliConfigExt;
use std::{collections::BTreeMap, env, str::FromStr};
use url::Url;
use diem::{
    account::key_rotation::lookup_address,
    common::{
        init::Network,
        types::{
            account_address_from_public_key, CliConfig, CliError, CliTypedResult, ProfileConfig,
            PromptOptions, DEFAULT_PROFILE,
        },
        utils::{prompt_yes_with_override, read_line},
    },
};
use diem_crypto::{ed25519::Ed25519PublicKey, ValidCryptoMaterialStringExt};
use diem_rest_client::{
    diem_api_types::{DiemError, DiemErrorCode},
    error::{DiemErrorResponse, RestError},
    Client,
};
use diem_types::account_address::AccountAddress;

pub async fn run(public_key: &str, profile: Option<&str>, workspace: bool) -> Result<()> {
    // init_workspace
    let mut config = CliConfig::default();

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

    let mut address = account_address_from_public_key(&public_key);

    // lookup the address from onchain instead of deriving it
    // if this is the rotated key, deriving it will outputs an incorrect address
    eprintln!("Have you ever rotated the account keys? We can look on chain to see what your actual address is. [y/N]");
    let input = read_line("address")?;
    let input = input.trim();
    if input.contains('y') {
        let client = check_network(&profile_config);
        if client.is_ok() {
            match lookup_address(&client.unwrap(), address, false).await {
                Ok(a) => address = a,
                Err(_) => eprintln!("Could not lookup address on chain, using derived address"),
            };
        } else {
            eprintln!("Could not lookup address on chain, using derived address");
        }
    };

    profile_config.private_key = None;
    profile_config.public_key = Some(public_key);
    profile_config.account = Some(address);

    // Ensure the loaded config has profiles setup for a possible empty file
    if config.profiles.is_none() {
        config.profiles = Some(BTreeMap::new());
    }
    config
        .profiles
        .as_mut()
        .expect("Must have profiles, as created above")
        .insert(profile_name.to_string(), profile_config);

    // In 0L we default to the configs being global in $HOME/.libra
    // Otherwise you should pass -w to use the workspace configuration.
    let config_location = if workspace {
        env::current_dir().ok()
    } else {
        None
    };
    let path = config.save_ext(config_location)?;
    eprintln!(
        "\nThe libra configuration is saved! \nfor account {} \nat path: {}  as profile {}",
        address,
        path.to_str().unwrap(),
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

fn check_network(cfg: &ProfileConfig) -> Result<Client> {
    let base = Url::parse(
        cfg.rest_url
            .as_ref()
            .expect("Must have rest client as created above"),
    )
    .map_err(|err| CliError::UnableToParse("rest_url", err.to_string()))?;

    let client = Client::new(base);

    Ok(client)
}

async fn _check_account_on_chain(client: &Client, address: AccountAddress) {
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
                eprintln!("Failed to check if account exists on chain: {:?}", err);
                false
            }
        }
    };

    if account_exists {
        eprintln!("Account {} has been already found onchain", address);
    } else {
        eprintln!("Account {} has been initialized locally, but you must transfer coins to it to create the account onchain", address);
    }
}
