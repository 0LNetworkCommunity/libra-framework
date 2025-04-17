use diem_sdk::crypto::ValidCryptoMaterialStringExt;
use diem_sdk::types::LocalAccount;
use diem_temppath::TempPath;
use diem_types::account_address::AccountAddress;
use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::txs_cli::{TxsCli, TxsSub, TxsSub::Transfer};
use libra_txs::txs_cli_community::{
    AdminTx, CageTx, ClaimTx, CommunityTxs, InitTx, MigrateOfferTx, OfferTx,
};
use libra_types::core_types::app_cfg::TxCost;
use std::path::PathBuf;
use url::Url;

/*
/// TODO: Test the migration of an existing community wallet with n flag
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn migrate_community_wallet_with_flag() {
    let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;

    // Set up migrated v6 community wallet
    //TODO: Use mocked migrated state v6 community wallet

    let signers = get_signers(&s);

    //create wallet with -n flag option
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: Some(2)
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), //TODO: Use mocked migrated state v6 community wallet
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create a new Community Wallet with -n flag set");

    // Verify if the account is a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    Ok(())

}

/// TODO: Test the migration of an existing community wallet
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn migrate_community_wallet() {
    let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;

    // Set up migrated v6 community wallet
    //TODO: Use mocked migrated state v6 community wallet

    let signers = get_signers(&s);

    //create wallet with -n flag option
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), //TODO: Use mocked migrated state v6 community wallet
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create a new Community Wallet with -n flag set");

    // Verify if the account is a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    Ok(())
}

// TODO: Not passing likely due to donor_voice not adding the community wallet tag
// Create a V7 community wallet with -n flag
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn create_community_wallet_with_flag() -> Result<(), anyhow::Error> {
    let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;

    let signers = get_signers(&s);

    //create wallet with -n flag option
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: Some(2)
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), //Issue using the private key private_key.to_string()
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create a new Community Wallet with -n flag set");

    // Verify if the account is a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    Ok(())
}

// TODO: Need to find way to capture the panic
// Attempt to transfer from a community wallet account and bypass multisig
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn new_community_wallet_cant_transfer() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;

    // SETUP ADMIN SIGNERS
    // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
    // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

    // SETUP COMMUNITY WALLET
    // 3. Prepare a new admin account but do not immediately use it within the community wallet.
    // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
    // 5. Confirm the successful creation of the community wallet and its recognition by the system.
    // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

    // SETUP ADMIN SIGNERS //
    // We set up 5 new accounts and also fund them from each of the 5 validators

    let (signers, signer_addresses) = s.create_accounts(5).await?;

    // Ensure there's a one-to-one correspondence between signers and private keys
    if signer_addresses.len() != s.validator_private_keys.len() {
        panic!("The number of signer addresses does not match the number of validator private keys.");
    }

    for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
        let to_account = signer_address.clone();

        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        let cli_transfer = TxsCli {
            subcommand: Some(Transfer {
                to_account,
                amount: 10.0,
            }),
            mnemonic: None,
            test_private_key: Some(validator_private_key.clone()),
            chain_id: None,
            config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
            url: Some(s.api_endpoint.clone()),
            tx_profile: None,
            tx_cost: Some(TxCost::default_baseline_cost()),
            estimate_only: false,
        };

        // Execute the transfer
        cli_transfer.run()
            .await
            .expect(&format!("CLI could not transfer funds to account {}", signer_address));

    }

    // SETUP COMMUNITY WALLET //

    // Prepare new admin account
    let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

    let new_admin_address = AccountAddress::from_hex_literal(new_admin)
    .expect("Failed to parse account address");

    // Fund with the last signer to avoid ancestry issues
    let private_key_of_fifth_signer = signers[4]
    .private_key()
    .to_encoded_string()
    .expect("cannot decode pri key");

    // Transfer funds to ensure the account exists on-chain
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: new_admin_address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(private_key_of_fifth_signer),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_transfer.run()
    .await
    .expect("CLI could not transfer funds to the new account");

    // Get 3 signers to be admins
    let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
    .clone()
    .into_iter()
    .take(3)
    .collect();

    //create new community wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:first_three_signer_addresses,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create community wallet");

    // Verify if the account is a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    // Attempt to create a transfer from the new community wallet
    let transfer_cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: new_admin_address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    match transfer_cli.run().await {
        Ok(_) => {
            // Instead of panicking, you might want to assert that this scenario should not happen
            assert!(false, "Transfer succeeded unexpectedly. Expected an error.");
        },
        Err(e) => {
            // Check if the error is what you expected
            if e.to_string().contains("MoveAbort") && e.to_string().contains("196618") && e.to_string().contains("ENOT_FOR_CW") {
                // If the error matches the expected condition, you can consider this a successful test case
                println!("Received expected error: {:?}", e);
            } else {
                // If the error does not match, you should consider it an actual error case
                return Err(e);
            }
        },
    }

    Ok(())
}*/

// Create a v7 community wallet
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn create_community_wallet() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_pk, comm_wallet_addr) =
        setup_environment().await;
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");

    // SETUP ADMIN SIGNERS
    // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
    // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

    // SETUP COMMUNITY WALLET
    // 3. Prepare a new admin account but do not immediately use it within the community wallet.
    // 4. Initialize a community wallet offering the first three of the newly funded accounts as its admins.
    // 5. Update the community wallet with a new admin account.

    // SETUP ADMIN SIGNERS //
    // 1. Generate and fund 5 new accounts
    let (signers, signer_addresses) = s.create_accounts(5).await?;

    // Ensure there's a one-to-one correspondence between signers and private keys
    if signer_addresses.len() != s.validator_private_keys.len() {
        panic!(
            "The number of signer addresses does not match the number of validator private keys."
        );
    }

    // 2. Transfer funds to the newly created signer accounts
    for (signer_address, validator_private_key) in
        signer_addresses.iter().zip(s.validator_private_keys.iter())
    {
        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        run_cli_transfer(
            *signer_address,
            10.0,
            validator_private_key.clone(),
            s.api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }

    // SETUP COMMUNITY WALLET //

    // 3. Prepare a new admin account
    let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";
    let new_admin_address =
        AccountAddress::from_hex_literal(new_admin).expect("Failed to parse account address");

    // Fund with the last signer to avoid ancestry issues
    let private_key_of_fifth_signer = signers[4]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");

    // Transfer funds to ensure the account exists on-chain
    run_cli_transfer(
        new_admin_address,
        1.0,
        private_key_of_fifth_signer.clone(),
        s.api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // Get 3 signers to be admins
    let first_three_signer_addresses: Vec<AccountAddress> =
        signer_addresses.clone().into_iter().take(3).collect();

    // 4. Initialize the community wallet
    run_cli_community_init(
        comm_wallet_pk.clone(),
        first_three_signer_addresses.clone(),
        3,
        s.api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // Verify if the account is not a community wallet yet
    let is_comm_wallet_query_res = query_view::get_view(
        &s.client(),
        "0x1::community_wallet::is_init",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet init check");

    assert!(
        !is_comm_wallet_query_res.as_array().unwrap()[0]
            .as_bool()
            .unwrap(),
        "Account should not be a community wallet yet"
    );

    // Check offer proposed
    let proposed_query_res = query_view::get_view(
        &s.client(),
        "0x1::multi_action::get_offer_proposed",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet proposed offer");

    // Assert authorities are the three proposed
    let authorities = proposed_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(authorities.len(), 3, "There should be 3 authorities");
    for i in 0..3 {
        let authority_str = &authorities[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            authority_str,
            first_three_signer_addresses[i]
                .to_string()
                .trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    // 5. Admins claim the offer.
    for authority in signers.iter().take(3) {
        let authority_pk = authority
            .private_key()
            .to_encoded_string()
            .expect("cannot decode pri key");
        run_cli_claim_offer(
            authority_pk,
            comm_wallet_addr,
            s.api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }

    // Check offer claimed
    let proposed_query_res = query_view::get_view(
        &s.client(),
        "0x1::multi_action::get_offer_claimed",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet offer claimed");

    // Assert authorities are the three proposed
    let authorities = proposed_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(authorities.len(), 3, "There should be 3 authorities");
    for i in 0..3 {
        let authority_str = &authorities[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            authority_str,
            first_three_signer_addresses[i]
                .to_string()
                .trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    // 6. Donor finalize and cage the community wallet
    run_cli_community_cage(
        comm_wallet_pk.clone(),
        3,
        s.api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // Ensure the account is now a multisig account
    let is_comm_wallet_query_res = query_view::get_view(
        &s.client(),
        "0x1::multi_action::is_multi_action",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet multisig check");

    assert!(
        is_comm_wallet_query_res.as_array().unwrap()[0]
            .as_bool()
            .unwrap(),
        "Account should be a multisig account"
    );

    // Ensure authorities are the three proposed
    let authrotities_query_res = query_view::get_view(
        &s.client(),
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(authorities.len(), 3, "There should be 3 authorities");
    for i in 0..3 {
        let authority_str = &authorities[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            authority_str,
            first_three_signer_addresses[i]
                .to_string()
                .trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    Ok(())
}

// Happy day: update community wallet offer before cage
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn update_community_wallet_offer() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_pk, comm_wallet_addr) =
        setup_environment().await;
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");

    // SETUP ADMIN SIGNERS
    // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
    // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

    // SETUP COMMUNITY WALLET
    // 3. Prepare a new admin account but do not immediately use it within the community wallet.
    // 4. Initialize a community wallet offering the first three of the newly funded accounts as its admins.
    // 5. Admins claim the offer.
    // 6. Donor finalize and cage the community wallet to ensure its independence and security.

    // SETUP ADMIN SIGNERS //
    // 1. Generate and fund 5 new accounts
    let (signers, signer_addresses) = s.create_accounts(5).await?;

    // Ensure there's a one-to-one correspondence between signers and private keys
    if signer_addresses.len() != s.validator_private_keys.len() {
        panic!(
            "The number of signer addresses does not match the number of validator private keys."
        );
    }

    // 2. Transfer funds to the newly created signer accounts
    for (signer_address, validator_private_key) in
        signer_addresses.iter().zip(s.validator_private_keys.iter())
    {
        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        run_cli_transfer(
            *signer_address,
            10.0,
            validator_private_key.clone(),
            s.api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }

    // SETUP COMMUNITY WALLET //

    // 3. Prepare a new admin account
    let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";
    let new_admin_address =
        AccountAddress::from_hex_literal(new_admin).expect("Failed to parse account address");

    // Fund with the last signer to avoid ancestry issues
    let private_key_of_fifth_signer = signers[4]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");

    // Transfer funds to ensure the account exists on-chain
    run_cli_transfer(
        new_admin_address,
        1.0,
        private_key_of_fifth_signer.clone(),
        s.api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // Get 3 signers to be admins
    let mut authorities: Vec<AccountAddress> =
        signer_addresses.clone().into_iter().take(3).collect();

    // 4. Initialize the community wallet
    run_cli_community_init(
        comm_wallet_pk.clone(),
        authorities.clone(),
        3,
        s.api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // 5. Update the community wallet with a new admin account.

    // Add forth signer as admin
    let forth_signer_address = signer_addresses[3];
    authorities.push(forth_signer_address);
    run_cli_community_propose_offer(
        comm_wallet_pk.clone(),
        authorities.clone(),
        4,
        s.api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // Check offer proposed
    let proposed_query_res = query_view::get_view(
        &s.client(),
        "0x1::multi_action::get_offer_proposed",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet proposed offer");

    // Assert authorities are the three proposed
    let proposed = proposed_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(proposed.len(), 4, "There should be 4 authorities");
    for i in 0..4 {
        let proposed_str = &proposed[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            proposed_str,
            authorities[i].to_string().trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    Ok(())
}

/*

// TODO: apply once we have a method to progress epochs
// Propose and sign payment
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn community_wallet_payment() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;

    // SETUP ADMIN SIGNERS
    // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
    // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

    // SETUP COMMUNITY WALLET
    // 3. Prepare a new admin account but do not immediately use it within the community wallet.
    // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
    // 5. Confirm the successful creation of the community wallet and its recognition by the system.
    // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

    // EXECUTE PAYMENT PROPOSAL
    // 7. Propose a payment from the community wallet to a new worker account using one of the signer accounts.
    // Configure and execute payment proposal command.
    // 8. Validate the successful execution of the payment proposal by checking transaction receipt or wallet balance.

    // SETUP ADMIN SIGNERS //
    // Generate and fund 5 new accounts from validators for transaction signing
    let (signers, signer_addresses) = s.create_accounts(5).await?;

    // Ensure each signer has a corresponding private key
    if signer_addresses.len() != s.validator_private_keys.len() {
        panic!("The number of signer addresses does not match the number of validator private keys.");
    }

    // Transfer funds to newly created accounts to ensure they exist on-chain
    for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
        let to_account = signer_address.clone(); // Target account for funds transfer

        // Configure transfer command with validator's private key
        let cli_transfer = TxsCli {
            subcommand: Some(Transfer {
                to_account, // Destination account for this iteration
                amount: 10.0, // Specified transfer amount
            }),
            mnemonic: None,
            test_private_key: Some(validator_private_key.clone()), // Validator's private key for authentication
            chain_id: None,
            config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
            url: Some(s.api_endpoint.clone()),
            tx_profile: None,
            tx_cost: Some(TxCost::default_baseline_cost()),
            estimate_only: false,
        };

        // Execute funds transfer to the account
        cli_transfer.run()
            .await
            .expect(&format!("CLI could not transfer funds to account {}", signer_address));
    }

    // SETUP COMMUNITY WALLET //

    // Prepare a new admin account but do not use it immediately
    let new_worker = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";
    let new_worker_address = AccountAddress::from_hex_literal(new_worker)
        .expect("Failed to parse account address");

    // Transfer funds to new admin account to ensure its on-chain presence
    let private_key_of_fifth_signer = signers[4]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");

    // Configuration for transferring funds to new admin account
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: new_worker_address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(private_key_of_fifth_signer),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    // Execute the transfer to the new admin account
    cli_transfer.run()
        .await
        .expect("CLI could not transfer funds to the new account");

    // Select first three signers as initial community wallet admins
    let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
        .clone()
        .into_iter()
        .take(3)
        .collect();

    // Command to initialize community wallet with selected admins
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:first_three_signer_addresses,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    // Execute community wallet creation
    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create community wallet");

    // Verify the account is now a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");
    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    // Finalize community wallet setup by revoking original account's access
    let cli_finalize_cage = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::FinalizeAndCage(FinalizeCageTx {}))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };
    cli_finalize_cage.run()
        .await
        .expect("CLI could not finalize and cage the community wallet");

    // EXECUTE PAYMENT PROPOSAL //

    // Propose a payment from the community wallet to the new worker account
    let private_key_of_first_signer = signers[1]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");

    // Configure payment proposal command
    let cli_propose_payment = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
            community_wallet: comm_wallet_addr,
            recipient: new_worker_address,
            amount: 10,
            description: "Thanks Mate".to_string(),
        }))),
        mnemonic: None,
        test_private_key: Some(private_key_of_first_signer),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,    };

    // Execute the payment proposal
    cli_propose_payment.run()
        .await
        .expect("CLI could not propose payment");

    Ok(())
}

*/

// Add an admin
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn add_community_wallet_admin() -> Result<(), anyhow::Error> {
    // 1. Setup environment
    let (mut smoke, dir, _account_address, comm_wallet_pk, comm_wallet_addr) =
        setup_environment().await;
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");
    let api_endpoint = smoke.api_endpoint.clone();
    let client = smoke.client();

    // 2. Setup 4 funded accounts
    let (signers, addresses) = smoke.create_accounts(5).await?;
    for (signer_address, validator_private_key) in
        addresses.iter().zip(smoke.validator_private_keys.iter())
    {
        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        run_cli_transfer(
            *signer_address,
            10.0,
            validator_private_key.clone(),
            smoke.api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }

    // 3. Setup community wallet caged with 3 authorities and 2 signitures
    let initial_authorities: Vec<_> = signers.iter().take(3).collect();
    setup_community_wallet_caged(
        comm_wallet_pk.clone(),
        comm_wallet_addr,
        &initial_authorities,
        2,
        config_path.clone(),
        api_endpoint.clone(),
    )
    .await;

    // 4. The first authority propose a new community wallet admin and 3 signitures
    let new_admin_address = addresses[3];
    let new_admin_pk = signers[3]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");
    let private_key_of_first_signer = signers[0]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key")
        .clone();

    let cli_add_new_admin_proposal = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
            community_wallet: comm_wallet_addr,
            admin: new_admin_address,
            drop: Some(true),
            n: 3,
            epochs: Some(10),
        }))),
        mnemonic: None,
        test_private_key: Some(private_key_of_first_signer),
        chain_name: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_add_new_admin_proposal
        .run()
        .await
        .expect("CLI could not add new admin to community wallet");

    // Verify the admins remain unchanged
    let authrotities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities_queried = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(
        authorities_queried.len(),
        3,
        "There should be 3 authorities"
    );

    let authorities_addresses: Vec<AccountAddress> =
        initial_authorities.iter().map(|a| a.address()).collect();
    for i in 0..3 {
        let authority_str = &authorities_queried[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            authority_str,
            authorities_addresses[i].to_string().trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    // 5. All the other authorities vote to add the new admin and change threshold to 3
    for authority in initial_authorities.iter().take(3).skip(1) {
        let private_key_of_signer = authority
            .private_key()
            .to_encoded_string()
            .expect("cannot decode pri key");
        let cli_add_new_admin_proposal = TxsCli {
            subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
                community_wallet: comm_wallet_addr,
                admin: new_admin_address,
                drop: Some(true),
                n: 3,
                epochs: Some(10),
            }))),
            mnemonic: None,
            test_private_key: Some(private_key_of_signer),
            chain_name: None,
            config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
            url: Some(api_endpoint.clone()),
            tx_profile: None,
            tx_cost: Some(TxCost::default_baseline_cost()),
            estimate_only: false,
            legacy_address: false,
        };

        cli_add_new_admin_proposal
            .run()
            .await
            .expect("CLI could not add new admin to community wallet");
    }

    // Verify the admins remain unchanged
    let authrotities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities_queried = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(
        authorities_queried.len(),
        3,
        "There should be 3 authorities"
    );

    let authorities_addresses: Vec<AccountAddress> =
        initial_authorities.iter().map(|a| a.address()).collect();
    for i in 0..3 {
        let authority_str = &authorities_queried[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            authority_str,
            authorities_addresses[i].to_string().trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    // 6. New admin claim the offer
    run_cli_claim_offer(
        new_admin_pk,
        comm_wallet_addr,
        api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // 7. Validate the new admin was added
    let authrotities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities_queried = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(
        authorities_queried.len(),
        4,
        "There should be 4 authorities"
    );

    let new_authorities_addresses: Vec<AccountAddress> =
        signers.iter().take(4).map(|a| a.address()).collect();
    for i in 0..4 {
        let authority_str = &authorities_queried[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            new_authorities_addresses[i]
                .to_string()
                .trim_start_matches('0'),
            authority_str,
            "Authority should be the same"
        );
    }

    // Verify the number of signitures have changed to 3
    let query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_threshold",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let query_ret = query_res.as_array().unwrap();
    assert_eq!(query_ret[0], "3", "There should be 3 signitures");
    assert_eq!(query_ret[1], "4", "There should be 3 signers");

    Ok(())
}

// Remove an admin
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn remove_community_wallet_admin() -> Result<(), anyhow::Error> {
    // 1. Setup environment
    let (mut smoke, dir, _account_address, comm_wallet_pk, comm_wallet_addr) =
        setup_environment().await;
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");
    let api_endpoint = smoke.api_endpoint.clone();
    let client = smoke.client();

    // 2. Setup 4 funded accounts
    let (signers, addresses) = smoke.create_accounts(4).await?;
    for (signer_address, validator_private_key) in
        addresses.iter().zip(smoke.validator_private_keys.iter())
    {
        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        run_cli_transfer(
            *signer_address,
            10.0,
            validator_private_key.clone(),
            smoke.api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }

    // 3. Setup community wallet caged with 4 authorities and 3 signitures
    let initial_authorities: Vec<_> = signers.iter().take(4).collect();
    setup_community_wallet_caged(
        comm_wallet_pk.clone(),
        comm_wallet_addr,
        &initial_authorities,
        3,
        config_path.clone(),
        api_endpoint.clone(),
    )
    .await;

    // Verify the cw #admins
    let authrotities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities_queried = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(
        authorities_queried.len(),
        4,
        "There should be 4 authorities"
    );

    // 4. The first authority propose to remove the forth admin and set signitures threshold to 2
    let admin_to_remove = addresses[3];
    let private_key_of_first_signer = signers[0]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key")
        .clone();

    let cli_add_new_admin_proposal = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
            community_wallet: comm_wallet_addr,
            admin: admin_to_remove,
            drop: Some(false),
            n: 2,
            epochs: Some(10),
        }))),
        mnemonic: None,
        test_private_key: Some(private_key_of_first_signer),
        chain_name: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_add_new_admin_proposal
        .run()
        .await
        .expect("CLI could not add new admin to community wallet");

    // Verify the admins remain unchanged
    let authrotities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities_queried = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(
        authorities_queried.len(),
        4,
        "There should be 4 authorities"
    );

    let authorities_addresses: Vec<AccountAddress> =
        initial_authorities.iter().map(|a| a.address()).collect();
    for i in 0..4 {
        println!("{:?}", authorities_queried[i]);
        let authority_str = &authorities_queried[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            authority_str,
            authorities_addresses[i].to_string().trim_start_matches('0'),
            "Authority should be the same"
        );
    }

    // 5. All the other authorities vote to remove the third admin and change threshold to 2
    for authority in initial_authorities.iter().take(3).skip(1) {
        let private_key_of_signer = authority
            .private_key()
            .to_encoded_string()
            .expect("cannot decode pri key");
        let cli_add_new_admin_proposal = TxsCli {
            subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
                community_wallet: comm_wallet_addr,
                admin: admin_to_remove,
                drop: Some(false),
                n: 2,
                epochs: Some(10),
            }))),
            mnemonic: None,
            test_private_key: Some(private_key_of_signer),
            chain_name: None,
            config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
            url: Some(api_endpoint.clone()),
            tx_profile: None,
            tx_cost: Some(TxCost::default_baseline_cost()),
            estimate_only: false,
            legacy_address: false,
        };

        cli_add_new_admin_proposal
            .run()
            .await
            .expect("CLI could not add new admin to community wallet");
    }

    // 7. Validate the third admin was removed
    let authrotities_query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_authorities",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let authorities_queried = authrotities_query_res.as_array().unwrap()[0]
        .as_array()
        .unwrap();
    assert_eq!(
        authorities_queried.len(),
        3,
        "There should be 3 authorities"
    );

    let new_authorities_addresses: Vec<AccountAddress> =
        signers.iter().take(3).map(|a| a.address()).collect();
    for i in 0..3 {
        let authority_str = &authorities_queried[i].as_str().unwrap()[2..]; // Remove the "0x" prefix
        assert_eq!(
            new_authorities_addresses[i]
                .to_string()
                .trim_start_matches('0'),
            authority_str,
            "Authority should be the same"
        );
    }

    // Verify the number of signitures have changed to 3
    let query_res = query_view::get_view(
        &client,
        "0x1::multi_action::get_threshold",
        None,
        Some(comm_wallet_addr.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet authorities check");

    let query_ret = query_res.as_array().unwrap();
    assert_eq!(query_ret[0], "2", "There should be 2 signitures");
    assert_eq!(query_ret[1], "3", "There should be 3 signers");

    Ok(())
}
/*
// TODO: apply once we have a method to progress epochs
// Veto a payment
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn cancel_community_wallet_payment() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;
    let client = s.client();

    // SETUP ADMIN SIGNERS
    // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
    // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

    // SETUP COMMUNITY WALLET
    // 3. Prepare a new admin account but do not immediately use it within the community wallet.
    // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
    // 5. Confirm the successful creation of the community wallet and its recognition by the system.
    // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

    // ADD NEW ADMIN
    // 7. Initiate the process to add a new admin to the community wallet by proposing through an existing admin.
    // 8. Validate the addition of the new admin by checking the updated count of admins/signers in the wallet.

    // PROPOSAL AND FUNDING OPERATIONS
    // 6. Propose payments to the new worker account from the community wallet using two of the signers.
    // 7. Assert the balance of the new worker account to ensure payments are not yet processed.
    // 8. Check and assert the balance of the community wallet before depositing additional funds.
    // 9. Fund the community wallet with an additional amount to assert governance.

    // VETOING A PAYMENT
    // 10. Veto the previously proposed payment, effectively canceling it.
    // 11. (Optional steps for further validation and checks, e.g., verifying veto state, could be added here.)

    // SETUP ADMIN SIGNERS //
    // We set up 5 new accounts and also fund them from each of the 5 validators

    let (signers, signer_addresses) = s.create_accounts(5).await?;

    // Ensure there's a one-to-one correspondence between signers and private keys
    if signer_addresses.len() != s.validator_private_keys.len() {
        panic!("The number of signer addresses does not match the number of validator private keys.");
    }

    for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
        let to_account = signer_address.clone(); // Adjust this line if necessary

        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        let cli_transfer = TxsCli {
            subcommand: Some(Transfer {
                to_account,
                amount: 10.0, // Adjust the amount as needed
            }),
            mnemonic: None,
            test_private_key: Some(validator_private_key.clone()), // Use the corresponding validator's private key
            chain_id: None,
            config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
            url: Some(s.api_endpoint.clone()),
            tx_profile: None,
            tx_cost: Some(TxCost::default_baseline_cost()),
            estimate_only: false,
        };

        // Execute the transfer
        cli_transfer.run()
            .await
            .expect(&format!("CLI could not transfer funds to account {}", signer_address));
    }

    // SETUP COMMUNITY WALLET //

    // Prepare new admin account
    let new_worker = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

    let new_worker_address = AccountAddress::from_hex_literal(new_worker)
    .expect("Failed to parse account address");

    // Fund with the last signer to avoid ancestry issues
    let private_key_of_fifth_signer = signers[4]
    .private_key()
    .to_encoded_string()
    .expect("cannot decode pri key");

    // Transfer funds to ensure the account exists on-chain
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: new_worker_address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(private_key_of_fifth_signer),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_transfer.run()
    .await
    .expect("CLI could not transfer funds to the new account");

    // Get 3 signers to be admins
    let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
    .clone()
    .into_iter()
    .take(3)
    .collect();

    //create new community wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:first_three_signer_addresses,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create community wallet");

    // Verify if the account is a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    // Remove the ability for the original account to access
    let cli_finalize_cage = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::FinalizeAndCage(FinalizeCageTx {}))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_finalize_cage.run()
        .await
        .expect("CLI could not finalize and cage the community wallet");

    let private_key_of_first_signer = signers[1]
    .private_key()
    .to_encoded_string()
    .expect("cannot decode pri key");

    // Propose payment signer 1
    let cli_propose_payment = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
            community_wallet: comm_wallet_addr,
            recipient: new_worker_address,
            amount: 10,
            description: "Thanks Mate".to_string(),
        }))),
        mnemonic: None,
        test_private_key: Some(private_key_of_first_signer),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_propose_payment.run()
    .await
    .expect("CLI could not propose payment from community wallet");

    let bal = get_libra_balance(&client, new_worker_address).await?;
    assert_eq!(bal.total, 1000000, "Balance of the account(1000000) should not be processed yet");

    let private_key_of_second_signer = signers[2]
    .private_key()
    .to_encoded_string()
    .expect("cannot decode pri key");

    // Propose payment signer 2
    let cli_propose_payment_signer_two = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
            community_wallet: comm_wallet_addr,
            recipient: new_worker_address,
            amount: 10,
            description: "Thanks Mate".to_string(),
        }))),
        mnemonic: None,
        test_private_key: Some(private_key_of_second_signer),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_propose_payment_signer_two.run()
    .await
    .expect("CLI could not propose payment from community wallet");

    let bal = get_libra_balance(&client, new_worker_address).await?;
    assert_eq!(bal.total, 1000000, "Balance of the account(1000000) should not be processed yet");

    //TODO: How to get the uid to query a community wallet
    // Check multisig proposal state
    // let is_payment_scheduled_query_res = query_view::get_view(&s.client(), "0x1::donor_voice_tx::get_multisig_proposal_state", Some(comm_wallet_addr.to_string()), Some("0".to_string()))
    // .await
    // .expect("Query failed: Can not get proposal state");
    // println!{"{:#?}", is_payment_scheduled_query_res};

    //assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account state should be in scheduled");

    // verify balance of community wallet prior to depositing
    let bal = get_libra_balance(&client, comm_wallet_addr).await?;
    assert_eq!(bal.total, 889049874, "Balance of the account(889049874) does not match");

    // Fund with the community wallet with the last signer to assert governance on the community wallet
    let private_key_of_forth_signer = signers[3]
    .private_key()
    .to_encoded_string()
    .expect("cannot decode pri key");

    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: comm_wallet_addr,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(private_key_of_forth_signer.clone()), // Use the corresponding validator's private key
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    // Execute the transfer
    cli_transfer.run()
        .await
        .expect(&format!("CLI could not transfer funds to account {}", comm_wallet_addr));

    let bal = get_libra_balance(&client, comm_wallet_addr).await?;
    assert_eq!(bal.total, 890049874, "Balance of the account(890049874) after transfer does not match");

    // Veto the payment to the community wallet
    let cli_veto_tx = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Veto(VetoTx {
            community_wallet: comm_wallet_addr,
            proposal_id: 0,
        }))),
        mnemonic: None,
        test_private_key: Some(private_key_of_forth_signer.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    // Execute the VetoTx command
    cli_veto_tx.run()
    .await
    .expect("CLI could not execute veto transaction");

    //TODO: Check veto state
    // Check multisig proposal state
    // let vote_state_query_res = query_view::get_view(&s.client(), "donor_voice_governance::get_veto_tally", Some(comm_wallet_addr.to_string()), Some("0".to_string()))
    // .await
    // .expect("Query failed: Can not get veto state");
    // println!{"{:#?}", vote_state_query_res};

    //assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account state should be in scheduled");

    Ok(())
}

// TODO: Create test
// Liquidate a community wallet
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn liquidate_community_wallet() {
}
*/

// UTILITY //

async fn setup_environment() -> (LibraSmoke, TempPath, AccountAddress, String, AccountAddress) {
    let dir = diem_temppath::TempPath::new();
    let mut s = LibraSmoke::new(Some(5), None)
        .await
        .expect("Could not start libra smoke");

    configure_validator::init_val_config_files(&mut s.swarm, 0, Some(dir.path().to_owned()))
        .expect("Could not initialize validator config");

    let account_address = "0x029633a96b0c0e81cc26cf2baefdbd479dab7161fbd066ca3be850012342cdee";

    let account_address_wrapped =
        AccountAddress::from_hex_literal(account_address).expect("Failed to parse account address");

    // Transfer funds to ensure the account exists on-chain
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: account_address_wrapped,
            amount: 100.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_name: None,
        config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_transfer
        .run()
        .await
        .expect("CLI could not transfer funds to the new account");

    // get the address of the first node, the private key that was used to create the comm wallet
    let first_node = s
        .swarm
        .validators()
        .next()
        .expect("no first validator")
        .to_owned();
    let comm_wallet_addr = first_node.peer_id();
    let comm_wallet_pk = first_node
        .account_private_key()
        .as_ref()
        .unwrap()
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");

    (
        s,
        dir,
        account_address_wrapped,
        comm_wallet_pk,
        comm_wallet_addr,
    )
}

async fn run_cli_transfer(
    to_account: AccountAddress,
    amount: f64,
    private_key: String,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    // Build the CLI command
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer { to_account, amount }),
        mnemonic: None,
        test_private_key: Some(private_key),
        chain_name: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    // Execute the transfer
    cli_transfer
        .run()
        .await
        .unwrap_or_else(|_| panic!("CLI could not transfer funds to account {}", to_account));
}

async fn run_cli_community_init(
    donor_private_key: String,
    admins: Vec<AccountAddress>,
    num_signers: u64,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    // Build the CLI command
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins,
            num_signers,
        }))),
        mnemonic: None,
        test_private_key: Some(donor_private_key),
        chain_name: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    // Execute the transaction
    cli_set_community_wallet
        .run()
        .await
        .expect("CLI could not create community wallet");
}

async fn run_cli_claim_offer(
    signer_pk: String,
    community_address: AccountAddress,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    let cli_claim_offer = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovClaim(ClaimTx {
            community_wallet: community_address,
        }))),
        mnemonic: None,
        test_private_key: Some(signer_pk),
        chain_name: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_claim_offer
        .run()
        .await
        .expect("CLI could not claim offer");
}

async fn run_cli_community_cage(
    donor_private_key: String,
    num_signers: u64,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    let cli_finalize_cage = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovCage(CageTx {
            num_signers,
        }))),
        mnemonic: None,
        test_private_key: Some(donor_private_key),
        chain_name: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_finalize_cage
        .run()
        .await
        .expect("CLI could not finalize and cage community wallet");
}

async fn run_cli_community_propose_offer(
    donor_private_key: String,
    admins: Vec<AccountAddress>,
    num_signers: u64,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    let cli_propose_offer = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovOffer(OfferTx {
            admins,
            num_signers,
        }))),
        mnemonic: None,
        test_private_key: Some(donor_private_key),
        chain_name: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_propose_offer
        .run()
        .await
        .expect("CLI could not propose offer");
}

// Helper to setup a community wallet caged with a given number of authorities and signitures
async fn setup_community_wallet_caged(
    donor_pk: String,
    donor_address: AccountAddress,
    authorities: &[&LocalAccount],
    num_signitures: u64,
    config_path: PathBuf,
    api_endpoint: Url,
) {
    // 1. Initialize the community wallet
    let authorities_addresses = authorities.iter().map(|a| a.address()).collect();
    run_cli_community_init(
        donor_pk.clone(),
        authorities_addresses,
        num_signitures,
        api_endpoint.clone(),
        config_path.clone(),
    )
    .await;

    // 2. Admins claim the offer.
    for auth in authorities {
        // print private key
        let authority_pk = auth
            .private_key()
            .to_encoded_string()
            .expect("cannot decode pri key");
        run_cli_claim_offer(
            authority_pk,
            donor_address,
            api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }

    // 3. Donor finalize and cage the community wallet
    run_cli_community_cage(donor_pk, num_signitures, api_endpoint, config_path).await;
}

// Test Offer migration of a legacy account
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_offer_migration() -> Result<(), anyhow::Error> {
    // 1. Setup environment
    let (mut smoke, dir, _account_address, _, _) = setup_environment().await;
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");
    let api_endpoint = smoke.api_endpoint.clone();
    // let client = smoke.client();

    // 2. Setup legacy account
    let (signers, addresses) = smoke.create_accounts(1).await?;
    for (signer_address, validator_private_key) in
        addresses.iter().zip(smoke.validator_private_keys.iter())
    {
        // Transfer funds to ensure the account exists on-chain using the specific validator's private key
        run_cli_transfer(
            *signer_address,
            10.0,
            validator_private_key.clone(),
            smoke.api_endpoint.clone(),
            config_path.clone(),
        )
        .await;
    }
    let community_wallet_pk = signers[0]
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");
    let community_wallet_address = addresses[0];

    // 3. Initialize deprecated governance
    let init_gov_deprecated = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInitDeprectated)),
        mnemonic: None,
        test_private_key: Some(community_wallet_pk.clone()),
        chain_name: None,
        config_path: Some(config_path.clone()),
        url: Some(api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    init_gov_deprecated
        .run()
        .await
        .expect("CLI could not propose offer");

    // Certify the account does not have an offer
    let is_offer_query_res = query_view::get_view(
        &smoke.client(),
        "0x1::multi_action::exists_offer",
        None,
        Some(community_wallet_address.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet offer check");

    assert!(
        !is_offer_query_res.as_array().unwrap()[0].as_bool().unwrap(),
        "Account should not have an offer"
    );

    // 4. Run offer migration
    let offer_migration = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Migration(MigrateOfferTx {
            community_wallet: community_wallet_address,
        }))),
        mnemonic: None,
        test_private_key: Some(community_wallet_pk),
        chain_name: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    offer_migration
        .run()
        .await
        .expect("CLI could not propose offer");

    // certify the account has an offer
    let is_offer_query_res = query_view::get_view(
        &smoke.client(),
        "0x1::multi_action::exists_offer",
        None,
        Some(community_wallet_address.clone().to_string()),
    )
    .await
    .expect("Query failed: community wallet offer check");

    assert!(
        is_offer_query_res.as_array().unwrap()[0].as_bool().unwrap(),
        "Account should have an offer"
    );

    Ok(())
}
