use crate::cli_output::TestInfo;
use anyhow::bail;
use diem_genesis::config::{HostAndPort, ValidatorConfiguration};
use libra_backwards_compatibility::legacy_recovery_v6::LegacyRecoveryV6;
use libra_config::validator_config;
use libra_genesis_tools::{genesis_builder, parse_json};
use libra_types::{
    core_types::{app_cfg::CONFIG_FILE_NAME, fixtures::TestPersona},
    exports::{AccountAddress, AuthenticationKey, NamedChain},
    move_resource::{
        cumulative_deposits::LegacyBalanceResourceV6,
        pledge_account::{MyPledgesResource, PledgeAccountResource},
    },
    ONCHAIN_DECIMAL_PRECISION,
};
use libra_wallet::account_keys;
use std::{fs, path::PathBuf};

// Sets up the environment for the given test persona.
// returns the home data path
pub async fn setup(
    host_list: &[HostAndPort],
    chain: NamedChain,
    data_dir: PathBuf,
    legacy_data_path: Option<PathBuf>,
    framework_mrb_path: Option<PathBuf>,
) -> anyhow::Result<Vec<TestInfo>> {
    // config the host address for this persona
    if host_list.len() < 3 {
        bail!("cannot start a testnet with less than 3 nodes, use --host-list for each of Alice, Bob, Carol and Dave but not more. Exiting.")
    }
    if host_list.len() > 4 {
        bail!("too many hosts provided, you just need 3 or 4 for a good testnet genesis. Exiting.")
    }

    let operator_files_path = data_dir.join("operator_files");
    fs::create_dir_all(&operator_files_path)?;

    // create validator configurations from fixtures
    // without needing to use a github repo to register and read
    let mut val_cfg: Vec<ValidatorConfiguration> = vec![];
    let mut test_info: Vec<TestInfo> = vec![];

    for (idx, host) in host_list.iter().enumerate() {
        let p = TestPersona::from(idx)?;
        let mnem = p.get_persona_mnem();

        let data_dir = operator_files_path.join(p.to_string());

        let (_, mut app_cfg) = validator_config::initialize_validator_files(
            Some(data_dir.clone()),
            Some(&p.to_string()),
            host.clone(),
            Some(mnem.clone()),
            false,
            Some(chain),
        )
        .await?;

        let w = account_keys::get_keys_from_mnem(mnem.clone())?;

        // Sets private key to file
        // DANGER: this is only for testnet
        app_cfg
            .get_profile_mut(None)
            .unwrap()
            .set_private_key(&w.child_0_owner.pri_key);

        let v_reg = genesis_builder::generate_validator_registration_config(mnem, host)?;
        val_cfg.push(v_reg);

        // set info output for testnet operator to find paths easily
        let o = TestInfo {
            validator_address: app_cfg.get_profile(None).unwrap().account,
            val_set_index: idx,
            data_dir,
            api_endpoint: host.to_owned(),
            app_cfg_path: app_cfg.workspace.node_home.join(CONFIG_FILE_NAME),
        };
        test_info.push(o);
    }

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
        data_dir.clone(),
        framework_mrb_path,
        &mut recovery,
        chain,
        Some(val_cfg),
    )?;

    Ok(test_info)
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
