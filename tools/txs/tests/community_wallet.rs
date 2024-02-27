use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke, helpers::get_libra_balance};
use libra_types::legacy_types::app_cfg::TxCost;
use libra_txs::txs_cli_community::{ CommunityTxs, InitTx, ProposeTx, AdminTx}; //, VetoTx, 
use libra_txs::txs_cli::{TxsCli, TxsSub, TxsSub::Transfer};
use libra_query::query_view;
use diem_types::account_address::AccountAddress;
use diem_crypto::ValidCryptoMaterialStringExt;
use diem_temppath::TempPath;




/// Test the migration of an existing community wallet with n flag
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
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
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

/// Test the migration of an existing community wallet
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
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
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
/// Create a V7 community wallet with -n flag
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
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
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

// TODO: Workout error codes for move
/// Attempt to transfer from a community wallet account and bypass multisig
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn new_community_wallet_cant_transfer() -> Result<(), anyhow::Error> {
    let (mut s, dir, account_address) = setup_environment().await;

    // Create a new wallet instance
    let mut wallet = WalletLibrary::new();

    // Generate a new address
    let (auth_key, _child_number) = wallet.new_address()?;

    // Get the derived address from the authentication key
    let address = auth_key.derived_address();

    let signers = get_signers(&s);

    // Add acc on chain
    let transfer_cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    transfer_cli.run()
        .await
        .expect("cli could not send to existing account");

    //create new wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), //Issue using the private key private_key.to_string()
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create community wallet");

    // Attempt to create a transfer from the new community wallet 
    let transfer_cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: address,
            amount: 1.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    //TODO: handle move errors using diem framework package
    transfer_cli.run()
    .await {
        Ok(_) => panic!("Expected an error, but the operation succeeded"),
        Err(e) => {
            // Check if the error is the one you expect
            if let Some(error_info) = e.downcast_ref::<YourErrorType>() {
                // Check for specific error properties, such as code, location, etc.
                assert_eq!(error_info.code, 196618, "Unexpected error code");
                // Additional checks can be added here
            } else {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    Ok(())
}

/// PASSING
/// Create a V7 community wallet
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn create_community_wallet() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;

    let signers = get_signers(&s);

    //create new wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), //Issue using the private key private_key.to_string()
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create community wallet");

    // Verify if the account is a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    Ok(())
}

// TODO: progress epochs
/// Propose and sign payment
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn community_wallet_payment() -> Result<(), anyhow::Error> {
    let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;
    let client = s.client();

    let signers = get_signers(&s);

    //create new wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
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

    // Get signer 1 private key
    let first_signer = get_private_key_at_n(&s, 2);

    
    // Propose payment signer 1
    let cli_propose_payment = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
            community_wallet: comm_wallet_addr,
            recipient: account_address,
            amount: 10,
            description: "Thanks Mate".to_string(),
        }))),
        mnemonic: None,
        test_private_key: Some(first_signer?), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_propose_payment.run()
    .await
    .expect("CLI could not propose payment from community wallet");


    let bal = get_libra_balance(&client, account_address).await?;
    assert_eq!(bal.total, 100000000, "Balance of the account(100000000) should not be processed yet");


    // Get signer 2 private key
    let second_signer = get_private_key_at_n(&s, 2);

    // Propose payment signer 2
    let cli_propose_payment_signer_two = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
            community_wallet: comm_wallet_addr,
            recipient: account_address,
            amount: 10,
            description: "Thanks Mate".to_string(),
        }))),
        mnemonic: None,
        test_private_key: Some(second_signer?), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    let bal = get_libra_balance(&client, account_address).await?;
    assert_eq!(bal.total, 100000000, "Balance of the account(100000000) should not be processed yet");



    cli_propose_payment_signer_two.run()
    .await
    .expect("CLI could not propose payment from community wallet");




    Ok(())
}

/// TODO: insufficient balance
///Add an admin
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn add_community_wallet_admin() -> Result<(), anyhow::Error> {
    let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;
    let client = s.client();

    let signers = get_signers(&s);

    // Mint and unlock tokens for the signers.
    s.mint_and_unlock(*signers.get(0).expect("Expected at least one signer"), 1000).await?;
    s.mint_and_unlock(*signers.get(1).expect("Expected at least one signer"), 1000).await?;
    s.mint_and_unlock(*signers.get(2).expect("Expected at least one signer"), 1000).await?;


    // Prepare new admin account
    let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

    let new_admin_address = AccountAddress::from_hex_literal(new_admin)
        .expect("Failed to parse account address");

    // Transfer funds to ensure the account exists on-chain
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: new_admin_address,
            amount: 10.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_transfer.run()
    .await
    .expect("CLI could not transfer funds to the new account");


    //create new wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers.clone(),
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
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

    // Get signer 1 private key
    let first_signer = get_private_key_at_n(&s, 2);



    //verify we have enough balance
    let first_signer_address = signers.get(1).expect("Expected at least one signer"); 
    let bal = get_libra_balance(&client, *first_signer_address).await?;
     assert_eq!(bal.total, 1000, "Balance of the new account should be 1.0(1000000) after the transfer");

    
    // Propose add admin
    let cli_add_new_admin_proposal = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
            community_wallet: comm_wallet_addr,
            admin: new_admin_address,
            drop: Some(true),
            n: 2,
            epochs: Some(10), 
        }))),
        mnemonic: None,
        test_private_key: Some(first_signer?), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_add_new_admin_proposal.run()
    .await
    .expect("CLI could not propose new admin to community wallet");


    // Verify the admins remain unchanged
    let comm_wallet_signers = query_view::get_view(&s.client(), "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

        println!("{:?}", comm_wallet_signers);

    assert_eq!(comm_wallet_signers.len(), 3, "The number of signers should be 3");


    // Get signer 2 private key
    let second_signer = get_private_key_at_n(&s, 2);

    // Singer 2 verify new admin
    let cli_add_new_admin_proposal = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
            community_wallet: comm_wallet_addr,
            admin: new_admin_address,
            drop: Some(true),
            n: 2,
            epochs: Some(10), 
        }))),
        mnemonic: None,
        test_private_key: Some(second_signer?), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_add_new_admin_proposal.run()
    .await
    .expect("CLI could not add new admin to community wallet");

    // Verify the admins have dropped
    let comm_wallet_signers = query_view::get_view(&s.client(), "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert_eq!(comm_wallet_signers.len(), 4, "The number of signers should be 4");




    Ok(())
}

// ///Remove an admin
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn remove_community_wallet_admin()  -> Result<(), anyhow::Error> {
    let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;
    let client = s.client();

    let signers = get_signers(&s);

    //create new wallet
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins:signers,
            migrate_n: None
        }))),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
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

    // Get signer 1 private key
    let first_signer = get_private_key_at_n(&s, 2);

    //Get admin to drop signer 3
    let signer_three_address = s.swarm
                                .validators()
                                .nth(4)
                                .expect("Validator at specified index not found")
                                .peer_id();

    
    // Propose drop admin
    let cli_add_new_admin_proposal = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
            community_wallet: comm_wallet_addr,
            admin: signer_three_address,
            drop: Some(false),
            n: 2,
            epochs: Some(10), 
        }))),
        mnemonic: None,
        test_private_key: Some(first_signer?), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_add_new_admin_proposal.run()
    .await
    .expect("CLI could not propose new admin to community wallet");


    // Verify the admins remain unchanged
    let comm_wallet_signers = query_view::get_view(&s.client(), "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert_eq!(comm_wallet_signers.len(), 3, "The number of signers should be 3");


    // Get signer 2 private key
    let second_signer = get_private_key_at_n(&s, 2);

    // Singer 2 verify drop admin 3
    let cli_add_new_admin_proposal = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
            community_wallet: comm_wallet_addr,
            admin: new_admin_address,
            drop: Some(false),
            n: 2,
            epochs: Some(10), 
        }))),
        mnemonic: None,
        test_private_key: Some(second_signer?), 
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_add_new_admin_proposal.run()
    .await
    .expect("CLI could not add new admin to community wallet");

    // Verify the admins have dropped
    let comm_wallet_signers = query_view::get_view(&s.client(), "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert_eq!(comm_wallet_signers.len(), 2, "The number of signers should be 2");




    Ok(())
}

/// TODO: Create test
// ///Veto a payment
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn cancel_community_wallet_payment() {

// }

/// TODO: Create test
/// Liquidate a community wallet
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn liquidate_community_wallet() {
// }


/// UTILITY ///

async fn setup_environment() -> (LibraSmoke, TempPath,AccountAddress, AccountAddress) {
    let dir = diem_temppath::TempPath::new();
    let mut s = LibraSmoke::new(Some(5))
        .await
        .expect("Could not start libra smoke");

    configure_validator::init_val_config_files(&mut s.swarm, 0, dir.path().to_owned())
        .await
        .expect("Could not initialize validator config");

    let account_address = "0x029633a96b0c0e81cc26cf2baefdbd479dab7161fbd066ca3be850012342cdee";

    let account_address_wrapped = AccountAddress::from_hex_literal(account_address)
        .expect("Failed to parse account address");

    // Transfer funds to ensure the account exists on-chain
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account: account_address_wrapped,
            amount: 100.0,
        }),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(dir.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
    };

    cli_transfer.run()
        .await
        .expect("CLI could not transfer funds to the new account");

    // get the address of the first node, the private key that was used to create the comm wallet
    let first_node = s.swarm
    .validators()
    .next()
    .expect("no first validator")
    .to_owned();
    let comm_wallet_addr = first_node.peer_id();

    (s, dir, account_address_wrapped,comm_wallet_addr)
}

fn get_signers(smoke: &LibraSmoke) -> Vec<AccountAddress> {
    smoke.swarm.validators()
        .skip(1) // Skip the first validator as it creates the community account from the second
        .take(3) // Take the next 3 validators
        .map(|validator| validator.peer_id()) // Extract `peer_id` from each validator
        .collect() // Collect into a Vec<AccountAddress>
}

fn get_private_key_at_n(smoke: &LibraSmoke, n: usize) -> Result<String, anyhow::Error> {
    let node = smoke.swarm
    .validators()
    .nth(n)
    .to_owned();
    let pri_key = node.expect("Node not found").account_private_key().clone().expect("Private key not found");

    let encoded_pri_key = pri_key
        .private_key()
        .to_encoded_string()
        .expect("cannot decode pri key");
    
    Ok( encoded_pri_key)
}
