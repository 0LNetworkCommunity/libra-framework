use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::txs_cli::{
    TxsCli,
    TxsSub::{GenerateTransaction, Publish},
};
use libra_types::{core_types::app_cfg::TxCost, type_extensions::client_ext::ClientExt};

use diem::common::types::MovePackageDir;
use std::{path::PathBuf, str::FromStr};
/// Testing that a smart contract can be published. It should be possible for:
/// 1) the genesis validator to build and publish a fixture Move module ("tests/fixtures/test_publish").
/// 2) any account should be able to change state on that contract.
/// 3) any client should be able to call a view function on that contract.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_publish() {
    let d = diem_temppath::TempPath::new();

    let mut s = LibraSmoke::new(Some(2), None)
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    // 1) the genesis validator to build and publish a fixture Move module ("tests/fixtures/test_publish").

    let path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let mut move_package = MovePackageDir::new(path.join("tests/fixtures/test_publish"));

    let val_addr_string = s.first_account.address().to_string();
    move_package.add_named_address("this_address".to_string(), val_addr_string.clone());

    let mut cli = TxsCli {
        subcommand: Some(Publish(move_package)),
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

    cli.run().await.expect("cli could not publish contract");

    // 2. now that the contract is published lets add some state to it
    cli.subcommand = Some(GenerateTransaction {
        function_id: format!("0x{}::message::set_message", &val_addr_string),
        type_args: None,
        args: Some("42u64".to_string()),
    });

    cli.run()
        .await
        .expect("cli could not call deployed contract function");

    // 2. now that the contract is published lets add some state to it
    cli.subcommand = Some(GenerateTransaction {
        function_id: format!("0x{}::message::set_message", &val_addr_string),
        type_args: None,
        args: Some("42u64".to_string()),
    });

    cli.run()
        .await
        .expect("cli could not call deployed contract function");

    let res = s
        .client()
        .view_ext(
            &format!("0x{}::message::read", &val_addr_string),
            None,
            Some(format!("0x{}", &val_addr_string)),
        )
        .await
        .expect("could not run view function");

    let de: Vec<String> = serde_json::from_value(res).unwrap();

    assert_eq!(de, vec!["42".to_string()]);
}
