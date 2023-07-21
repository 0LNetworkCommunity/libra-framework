//! build the genesis file
use crate::supply::SupplySettings;
use anyhow::{anyhow, bail, Result};
use libra_wallet::utils::{check_if_file_exists, from_yaml, write_to_user_only_file};
use libra_framework::release;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;


use std::str::FromStr;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};
use zapatos_crypto::ed25519::ED25519_PUBLIC_KEY_LENGTH;
use zapatos_crypto::ValidCryptoMaterialStringExt;
use zapatos_crypto::{bls12381, ed25519::Ed25519PublicKey, ValidCryptoMaterial};
use zapatos_framework::ReleaseBundle;
use zapatos_genesis::{
    builder::GenesisConfiguration,
    config::{
        Layout, StringOperatorConfiguration, StringOwnerConfiguration, ValidatorConfiguration,
    },
    GenesisInfo,
};
use zapatos_github_client::Client;
use zapatos_types::account_address::AccountAddress;
use zapatos_types::{
    account_address::AccountAddressWithChecks, on_chain_config::{OnChainConsensusConfig, OnChainExecutionConfig},

};
use zapatos_vm_genesis::default_gas_schedule;

use crate::genesis::make_recovery_genesis_from_vec_legacy_recovery;
use libra_types::ol_progress::OLProgress;
use crate::wizard::DEFAULT_GIT_BRANCH;

pub const LAYOUT_FILE: &str = "layout.yaml";
pub const OPERATOR_FILE: &str = "operator.yaml";
pub const OWNER_FILE: &str = "owner.yaml";
pub const FRAMEWORK_NAME: &str = "framework.mrb";
const WAYPOINT_FILE: &str = "waypoint.txt";
const GENESIS_FILE: &str = "genesis.blob";

pub fn build(
    github_owner: String,
    github_repository: String,
    github_token: String,
    home_path: PathBuf,
    use_local_framework: bool,
    legacy_recovery: Option<&[LegacyRecovery]>,
    supply_settings: Option<SupplySettings>,
) -> Result<Vec<PathBuf>> {
    let output_dir = home_path.join("genesis");
    std::fs::create_dir_all(&output_dir)?;

    let genesis_file = output_dir.join(GENESIS_FILE);
    let waypoint_file = output_dir.join(WAYPOINT_FILE);

    check_if_file_exists(genesis_file.as_path())?;
    check_if_file_exists(waypoint_file.as_path())?;

    // Generate genesis and waypoint files
    let (genesis_bytes, waypoint) = {
        println!("fetching genesis info from github");
        let mut gen_info = fetch_genesis_info(github_owner, github_repository, github_token, use_local_framework)?;

        println!("building genesis block");
        let tx = make_recovery_genesis_from_vec_legacy_recovery(
            legacy_recovery,
            &gen_info.validators,
            &gen_info.framework,
            gen_info.chain_id,
            supply_settings,
          )?;
        // NOTE: if genesis TX is not set, then it will run the vendor's release workflow, which we do not want.
        gen_info.genesis = Some(tx);

        (bcs::to_bytes(gen_info.get_genesis())?, gen_info.generate_waypoint()?)
    };

    write_to_user_only_file(genesis_file.as_path(), GENESIS_FILE, &genesis_bytes)?;
    write_to_user_only_file(
        waypoint_file.as_path(),
        WAYPOINT_FILE,
        waypoint.to_string().as_bytes(),
    )?;

    // TODO!: compare output
    // if let Some(l) = legacy_recovery {
    //   compare_json_to_genesis_blob(legacy_recovery, genesis_file, );
    // }

    OLProgress::complete(&format!("genesis successfully built at {}", output_dir.to_str().unwrap()));
    Ok(vec![genesis_file, waypoint_file])
}

/// Retrieves all information for mainnet genesis from the Git repository
pub fn fetch_genesis_info(
    github_owner: String,
    github_repository: String,
    github_token: String,
    use_local_framework: bool,
) -> Result<GenesisInfo> {
    // let client = git_options.get_client()?;
    let client = Client::new(
        github_owner.clone(), // doesn't matter
        github_repository.clone(),
        DEFAULT_GIT_BRANCH.to_string(),
        github_token.clone(),
    );

    // let layout: Layout = client.get(Path::new(LAYOUT_FILE))?;
    let l_file = client.get_file(&Path::new(LAYOUT_FILE).display().to_string())?;
    let layout: Layout = from_yaml(&String::from_utf8(base64::decode(l_file)?)?)?;
    OLProgress::complete("fetched layout file");


    let pb = OLProgress::spin_steady(100, "fetching validator registrations".to_string());
    let validators = get_validator_configs(&client, &layout, false)?;
    OLProgress::complete("fetched validator configs");
    pb.finish_and_clear();

    let framework = if use_local_framework {
      // use the local head release
      release::ReleaseTarget::Head.load_bundle()?
    } else {
      // get from github
      let bytes = base64::decode(client.get_file(FRAMEWORK_NAME)?)?;
      bcs::from_bytes::<ReleaseBundle>(&bytes)?
    };


    // let framework = client.get_framework()?;
    let dummy_root = Ed25519PublicKey::from_encoded_string(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("could not parse dummy root");

    Ok(GenesisInfo::new(
        layout.chain_id,
        dummy_root, // TODO: neuter from Move code
        validators,
        framework,
        &GenesisConfiguration {
            allow_new_validators: layout.allow_new_validators,
            epoch_duration_secs: layout.epoch_duration_secs,
            is_test: layout.is_test,
            min_stake: layout.min_stake,
            min_voting_threshold: layout.min_voting_threshold,
            max_stake: layout.max_stake,
            recurring_lockup_duration_secs: layout.recurring_lockup_duration_secs,
            required_proposer_stake: layout.required_proposer_stake,
            rewards_apy_percentage: layout.rewards_apy_percentage,
            voting_duration_secs: layout.voting_duration_secs,
            voting_power_increase_limit: layout.voting_power_increase_limit,
            employee_vesting_start: layout.employee_vesting_start,
            employee_vesting_period_duration: layout.employee_vesting_period_duration,
            consensus_config: OnChainConsensusConfig::default(),
            execution_config: OnChainExecutionConfig::default(),
            gas_schedule: default_gas_schedule(),
        },
    )?)
}

fn get_validator_configs(
    client: &Client,
    layout: &Layout,
    is_mainnet: bool,
) -> Result<Vec<ValidatorConfiguration>> {
    let mut validators = Vec::new();
    let mut errors = Vec::new();
    for user in &layout.users {
        match get_config(client, user, is_mainnet) {
            Ok(validator) => {
                validators.push(validator);
            }
            Err(failure) => {
                errors.push(format!("{}: {:?}", user, failure));
            }
        }
    }

    if errors.is_empty() {
        Ok(validators)
    } else {
        bail!("{:?}", errors)
    }
}

/// Do proper parsing so more information is known about failures
fn get_config(client: &Client, user: &str, _is_mainnet: bool) -> Result<ValidatorConfiguration> {
    // Load a user's configuration files
    let dir = PathBuf::from(user);
    let owner_file = dir.join(OWNER_FILE);
    let owner_file = owner_file.as_path();

    let file = client.get_file(&Path::new(owner_file).display().to_string())?;
    let owner_config: StringOwnerConfiguration =
        from_yaml(&String::from_utf8(base64::decode(file)?)?)?;

    // Check and convert fields in owner file
    let owner_account_address: AccountAddress = parse_required_option(
        &owner_config.owner_account_address,
        owner_file,
        "owner_account_address",
        AccountAddressWithChecks::from_str,
    )?
    .into();
    let owner_account_public_key = parse_required_option(
        &owner_config.owner_account_public_key,
        owner_file,
        "owner_account_public_key",
        |str| parse_key(ED25519_PUBLIC_KEY_LENGTH, str),
    )?;

    let operator_account_address: AccountAddress = parse_required_option(
        &owner_config.operator_account_address,
        owner_file,
        "operator_account_address",
        AccountAddressWithChecks::from_str,
    )?
    .into();
    let operator_account_public_key = parse_required_option(
        &owner_config.operator_account_public_key,
        owner_file,
        "operator_account_public_key",
        |str| parse_key(ED25519_PUBLIC_KEY_LENGTH, str),
    )?;

    let voter_account_address: AccountAddress = parse_required_option(
        &owner_config.voter_account_address,
        owner_file,
        "voter_account_address",
        AccountAddressWithChecks::from_str,
    )?
    .into();
    let voter_account_public_key = parse_required_option(
        &owner_config.voter_account_public_key,
        owner_file,
        "voter_account_public_key",
        |str| parse_key(ED25519_PUBLIC_KEY_LENGTH, str),
    )?;

    let stake_amount = parse_required_option(
        &owner_config.stake_amount,
        owner_file,
        "stake_amount",
        u64::from_str,
    )?;

    // Default to 0 for commission percentage if missing.
    let commission_percentage = parse_optional_option(
        &owner_config.commission_percentage,
        owner_file,
        "commission_percentage",
        u64::from_str,
    )?
    .unwrap_or(0);

    // Default to true for whether the validator should be joining during genesis.
    let join_during_genesis = parse_optional_option(
        &owner_config.join_during_genesis,
        owner_file,
        "join_during_genesis",
        bool::from_str,
    )?
    .unwrap_or(true);

    // We don't require the operator file if the validator is not joining during genesis.
    // if is_mainnet && !join_during_genesis {
    //     return Ok(ValidatorConfiguration {
    //         owner_account_address: owner_account_address.into(),
    //         owner_account_public_key,
    //         operator_account_address: operator_account_address.into(),
    //         operator_account_public_key,
    //         voter_account_address: voter_account_address.into(),
    //         voter_account_public_key,
    //         consensus_public_key: None,
    //         proof_of_possession: None,
    //         validator_network_public_key: None,
    //         validator_host: None,
    //         full_node_network_public_key: None,
    //         full_node_host: None,
    //         stake_amount,
    //         commission_percentage,
    //         join_during_genesis,
    //     });
    // };

    let operator_file = dir.join(OPERATOR_FILE);
    let operator_file = operator_file.as_path();

    let file = client.get_file(&Path::new(operator_file).display().to_string())?;
    let operator_config: StringOperatorConfiguration =
        from_yaml(&String::from_utf8(base64::decode(file)?)?)?;

    // let operator_config = client.get::<StringOperatorConfiguration>(operator_file)?;

    // Check and convert fields in operator file
    let operator_account_address_from_file: AccountAddress = parse_required_option(
        &operator_config.operator_account_address,
        operator_file,
        "operator_account_address",
        AccountAddressWithChecks::from_str,
    )?
    .into();
    let operator_account_public_key_from_file = parse_required_option(
        &operator_config.operator_account_public_key,
        operator_file,
        "operator_account_public_key",
        |str| parse_key(ED25519_PUBLIC_KEY_LENGTH, str),
    )?;
    let consensus_public_key = parse_required_option(
        &operator_config.consensus_public_key,
        operator_file,
        "consensus_public_key",
        |str| parse_key(bls12381::PublicKey::LENGTH, str),
    )?;
    let consensus_proof_of_possession = parse_required_option(
        &operator_config.consensus_proof_of_possession,
        operator_file,
        "consensus_proof_of_possession",
        |str| parse_key(bls12381::ProofOfPossession::LENGTH, str),
    )?;
    let validator_network_public_key = parse_required_option(
        &operator_config.validator_network_public_key,
        operator_file,
        "validator_network_public_key",
        |str| parse_key(ED25519_PUBLIC_KEY_LENGTH, str),
    )?;
    let full_node_network_public_key = parse_optional_option(
        &operator_config.full_node_network_public_key,
        operator_file,
        "full_node_network_public_key",
        |str| parse_key(ED25519_PUBLIC_KEY_LENGTH, str),
    )?;

    // Verify owner & operator agree on operator
    if operator_account_address != operator_account_address_from_file {
        return Err(
            anyhow!("Operator account {} in owner file {} does not match operator account {} in operator file {}",
                        operator_account_address,
                        owner_file.display(),
                        operator_account_address_from_file,
                        operator_file.display()
                ));
    }
    if operator_account_public_key != operator_account_public_key_from_file {
        return Err(
            anyhow!("Operator public key {} in owner file {} does not match operator public key {} in operator file {}",
                        operator_account_public_key,
                        owner_file.display(),
                        operator_account_public_key_from_file,
                        operator_file.display()
                ));
    }

    // Build Validator configuration
    Ok(ValidatorConfiguration {
        owner_account_address: owner_account_address.into(),
        owner_account_public_key,
        operator_account_address: operator_account_address.into(),
        operator_account_public_key,
        voter_account_address: voter_account_address.into(),
        voter_account_public_key,
        consensus_public_key: Some(consensus_public_key),
        proof_of_possession: Some(consensus_proof_of_possession),
        validator_network_public_key: Some(validator_network_public_key),
        validator_host: Some(operator_config.validator_host),
        full_node_network_public_key,
        full_node_host: operator_config.full_node_host,
        stake_amount,
        commission_percentage,
        join_during_genesis,
    })
}

// TODO: Move into the Crypto libraries
fn parse_key<T: ValidCryptoMaterial>(num_bytes: usize, str: &str) -> Result<T> {
    let num_chars: usize = num_bytes * 2;
    let mut working = str.trim();

    // Checks if it has a 0x at the beginning, which is okay
    if working.starts_with("0x") {
        working = &working[2..];
    }

    match working.len().cmp(&num_chars) {
        Ordering::Less => {
            anyhow::bail!(
                "Key {} is too short {} must be {} hex characters",
                str,
                working.len(),
                num_chars
            )
        }
        Ordering::Greater => {
            anyhow::bail!(
                "Key {} is too long {} must be {} hex characters with or without a 0x in front",
                str,
                working.len(),
                num_chars
            )
        }
        Ordering::Equal => {}
    }

    if !working.chars().all(|c| char::is_ascii_hexdigit(&c)) {
        anyhow::bail!("Key {} contains a non-hex character", str)
    }

    Ok(T::from_encoded_string(str.trim())?)
}

fn parse_required_option<F: Fn(&str) -> Result<T, E>, T, E: std::fmt::Display>(
    option: &Option<String>,
    file: &Path,
    field_name: &'static str,
    parse: F,
) -> Result<T> {
    if let Some(ref field) = option {
        parse(field).map_err(|err| {
            anyhow!(
                "Field {} is invalid in file {}.  Err: {}",
                field_name,
                file.display(),
                err
            )
        })
    } else {
        Err(anyhow!("File {} is missing {}", file.display(), field_name))
    }
}

fn parse_optional_option<F: Fn(&str) -> Result<T, E>, T, E: std::fmt::Display>(
    option: &Option<String>,
    file: &Path,
    field_name: &'static str,
    parse: F,
) -> Result<Option<T>> {
    if let Some(ref field) = option {
        parse(field)
            .map_err(|err| {
                anyhow!(
                    "Field {} is invalid in file {}.  Err: {}",
                    field_name,
                    file.display(),
                    err
                )
            })
            .map(Some)
    } else {
        Ok(None)
    }
}

#[test]
#[ignore] //dev helper
fn test_github_info() {
    let gh_token_path = libra_types::global_config_dir()
        .join("github_token.txt");
    let token = std::fs::read_to_string(&gh_token_path).unwrap();

    let _genesis_info =
        fetch_genesis_info("0o-de-lally".to_string(), "a-genesis".to_string(), token, true).unwrap();
}


#[test]
#[ignore] //dev helper
fn test_build() {
    let home = libra_types::global_config_dir();
    let token = std::fs::read_to_string(&home.join("github_token.txt")).unwrap();

    build(
    "0o-de-lally".to_string(),
    "a-genesis".to_string(),
    token,
    home,
    true,
    None,
    None,
    ).unwrap();
}
