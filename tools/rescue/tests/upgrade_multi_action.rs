mod support;

use diem_crypto::ValidCryptoMaterialStringExt;
use libra_query::query_view;
use libra_rescue::twin::{Twin, TwinSetup};
use libra_txs::{
    txs_cli::{TxsCli, TxsSub, TxsSub::Transfer},
    txs_cli_community::{CageTx, ClaimTx, CommunityTxs, InitTx},
};
use libra_types::core_types::app_cfg::TxCost;
use std::{env, path::PathBuf};

// TODO: Remove after offer structure is migrated
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn upgrade_multi_action_on_twin_db() -> Result<(), anyhow::Error> {
    // 1. create swarm with prod db copy
    let home_dir = env::var("HOME").expect("HOME environment variable not set");
    // TODO: Download the snapshot from github instead of using a local db copy
    let prod_db_to_clone = PathBuf::from(format!("{}/.libra/data/db", home_dir));
    assert!(prod_db_to_clone.exists());
    let (mut swarm, dir) = Twin::apply_with_rando_e2e(prod_db_to_clone, 4)
        .await
        .unwrap();
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");
    let api_endpoint = swarm.api_endpoint.clone();
    let client = swarm.client();

    // build the CLI command
    let mut cli_scaffold = TxsCli {
        subcommand: None,
        mnemonic: None,
        chain_id: None,
        config_path: Some(config_path.clone()),
        url: Some(api_endpoint.clone()),
        test_private_key: None,
        tx_profile: None,
        tx_cost: Some(TxCost::prod_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    // 1.1 create accounts
    let (accounts, addresses) = swarm.create_accounts(4).await?;

    // 1.2 fund accounts
    for (address, validator_private_key) in
        addresses.iter().zip(swarm.validator_private_keys.iter())
    {
        cli_scaffold.subcommand = Some(Transfer {
            to_account: *address,
            amount: 500.0,
        });
        cli_scaffold.test_private_key = Some(validator_private_key.clone());

        // execute the transfer
        cli_scaffold
            .run()
            .await
            .unwrap_or_else(|_| panic!("CLI could not transfer funds to account {}", address));
    }

    // 1.3 pick accounts to compose community wallet
    let cw_address = swarm.swarm.validators().last().unwrap().peer_id();
    let cw_pk = swarm
        .swarm
        .validators()
        .last()
        .unwrap()
        .account_private_key()
        .as_ref()
        .unwrap()
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");
    let cw_authorities = [&accounts[0], &accounts[1], &accounts[2]];
    let cw_authorities_addresses: Vec<_> = addresses.iter().take(3).cloned().collect();

    // 2. upgrade to the latest version with Offer structure
    support::upgrade_multiple_impl("upgrade-multi-lib", vec!["3-libra-framework"], &mut swarm)
        .await
        .unwrap();

    // 3. Verify migration
    let exists_offer = query_view::get_view(
        &client,
        "0x1::multi_action::exists_offer",
        None,
        Some(cw_address.clone().to_string()),
    )
    .await
    .expect("Query failed: query should have succeeded");
    assert!(
        !exists_offer.as_array().unwrap()[0].as_bool().unwrap(),
        "expected exists_offer to succeed"
    );

    // 4. Init Community Wallet
    cli_scaffold.subcommand = Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
        admins: cw_authorities_addresses.clone(),
        num_signers: 3,
    })));
    cli_scaffold.test_private_key = Some(cw_pk.clone());

    println!(">>> Running CLI gov init");
    cli_scaffold
        .run()
        .await
        .expect("CLI could not create community wallet, details: {:?}");

    // 5. Claim Offer
    for authority in cw_authorities {
        cli_scaffold.subcommand = Some(TxsSub::Community(CommunityTxs::GovClaim(ClaimTx {
            community_wallet: cw_address,
        })));
        cli_scaffold.test_private_key = Some(
            authority
                .private_key()
                .to_encoded_string()
                .expect("cannot decode pri key"),
        );
        cli_scaffold.run().await.expect("CLI could not claim offer");
    }

    // 6. Finalize and cage
    cli_scaffold.subcommand = Some(TxsSub::Community(CommunityTxs::GovCage(CageTx {
        num_signers: 2,
    })));
    cli_scaffold.test_private_key = Some(cw_pk.clone());
    cli_scaffold
        .run()
        .await
        .expect("CLI could not finalize and cage");

    // Ensure the account is now a multisig account
    let is_comm_wallet_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::is_multi_action",
        None,
        Some(cw_address.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet multisig check");

    assert!(
        is_comm_wallet_query_res.as_array().unwrap()[0]
            .as_bool()
            .unwrap(),
        "Account should be a multisig account"
    );

    // Ensure the new account authorities are set
    let authorities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(cw_address.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities = authorities_query_res[0].as_array().unwrap();
    assert_eq!(authorities.len(), 3, "Expected 3 authorities");
    for (i, authority) in authorities.iter().enumerate() {
        assert_eq!(
            authority.as_str().unwrap()[2..],
            cw_authorities_addresses[i].to_string(),
            "Unexpected authority"
        );
    }

    Ok(())
}
