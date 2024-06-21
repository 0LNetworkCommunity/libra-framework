use std::path::PathBuf;

use diem_types::account_address::AccountAddress;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::{
    txs_cli::{
        TxsCli,
        TxsSub::{self, Transfer},
    },
    txs_cli_vals::ValidatorTxs,
};
use libra_types::core_types::app_cfg::TxCost;

// Scenario, a new user wants to become a validator.
// 1. the account needs to be created, and funded
// 2. the account registers validator settings
// 3. the account submits proof-of-fee bids
// 4. existing validators send vouches to the new account
// Expected behavior
// 1. new account will exist in the validator_universe state
// 2. on the epoch boundary the new validator set will include the new account.

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore] // TODO
async fn smoke_onboard_validator() -> anyhow::Result<()> {
    let d = diem_temppath::TempPath::new();

    let new_val_address = AccountAddress::from_hex_literal(
        "0x87515d94a244235a1433d7117bc0cb154c613c2f4b1e67ca8d98a542ee3f59f5",
    )?;

    let mut s = LibraSmoke::new(None, None)
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    // 1. CREATE THE ACCOUNT
    let alice_cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: new_val_address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    alice_cli
        .run()
        .await
        .expect("cli could not create and transfer to new account");

    // 2. REGISTER AS A VALIDATOR

    // the new validator needs their own cli struct
    let operator_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/validator_onboard/operator.yaml");

    let rando_cli = TxsCli {
        subcommand: Some(TxsSub::Validator(ValidatorTxs::Register {
            operator_file: Some(operator_file),
        })),
        mnemonic: None,
        test_private_key: Some(
            "0x74f18da2b80b1820b58116197b1c41f8a36e1b37a15c7fb434bb42dd7bdaa66b".to_owned(),
        ),
        chain_id: None,
        config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    rando_cli
        .run()
        .await
        .expect("cli could not register validator");

    Ok(())
}
