use anyhow::{bail, Context};
use diem_genesis::config::{HostAndPort, OperatorConfiguration, ValidatorConfiguration};
use libra_backwards_compatibility::legacy_recovery_v6::LegacyRecoveryV6;
use libra_config::validator_config;
use libra_genesis_tools::{genesis_builder, parse_json};
use libra_types::{
    core_types::fixtures::TestPersona,
    exports::{AccountAddress, AuthenticationKey, NamedChain},
    move_resource::{
        cumulative_deposits::LegacyBalanceResourceV6,
        pledge_account::{MyPledgesResource, PledgeAccountResource},
    },
    ONCHAIN_DECIMAL_PRECISION,
};
use serde_yaml;
use std::{fs, path::PathBuf, thread, time}; // Explicitly import serde_yaml

// Simple function to convert ValidatorConfiguration to OperatorConfiguration
fn validator_to_operator_config(
    config: &ValidatorConfiguration,
) -> anyhow::Result<OperatorConfiguration> {
    let consensus_public_key = config
        .consensus_public_key
        .clone()
        .context("Consensus public key is required for operator configuration")?;

    let consensus_proof_of_possession = config
        .proof_of_possession
        .clone()
        .context("Proof of possession is required for operator configuration")?;

    let validator_network_public_key = config
        .validator_network_public_key
        .context("Validator network public key is required for operator configuration")?;

    let validator_host = config
        .validator_host
        .clone()
        .context("Validator host is required for operator configuration")?;

    Ok(OperatorConfiguration {
        operator_account_address: config.operator_account_address,
        operator_account_public_key: config.operator_account_public_key.clone(),
        consensus_public_key,
        consensus_proof_of_possession,
        validator_network_public_key,
        validator_host,
        full_node_network_public_key: config.full_node_network_public_key,
        full_node_host: config.full_node_host.clone(),
    })
}

// Sets up the environment for the given test persona.
// returns the home data path
pub async fn setup(
    me: &TestPersona,
    host_list: &[HostAndPort],
    chain: NamedChain,
    data_path: PathBuf,
    legacy_data_path: Option<PathBuf>,
    framework_mrb_path: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    // config the host address for this persona
    if host_list.len() < 3 {
        bail!("cannot start a testnet with less than 3 nodes, use --host-list for each of Alice, Bob, Carol and Dave but not more. Exiting.")
    }
    if host_list.len() > 4 {
        bail!("too many hosts provided, you just need 3 or 4 for a good testnet genesis. Exiting.")
    }

    println!("Building genesis config files for a network with:");
    for (i, h) in host_list.iter().enumerate() {
        let character = TestPersona::from(i)?;

        let display = format!("{}:{}", h.host, h.port);
        println!("persona: {character} - host: {display}");
        println!("mnemonic: {}\n", character.get_persona_mnem());
    }

    let index = me.idx();
    let my_host = host_list.get(index).expect("could not get an IP and index");
    println!(
        "your persona '{me}' is expected to use network address: {}:{}\n",
        my_host.host, my_host.port
    );

    // create the local files for my_persona
    if data_path.exists() {
        println!("WARN: deleting {}, in 5 secs", &data_path.display());
        let delay = time::Duration::from_secs(5);
        thread::sleep(delay);
        fs::remove_dir_all(&data_path)?;
    }

    // Initializes the validator configuration.
    validator_config::initialize_validator(
        Some(data_path.clone()),
        Some(&me.to_string()),
        my_host.clone(),
        Some(me.get_persona_mnem()),
        false,
        Some(chain),
    )
    .await?;

    // create validator configurations from fixtures
    // without needing to use a github repo to register and read
    let val_cfg: Vec<ValidatorConfiguration> = host_list
        .iter()
        .enumerate()
        .filter_map(|(idx, h)| {
            let p = TestPersona::from(idx).ok()?;
            genesis_builder::testnet_validator_config(&p, h).ok()
        })
        .collect();

    // make a directory under data-path for `operator_files`
    let operator_files_path = data_path.join("operator_files");
    fs::create_dir_all(&operator_files_path)?;

    // save the identity files operator.yaml, we'll need them in cases of twin tests
    val_cfg.iter().for_each(|v| {
      match validator_to_operator_config(v) {
        Ok(o) => {
          // use serde yaml to write the operator configuration to a file
          // including the address
          let operator_file = operator_files_path.join(format!("operator_{}.yaml", v.owner_account_address));
          match serde_yaml::to_string(&o) {
              Ok(yaml_str) => {
                  if let Err(e) = fs::write(&operator_file, yaml_str) {
                      eprintln!("Could not write operator file to {:?}: {}", operator_file, e);
                  } else {
                      println!("Wrote operator file to {:?}", operator_file);
                  }
              },
              Err(e) => eprintln!("Failed to serialize operator config: {}", e),
          }
        }
        Err(e) => {
            eprintln!("Failed to convert ValidatorConfiguration to OperatorConfiguration for validator {:?}: {}",
                v.owner_account_address, e);
        }
      }
    });

    // Determines the path for the recovery data.
    // NOTE: test fixtures located at ./tests/fixtures/sample_export_recovery.json
    let mut recovery = if let Some(p) = legacy_data_path {
        parse_json::recovery_file_parse(p)?
    } else {
        // this is probably a testnet, we need to minimally start the infra escrow
        // and balance on validators
        generate_testnet_state_for_vals(&val_cfg)
    };

    // Builds the genesis block with the specified configurations.
    genesis_builder::build(
        "none".to_string(), // we ignore ceremony coordination for testnet
        "none".to_string(),
        "none".to_string(),
        data_path.clone(),
        framework_mrb_path,
        &mut recovery,
        chain,
        Some(val_cfg),
    )?;
    Ok(data_path)
}

fn generate_testnet_state_for_vals(vals: &[ValidatorConfiguration]) -> Vec<LegacyRecoveryV6> {
    let mut recovery: Vec<LegacyRecoveryV6> = vec![];
    for v in vals {
        let mut l = LegacyRecoveryV6 {
            account: Some(v.owner_account_address.into()),
            auth_key: Some(AuthenticationKey::ed25519(&v.owner_account_public_key)),
            balance: Some(LegacyBalanceResourceV6 {
                coin: 10_000_000 * 10u64.pow(ONCHAIN_DECIMAL_PRECISION as u32),
            }),
            ..Default::default()
        };

        let p = PledgeAccountResource {
            address_of_beneficiary: AccountAddress::ONE,
            amount: 100_000_000 * 10u64.pow(ONCHAIN_DECIMAL_PRECISION as u32),
            pledge: 100_000_000 * 10u64.pow(ONCHAIN_DECIMAL_PRECISION as u32),
            epoch_of_last_deposit: 0,
            lifetime_pledged: 100_000_000 * 10u64.pow(ONCHAIN_DECIMAL_PRECISION as u32),
            lifetime_withdrawn: 0,
        };
        l.my_pledge = Some(MyPledgesResource { list: vec![p] });
        recovery.push(l);
    }

    recovery
}
