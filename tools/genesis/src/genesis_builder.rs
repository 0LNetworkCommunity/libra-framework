//! build the genesis file
use crate::genesis::make_recovery_genesis_from_vec_legacy_recovery;
use crate::supply::SupplySettings;
use crate::vm::libra_genesis_default;
use crate::wizard::DEFAULT_GIT_BRANCH;
use crate::{compare, supply, vm};

use std::str::FromStr;
use std::time::Duration;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use indicatif::ProgressBar;

use libra_framework::release;
use libra_types::exports::ChainId;
use libra_types::exports::NamedChain;
use libra_types::legacy_types::fixtures::TestPersona;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
use libra_types::ol_progress::OLProgress;
use libra_wallet::account_keys::get_keys_from_mnem;
use libra_wallet::keys::generate_key_objects_from_legacy;
use libra_wallet::utils::{check_if_file_exists, from_yaml, write_to_user_only_file};
use serde::{Deserialize, Serialize};
use zapatos_crypto::ed25519::ED25519_PUBLIC_KEY_LENGTH;
use zapatos_crypto::ValidCryptoMaterialStringExt;
use zapatos_crypto::{bls12381, ed25519::Ed25519PublicKey, ValidCryptoMaterial};
use zapatos_framework::ReleaseBundle;
use zapatos_genesis::config::HostAndPort;
use zapatos_genesis::{
    builder::GenesisConfiguration,
    config::{StringOperatorConfiguration, StringOwnerConfiguration, ValidatorConfiguration},
    GenesisInfo,
};
use zapatos_github_client::Client;
use zapatos_types::account_address::AccountAddress;
use zapatos_types::{
    account_address::AccountAddressWithChecks,
    on_chain_config::{OnChainConsensusConfig, OnChainExecutionConfig},
};
use zapatos_vm_genesis::{
    default_gas_schedule,
    GenesisConfiguration as VmGenesisGenesisConfiguration, // in vendor codethere are two structs separately called the same name with nearly identical fields
};

pub const LAYOUT_FILE: &str = "layout.yaml";
pub const OPERATOR_FILE: &str = "operator.yaml";
pub const OWNER_FILE: &str = "owner.yaml";
pub const FRAMEWORK_NAME: &str = "framework.mrb";
const WAYPOINT_FILE: &str = "waypoint.txt";
const GENESIS_FILE: &str = "genesis.blob";

/// Minimal template for layout.yaml accounts in Genesis
///
#[derive(Debug, Deserialize, Serialize)]
struct LibraSimpleLayout {
    /// List of usernames or identifiers
    pub users: Vec<String>,
}

pub fn build(
    github_owner: String,
    github_repository: String,
    github_token: String,
    home_path: PathBuf,
    use_local_framework: bool,
    legacy_recovery: Option<&[LegacyRecovery]>,
    supply_settings: Option<SupplySettings>,
    chain_name: NamedChain,
    testnet_vals: Option<Vec<ValidatorConfiguration>>,
) -> Result<Vec<PathBuf>> {
    let output_dir = home_path.join("genesis");
    std::fs::create_dir_all(&output_dir)?;

    let genesis_file = output_dir.join(GENESIS_FILE);
    let waypoint_file = output_dir.join(WAYPOINT_FILE);

    // NOTE: export env LIBRA_CI=1 to avoid y/n prompt
    if testnet_vals.is_none() {
        check_if_file_exists(genesis_file.as_path())?;
        check_if_file_exists(waypoint_file.as_path())?;
    }

    let genesis_config = vm::libra_genesis_default(chain_name);
    // println!("\nfetching genesis info from github");
    // let mut gen_info = fetch_genesis_info(
    //     github_owner,
    //     github_repository,
    //     github_token,
    //     use_local_framework,
    //     &genesis_config,
    //     &chain_name,
    // )?;

    // Generate genesis and waypoint files
    // {
    let mut gen_info = if let Some(vals) = testnet_vals {
        let dummy_root = Ed25519PublicKey::from_encoded_string(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        .expect("could not parse dummy root");
        // make_testnet_tx(legacy_recovery, vals, chain_name, &supply_settings, &genesis_config)
        GenesisInfo::new(
            ChainId::new(chain_name.id()),
            dummy_root,
            vals,
            libra_framework::head_release_bundle(),
            &silly_config(&genesis_config),
        )?
    } else {
        fetch_genesis_info(
            github_owner,
            github_repository,
            github_token,
            use_local_framework,
            &genesis_config,
            &chain_name,
        )?
    };
    println!("building genesis block");
    let tx = make_recovery_genesis_from_vec_legacy_recovery(
        legacy_recovery,
        &gen_info.validators,
        &gen_info.framework,
        gen_info.chain_id,
        supply_settings.clone(),
        &genesis_config,
    )?;

    // NOTE: if genesis TX is not set, then it will run the vendor's release workflow, which we do not want.
    gen_info.genesis = Some(tx);
    OLProgress::complete("genesis transaction encoded");

    let pb = ProgressBar::new(1000)
        .with_style(OLProgress::spinner())
        .with_message("saving files");
    pb.enable_steady_tick(Duration::from_millis(100));

    write_to_user_only_file(
        genesis_file.as_path(),
        GENESIS_FILE,
        bcs::to_bytes(gen_info.get_genesis())?.as_slice(),
    )?;

    write_to_user_only_file(
        waypoint_file.as_path(),
        WAYPOINT_FILE,
        gen_info.generate_waypoint()?.to_string().as_bytes(),
    )?;
    pb.finish_and_clear();
    OLProgress::complete(&format!(
        "genesis file saved to {}",
        output_dir.to_str().unwrap()
    ));

    // (bcs::to_bytes(gen_info.get_genesis())?, gen_info.generate_waypoint()?, tx)
    // };

    // Audits the generated genesis.blob comparing to the JSON input.
    if let Some(recovery) = legacy_recovery {
        let settings = supply_settings.context("no supply settings provided")?;

        let mut s = supply::populate_supply_stats_from_legacy(recovery, &settings.map_dd_to_slow)?;

        s.set_ratios_from_settings(&settings)?;
        compare::compare_recovery_vec_to_genesis_tx(recovery, gen_info.get_genesis(), &s)?;
        OLProgress::complete("account balances as expected");

        compare::check_supply(settings.scale_supply() as u64, gen_info.get_genesis())?;
        OLProgress::complete("final supply as expected");
    }

    OLProgress::complete("LFG, ready for genesis");
    Ok(vec![genesis_file, waypoint_file])
}

/// there are two structs called GenesisConfiguration in Vendor code, sigh.
fn silly_config(cfg: &VmGenesisGenesisConfiguration) -> GenesisConfiguration {
    GenesisConfiguration {
        allow_new_validators: cfg.allow_new_validators,
        epoch_duration_secs: cfg.epoch_duration_secs,
        is_test: cfg.is_test,
        min_stake: cfg.min_stake,
        min_voting_threshold: cfg.min_voting_threshold,
        max_stake: cfg.max_stake,
        recurring_lockup_duration_secs: cfg.recurring_lockup_duration_secs,
        required_proposer_stake: cfg.required_proposer_stake,
        rewards_apy_percentage: cfg.rewards_apy_percentage,
        voting_duration_secs: cfg.voting_duration_secs,
        voting_power_increase_limit: cfg.voting_power_increase_limit,
        employee_vesting_start: None,
        employee_vesting_period_duration: None,
        consensus_config: OnChainConsensusConfig::default(),
        execution_config: OnChainExecutionConfig::default(),
        gas_schedule: default_gas_schedule(),
    }
}

// /// make genesis transaction from Github
// fn make_genesis_tx(
//   github_owner: String,
//   github_repository: String,
//   github_token: String,
//   use_local_framework: bool,
//   legacy_recovery: Option<&[LegacyRecovery]>,
//   // genesis_vals: &[Validator],
//   // framework_release: &ReleaseBundle,
//   chain_name: NamedChain,
//   supply_settings: &Option<SupplySettings>,
//   genesis_config: &VmGenesisGenesisConfiguration
// ) -> anyhow::Result<GenesisInfo>{
//     println!("\nfetching genesis info from github");
//     fetch_genesis_info(
//         github_owner,
//         github_repository,
//         github_token,
//         use_local_framework,
//         genesis_config,
//         &chain_name,
//     )?

//     // println!("building genesis block");
//     //     let tx = make_recovery_genesis_from_vec_legacy_recovery(
//     //         legacy_recovery,
//     //         &gen_info.validators,
//     //         &gen_info.framework,
//     //         gen_info.chain_id,
//     //         supply_settings.to_owned(),
//     //         &genesis_config,
//     //     )?;
//     Ok(tx)
// }

// /// helper to create a testnet with defaults
// fn make_testnet_tx(legacy_recovery: Option<&[LegacyRecovery]>, genesis_vals: &[Validator], chain_name: NamedChain, supply_settings: &Option<SupplySettings>, genesis_config: &VmGenesisGenesisConfiguration) -> anyhow::Result<Transaction>{

//       let framerwork_releae = libra_framework::head_release_bundle();
//       let tx = make_recovery_genesis_from_vec_legacy_recovery(
//         legacy_recovery,
//         genesis_vals,
//         &framework_release,
//         ChainId::new(chain_name.id()),
//         supply_settings.to_owned(),
//         genesis_config,
//     )?;
//     Ok(tx)
// }

/// Retrieves all information for mainnet genesis from the Git repository
pub fn fetch_genesis_info(
    github_owner: String,
    github_repository: String,
    github_token: String,
    use_local_framework: bool,
    genesis_config: &VmGenesisGenesisConfiguration,
    chain_id: &NamedChain,
) -> Result<GenesisInfo> {
    // let client = git_options.get_client()?;
    let client = Client::new(
        github_owner, // doesn't matter
        github_repository,
        DEFAULT_GIT_BRANCH.to_string(),
        github_token,
    );

    // let layout: Layout = client.get(Path::new(LAYOUT_FILE))?;
    let l_file = client.get_file(&Path::new(LAYOUT_FILE).display().to_string())?;
    let layout: LibraSimpleLayout = from_yaml(&String::from_utf8(base64::decode(l_file)?)?)?;
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

    // NOTE: in vendor code a root key is used in testnet to facilitate some tests. In libra we have written our test suite to be as close to mainnet as possible, so we don't have a faucet or other functions which need a root key.
    let dummy_root = Ed25519PublicKey::from_encoded_string(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("could not parse dummy root");

    GenesisInfo::new(
        ChainId::new(chain_id.id()),
        dummy_root, // NOTE: neutered in caller and in Move code
        validators,
        framework,
        &silly_config(genesis_config),
    )
}

fn get_validator_configs(
    client: &Client,
    layout: &LibraSimpleLayout,
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

/// create validator configs from fixture mnemonics
pub fn testnet_validator_config(
    persona: &TestPersona,
    host: &HostAndPort,
) -> anyhow::Result<ValidatorConfiguration> {
    let mnem = persona.get_persona_mnem();
    let key_chain = get_keys_from_mnem(mnem)?;
    let (_, _, _, public_identity) = generate_key_objects_from_legacy(&key_chain)?;

    Ok(ValidatorConfiguration {
        owner_account_address: public_identity.account_address.into(),
        owner_account_public_key: public_identity.account_public_key.clone(),
        operator_account_address: public_identity.account_address.into(),
        operator_account_public_key: public_identity.account_public_key.clone(),
        voter_account_address: public_identity.account_address.into(),
        voter_account_public_key: public_identity.account_public_key,
        consensus_public_key: public_identity.consensus_public_key,
        proof_of_possession: public_identity.consensus_proof_of_possession,
        validator_network_public_key: public_identity.validator_network_public_key,
        validator_host: Some(host.to_owned()),
        full_node_network_public_key: public_identity.full_node_network_public_key,
        full_node_host: Some(host.to_owned()),
        stake_amount: 1,
        commission_percentage: 1,
        join_during_genesis: true,
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
    let gh_token_path = libra_types::global_config_dir().join("github_token.txt");
    let token = std::fs::read_to_string(gh_token_path).unwrap();

    let _genesis_info = fetch_genesis_info(
        "0o-de-lally".to_string(),
        "a-genesis".to_string(),
        token,
        true,
        &libra_genesis_default(NamedChain::TESTING),
        &NamedChain::TESTING,
    )
    .unwrap();
}

#[test]
#[ignore] //dev helper
fn test_build() {
    let home = libra_types::global_config_dir();
    let token = std::fs::read_to_string(home.join("github_token.txt")).unwrap();

    build(
        "0o-de-lally".to_string(),
        "a-genesis".to_string(),
        token,
        home,
        true,
        None,
        None,
        NamedChain::TESTING,
        None,
    )
    .unwrap();
}
