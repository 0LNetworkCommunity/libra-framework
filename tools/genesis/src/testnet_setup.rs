use crate::{genesis_builder, parse_json};
use anyhow::bail;
use diem_genesis::config::{HostAndPort, ValidatorConfiguration};
use libra_backwards_compatibility::legacy_recovery_v6::LegacyRecoveryV6;
use libra_config::validator_config;
use libra_types::{
    core_types::fixtures::TestPersona,
    exports::{AccountAddress, AuthenticationKey, NamedChain},
    move_resource::{
        cumulative_deposits::LegacyBalanceResourceV6,
        pledge_account::{MyPledgesResource, PledgeAccountResource},
    },
    ONCHAIN_DECIMAL_PRECISION,
};
use std::{fs, path::PathBuf, thread, time};

// Sets up the environment for the given test persona.
pub async fn setup(
    me: &TestPersona,
    host_list: &[HostAndPort],
    chain: NamedChain,
    data_path: PathBuf,
    legacy_data_path: Option<PathBuf>,
    framework_mrb_path: Option<PathBuf>,
) -> anyhow::Result<()> {
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
        data_path,
        framework_mrb_path,
        &mut recovery,
        chain,
        Some(val_cfg),
    )?;
    Ok(())
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
