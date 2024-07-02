use libra_smoke_tests::{configure_validator, helpers::get_libra_balance, libra_smoke::LibraSmoke};
use libra_txs::submit_transaction::Sender;
use libra_txs::txs_cli::{to_legacy_address, TxsCli, TxsSub::Transfer};
use libra_types::core_types::app_cfg::TxCost;
use libra_wallet::account_keys;

// Testing that we can send the minimal transaction: a transfer from one existing validator to another.
// Case 1: send to an existing account: another genesis validator
// Case 2: send to an account which does not yet exist, and the account gets created on chain.

/// Case 1: send to an existing account: another genesis validator
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn smoke_transfer_existing_account() {
    let d = diem_temppath::TempPath::new();

    //std::env::set_var("DIEM_FORGE_NODE_BIN_PATH", "/root/.cargo/diem-node");
    let mut s = LibraSmoke::new(Some(2), None) // going to transfer from validator #0 to validator #1
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    // let s = LibraSmoke::new(Some(2)).await.expect("can't start swarm");

    // 1. simple case: account already exitsts
    let recipient = s.swarm.validators().nth(1).unwrap().peer_id(); // sending to second genesis node.
    let cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: recipient,
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

    cli.run()
        .await
        .expect("cli could not send to existing account");
}

/// Case 2: send to an account which does not yet exist, and the account gets created on chain.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_transfer_create_account() -> Result<(), anyhow::Error> {
    let d = diem_temppath::TempPath::new();

    //std::env::set_var("DIEM_FORGE_NODE_BIN_PATH", "/root/.cargo/diem-node");
    let mut s = LibraSmoke::new(None, None)
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    let client = s.client();

    let marlon = s.marlon_rando().address();

    // case 2. Account does not yet exist.
    let cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: marlon,
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

    cli.run()
        .await
        .expect("cli could not create and transfer to new account");

    let bal = get_libra_balance(&client, marlon).await?;
    assert_eq!(
        bal.total, 1000000,
        "Balance of the new account should be 1.0(1000000) after the transfer"
    );
    Ok(())
}

/// Estimate only. Esitmates will fail if the coin name is not set in the diem-node compiled binary.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// #[ignore] // TODO: ignore this test until diem-platform 1.6.3 final lands
async fn smoke_transfer_estimate() {
    let d = diem_temppath::TempPath::new();

    let mut s = LibraSmoke::new(None, None)
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    // case 2. Account does not yet exist.
    let cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: s.marlon_rando().address(),
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_cheap_txs_cost()),
        estimate_only: true, // THIS IS THE TEST
        legacy_address: false,
    };

    cli.run().await.expect("could not get estimate");

    // NOTE: This should not fail
}

// create v5 and v6 accouunts from the same seed phrase
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn send_v6_v5() -> anyhow::Result<()> {
    // create libra swarm and get app config for the first validator
    let mut ls = LibraSmoke::new(Some(1), None)
        .await
        .expect("could not start libra smoke");
    let val_app_cfg = ls.first_account_app_cfg()?;

    // get an appcfg struct from Alice's mnemonic
    let alice = account_keys::get_keys_from_mnem("talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse".to_owned())?;
    let alice_acct_v6 = &alice.child_0_owner.account;

    {
        // create an account for alice by transferring funds
        let mut s = Sender::from_app_cfg(&val_app_cfg, None).await?;
        let res = s
            .transfer(alice.child_0_owner.account, 100.0, false)
            .await?
            .unwrap();
        assert!(res.info.status().is_success());
        println!(
            "alice v6: {:?} auth: {:?} pri: {:?}",
            alice.child_0_owner.account,
            alice.child_0_owner.auth_key.to_string(),
            alice.child_0_owner.pri_key.to_string(),
        );
    }

    let client = ls.client();

    let bal = get_libra_balance(&client, *alice_acct_v6).await?;
    assert_eq!(
        bal.total, 100000000,
        "Balance of the new account should be 100.0(100000000) after the transfer"
    );

    // create alice v5 style account and transfer some funds to it
    let alice_acc_v5 = to_legacy_address(alice_acct_v6).unwrap();

    {
        let mut s = Sender::from_app_cfg(&val_app_cfg, None).await?;
        let res = s.transfer(alice_acc_v5, 200.0, false).await?.unwrap();
        assert!(res.info.status().is_success());

        let bal = get_libra_balance(&client, alice_acc_v5).await?;
        assert_eq!(
            bal.total, 200000000,
            "Balance of the new account should be 200.0(200000000) after the transfer"
        );
    }

    Ok(())
}
