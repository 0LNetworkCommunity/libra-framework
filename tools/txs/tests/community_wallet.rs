// use diem_crypto::ValidCryptoMaterialStringExt;
// use diem_temppath::TempPath;
// use diem_types::account_address::AccountAddress;
// use libra_query::query_view;
// use libra_smoke_tests::{configure_validator, helpers::get_libra_balance, libra_smoke::LibraSmoke};
// use libra_txs::txs_cli::{TxsCli, TxsSub, TxsSub::Transfer};
// use libra_txs::txs_cli_community::{
//     AdminTx, CommunityTxs, FinalizeCageTx, InitTx, ProposeTx, VetoTx,
// };
// use libra_types::legacy_types::app_cfg::TxCost;

// /// TODO: Test the migration of an existing community wallet with n flag
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn migrate_community_wallet_with_flag() {
//     let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;

//     // Set up migrated v6 community wallet
//     //TODO: Use mocked migrated state v6 community wallet

//     let signers = get_signers(&s);

//     //create wallet with -n flag option
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:signers,
//             migrate_n: Some(2)
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()), //TODO: Use mocked migrated state v6 community wallet
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create a new Community Wallet with -n flag set");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     Ok(())

// }

// /// TODO: Test the migration of an existing community wallet
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn migrate_community_wallet() {
//     let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;

//     // Set up migrated v6 community wallet
//     //TODO: Use mocked migrated state v6 community wallet

//     let signers = get_signers(&s);

//     //create wallet with -n flag option
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:signers,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()), //TODO: Use mocked migrated state v6 community wallet
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create a new Community Wallet with -n flag set");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     Ok(())
// }

// // TODO: Not passing likely due to donor_voice not adding the community wallet tag
// // Create a V7 community wallet with -n flag
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn create_community_wallet_with_flag() -> Result<(), anyhow::Error> {
//     let (mut s, dir, account_address, comm_wallet_addr) = setup_environment().await;

//     let signers = get_signers(&s);

//     //create wallet with -n flag option
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:signers,
//             migrate_n: Some(2)
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()), //Issue using the private key private_key.to_string()
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create a new Community Wallet with -n flag set");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     Ok(())
// }

// // TODO: Need to find way to capture the panic
// // Attempt to transfer from a community wallet account and bypass multisig
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn new_community_wallet_cant_transfer() -> Result<(), anyhow::Error> {
//     let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;

//     // SETUP ADMIN SIGNERS
//     // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
//     // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

//     // SETUP COMMUNITY WALLET
//     // 3. Prepare a new admin account but do not immediately use it within the community wallet.
//     // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
//     // 5. Confirm the successful creation of the community wallet and its recognition by the system.
//     // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

//     // SETUP ADMIN SIGNERS //
//     // We set up 5 new accounts and also fund them from each of the 5 validators

//     let (signers, signer_addresses) = s.create_accounts(5).await?;

//     // Ensure there's a one-to-one correspondence between signers and private keys
//     if signer_addresses.len() != s.validator_private_keys.len() {
//         panic!("The number of signer addresses does not match the number of validator private keys.");
//     }

//     for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
//         let to_account = signer_address.clone();

//         // Transfer funds to ensure the account exists on-chain using the specific validator's private key
//         let cli_transfer = TxsCli {
//             subcommand: Some(Transfer {
//                 to_account,
//                 amount: 10.0,
//             }),
//             mnemonic: None,
//             test_private_key: Some(validator_private_key.clone()),
//             chain_id: None,
//             config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//             url: Some(s.api_endpoint.clone()),
//             tx_profile: None,
//             tx_cost: Some(TxCost::default_baseline_cost()),
//             estimate_only: false,
//         };

//         // Execute the transfer
//         cli_transfer.run()
//             .await
//             .expect(&format!("CLI could not transfer funds to account {}", signer_address));

//     }

//     // SETUP COMMUNITY WALLET //

//     // Prepare new admin account
//     let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

//     let new_admin_address = AccountAddress::from_hex_literal(new_admin)
//     .expect("Failed to parse account address");

//     // Fund with the last signer to avoid ancestry issues
//     let private_key_of_fifth_signer = signers[4]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Transfer funds to ensure the account exists on-chain
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_admin_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_fifth_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_transfer.run()
//     .await
//     .expect("CLI could not transfer funds to the new account");

//     // Get 3 signers to be admins
//     let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
//     .clone()
//     .into_iter()
//     .take(3)
//     .collect();

//     //create new community wallet
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:first_three_signer_addresses,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create community wallet");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     // Attempt to create a transfer from the new community wallet
//     let transfer_cli = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_admin_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     match transfer_cli.run().await {
//         Ok(_) => {
//             // Instead of panicking, you might want to assert that this scenario should not happen
//             assert!(false, "Transfer succeeded unexpectedly. Expected an error.");
//         },
//         Err(e) => {
//             // Check if the error is what you expected
//             if e.to_string().contains("MoveAbort") && e.to_string().contains("196618") && e.to_string().contains("ENOT_FOR_CW") {
//                 // If the error matches the expected condition, you can consider this a successful test case
//                 println!("Received expected error: {:?}", e);
//             } else {
//                 // If the error does not match, you should consider it an actual error case
//                 return Err(e);
//             }
//         },
//     }

//     Ok(())
// }

// // PASSING
// //Create a V7 community wallet
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn create_community_wallet() -> Result<(), anyhow::Error> {
//     let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;

//     // SETUP ADMIN SIGNERS
//     // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
//     // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

//     // SETUP COMMUNITY WALLET
//     // 3. Prepare a new admin account but do not immediately use it within the community wallet.
//     // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
//     // 5. Confirm the successful creation of the community wallet and its recognition by the system.
//     // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

//     // SETUP ADMIN SIGNERS //
//     // We set up 5 new accounts and also fund them from each of the 5 validators

//     let (signers, signer_addresses) = s.create_accounts(5).await?;

//     // Ensure there's a one-to-one correspondence between signers and private keys
//     if signer_addresses.len() != s.validator_private_keys.len() {
//         panic!("The number of signer addresses does not match the number of validator private keys.");
//     }

//     for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
//         let to_account = signer_address.clone();

//         // Transfer funds to ensure the account exists on-chain using the specific validator's private key
//         let cli_transfer = TxsCli {
//             subcommand: Some(Transfer {
//                 to_account,
//                 amount: 10.0,
//             }),
//             mnemonic: None,
//             test_private_key: Some(validator_private_key.clone()),
//             chain_id: None,
//             config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//             url: Some(s.api_endpoint.clone()),
//             tx_profile: None,
//             tx_cost: Some(TxCost::default_baseline_cost()),
//             estimate_only: false,
//         };

//         // Execute the transfer
//         cli_transfer.run()
//             .await
//             .expect(&format!("CLI could not transfer funds to account {}", signer_address));
//     }

//     // SETUP COMMUNITY WALLET //

//     // Prepare new admin account
//     let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

//     let new_admin_address = AccountAddress::from_hex_literal(new_admin)
//     .expect("Failed to parse account address");

//     // Fund with the last signer to avoid ancestry issues
//     let private_key_of_fifth_signer = signers[4]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Transfer funds to ensure the account exists on-chain
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_admin_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_fifth_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_transfer.run()
//     .await
//     .expect("CLI could not transfer funds to the new account");

//     // Get 3 signers to be admins
//     let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
//     .clone()
//     .into_iter()
//     .take(3)
//     .collect();

//     //create new community wallet
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:first_three_signer_addresses,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create community wallet");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     Ok(())
// }

// // TODO: apply once we have a method to progress epochs
// // Propose and sign payment
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn community_wallet_payment() -> Result<(), anyhow::Error> {
//     let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;

//     // SETUP ADMIN SIGNERS
//     // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
//     // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

//     // SETUP COMMUNITY WALLET
//     // 3. Prepare a new admin account but do not immediately use it within the community wallet.
//     // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
//     // 5. Confirm the successful creation of the community wallet and its recognition by the system.
//     // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

//     // EXECUTE PAYMENT PROPOSAL
//     // 7. Propose a payment from the community wallet to a new worker account using one of the signer accounts.
//     // Configure and execute payment proposal command.
//     // 8. Validate the successful execution of the payment proposal by checking transaction receipt or wallet balance.

//     // SETUP ADMIN SIGNERS //
//     // Generate and fund 5 new accounts from validators for transaction signing
//     let (signers, signer_addresses) = s.create_accounts(5).await?;

//     // Ensure each signer has a corresponding private key
//     if signer_addresses.len() != s.validator_private_keys.len() {
//         panic!("The number of signer addresses does not match the number of validator private keys.");
//     }

//     // Transfer funds to newly created accounts to ensure they exist on-chain
//     for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
//         let to_account = signer_address.clone(); // Target account for funds transfer

//         // Configure transfer command with validator's private key
//         let cli_transfer = TxsCli {
//             subcommand: Some(Transfer {
//                 to_account, // Destination account for this iteration
//                 amount: 10.0, // Specified transfer amount
//             }),
//             mnemonic: None,
//             test_private_key: Some(validator_private_key.clone()), // Validator's private key for authentication
//             chain_id: None,
//             config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//             url: Some(s.api_endpoint.clone()),
//             tx_profile: None,
//             tx_cost: Some(TxCost::default_baseline_cost()),
//             estimate_only: false,
//         };

//         // Execute funds transfer to the account
//         cli_transfer.run()
//             .await
//             .expect(&format!("CLI could not transfer funds to account {}", signer_address));
//     }

//     // SETUP COMMUNITY WALLET //

//     // Prepare a new admin account but do not use it immediately
//     let new_worker = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";
//     let new_worker_address = AccountAddress::from_hex_literal(new_worker)
//         .expect("Failed to parse account address");

//     // Transfer funds to new admin account to ensure its on-chain presence
//     let private_key_of_fifth_signer = signers[4]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Configuration for transferring funds to new admin account
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_worker_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_fifth_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     // Execute the transfer to the new admin account
//     cli_transfer.run()
//         .await
//         .expect("CLI could not transfer funds to the new account");

//     // Select first three signers as initial community wallet admins
//     let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
//         .clone()
//         .into_iter()
//         .take(3)
//         .collect();

//     // Command to initialize community wallet with selected admins
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:first_three_signer_addresses,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     // Execute community wallet creation
//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create community wallet");

//     // Verify the account is now a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     // Finalize community wallet setup by revoking original account's access
//     let cli_finalize_cage = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::FinalizeAndCage(FinalizeCageTx {}))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };
//     cli_finalize_cage.run()
//         .await
//         .expect("CLI could not finalize and cage the community wallet");

//     // EXECUTE PAYMENT PROPOSAL //

//     // Propose a payment from the community wallet to the new worker account
//     let private_key_of_first_signer = signers[1]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Configure payment proposal command
//     let cli_propose_payment = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
//             community_wallet: comm_wallet_addr,
//             recipient: new_worker_address,
//             amount: 10,
//             description: "Thanks Mate".to_string(),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_first_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,    };

//     // Execute the payment proposal
//     cli_propose_payment.run()
//         .await
//         .expect("CLI could not propose payment");

//     Ok(())
// }

// // TODO: apply once we have a method to progress epochs
// // Add an admin
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn add_community_wallet_admin() -> Result<(), anyhow::Error> {
//     let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;
//     let client = s.client();

//     // SETUP ADMIN SIGNERS
//     // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
//     // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

//     // SETUP COMMUNITY WALLET
//     // 3. Prepare a new admin account but do not immediately use it within the community wallet.
//     // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
//     // 5. Confirm the successful creation of the community wallet and its recognition by the system.
//     // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

//     // ADD NEW ADMIN
//     // 7. Initiate the process to add a new admin to the community wallet by proposing through an existing admin.
//     // 8. Validate the addition of the new admin by checking the updated count of admins/signers in the wallet.

//     // SETUP ADMIN SIGNERS //
//     // We set up 5 new accounts and also fund them from each of the 5 validators

//     let (signers, signer_addresses) = s.create_accounts(5).await?;

//     // Ensure there's a one-to-one correspondence between signers and private keys
//     if signer_addresses.len() != s.validator_private_keys.len() {
//         panic!("The number of signer addresses does not match the number of validator private keys.");
//     }

//     for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
//         let to_account = signer_address.clone(); // Adjust this line if necessary

//         // Transfer funds to ensure the account exists on-chain using the specific validator's private key
//         let cli_transfer = TxsCli {
//             subcommand: Some(Transfer {
//                 to_account, // Use the current signer address for this iteration
//                 amount: 10.0, // Adjust the amount as needed
//             }),
//             mnemonic: None,
//             test_private_key: Some(validator_private_key.clone()), // Use the corresponding validator's private key
//             chain_id: None,
//             config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//             url: Some(s.api_endpoint.clone()),
//             tx_profile: None,
//             tx_cost: Some(TxCost::default_baseline_cost()),
//             estimate_only: false,
//         };

//         // Execute the transfer
//         cli_transfer.run()
//             .await
//             .expect(&format!("CLI could not transfer funds to account {}", signer_address));
//     }

//     // SETUP COMMUNITY WALLET //

//     // Prepare new admin account
//     let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

//     let new_admin_address = AccountAddress::from_hex_literal(new_admin)
//     .expect("Failed to parse account address");

//     // Fund with the last signer to avoid ancestry issues
//     let private_key_of_fifth_signer = signers[4]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Transfer funds to ensure the account exists on-chain
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_admin_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_fifth_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_transfer.run()
//     .await
//     .expect("CLI could not transfer funds to the new account");

//     // Get 3 signers to be admins
//     let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
//     .clone()
//     .into_iter()
//     .take(3)
//     .collect();

//     //create new community wallet
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:first_three_signer_addresses,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create community wallet");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     // Remove the ability for the original account to access
//     let cli_finalize_cage = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::FinalizeAndCage(FinalizeCageTx {}))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_finalize_cage.run()
//         .await
//         .expect("CLI could not finalize and cage the community wallet");

//     // ADD NEW ADMIN //

//     // Create initial proposal
//     let private_key_of_first_signer = signers[1]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Verify the admins remain unchanged
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_before = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_before, 3, "The number of signers should be 3");

//     // Propose add admin
//     let cli_add_new_admin_proposal = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
//             community_wallet: comm_wallet_addr,
//             admin: new_admin_address,
//             drop: Some(true),
//             n: 2,
//             epochs: Some(10),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_first_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_add_new_admin_proposal.run()
//     .await
//     .expect("CLI could not add new admin to community wallet");

//     // Verify the admins remain unchanged
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_after_proposal = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_after_proposal, 3, "The number of signers should be 3");

//     // Get second signer private key
//     let private_key_of_second_signer = signers[2]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Singer 2 verify new admin
//     let cli_add_new_admin_proposal = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
//             community_wallet: comm_wallet_addr,
//             admin: new_admin_address,
//             drop: Some(true),
//             n: 2,
//             epochs: Some(10),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_second_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_add_new_admin_proposal.run()
//     .await
//     .expect("CLI could not add new admin to community wallet");

//     // Verify the admins have dropped
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_after_second_proposal = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_after_second_proposal, 4, "The number of signers should be 4");

//     Ok(())
// }

// // TODO: apply once we have a method to progress epochs
// // Remove an admin
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn remove_community_wallet_admin()  -> Result<(), anyhow::Error> {
//     let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;
//     let client = s.client();

//     // SETUP ADMIN SIGNERS
//     // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
//     // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

//     // SETUP COMMUNITY WALLET
//     // 3. Prepare a new admin account but do not immediately use it within the community wallet.
//     // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
//     // 5. Confirm the successful creation of the community wallet and its recognition by the system.
//     // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

//     // ADD NEW ADMIN
//     // 7. Initiate the process to add a new admin to the community wallet by proposing through an existing admin.
//     // 8. Validate the addition of the new admin by checking the updated count of admins/signers in the wallet.

//     // REMOVE NEW ADMIN
//     // 9. Start the removal process of the newly added admin through a proposal from an existing admin.
//     // 10. Complete the admin removal process and verify by checking the updated admins/signers count.

//     // SETUP ADMIN SIGNERS //
//     // We set up 5 new accounts and also fund them from each of the 5 validators

//     let (signers, signer_addresses) = s.create_accounts(5).await?;

//     // Ensure there's a one-to-one correspondence between signers and private keys
//     if signer_addresses.len() != s.validator_private_keys.len() {
//         panic!("The number of signer addresses does not match the number of validator private keys.");
//     }

//     for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
//         let to_account = signer_address.clone(); // Adjust this line if necessary

//         // Transfer funds to ensure the account exists on-chain using the specific validator's private key
//         let cli_transfer = TxsCli {
//             subcommand: Some(Transfer {
//                 to_account,
//                 amount: 10.0,
//             }),
//             mnemonic: None,
//             test_private_key: Some(validator_private_key.clone()),
//             chain_id: None,
//             config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//             url: Some(s.api_endpoint.clone()),
//             tx_profile: None,
//             tx_cost: Some(TxCost::default_baseline_cost()),
//             estimate_only: false,
//         };

//         // Execute the transfer
//         cli_transfer.run()
//             .await
//             .expect(&format!("CLI could not transfer funds to account {}", signer_address));
//     }

//     // SETUP COMMUNITY WALLET //

//     // Prepare new admin account
//     let new_admin = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

//     let new_admin_address = AccountAddress::from_hex_literal(new_admin)
//     .expect("Failed to parse account address");

//     // Fund with the last signer to avoid ancestry issues
//     let private_key_of_fifth_signer = signers[4]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Transfer funds to ensure the account exists on-chain
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_admin_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_fifth_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_transfer.run()
//     .await
//     .expect("CLI could not transfer funds to the new account");

//     // Get 3 signers to be admins
//     let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
//     .clone()
//     .into_iter()
//     .take(3)
//     .collect();

//     //create new community wallet
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:first_three_signer_addresses,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create community wallet");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     // Remove the ability for the original account to access
//     let cli_finalize_cage = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::FinalizeAndCage(FinalizeCageTx {}))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_finalize_cage.run()
//         .await
//         .expect("CLI could not finalize and cage the community wallet");

//     // ADD NEW ADMIN //

//     // Create initial proposal
//     let private_key_of_first_signer = signers[1]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Verify the admins remain unchanged
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_before = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_before, 4, "The number of signers should be 4");

//     // Propose add admin
//     let cli_add_new_admin_proposal = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
//             community_wallet: comm_wallet_addr,
//             admin: new_admin_address,
//             drop: Some(true),
//             n: 2,
//             epochs: Some(10),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_first_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_add_new_admin_proposal.run()
//     .await
//     .expect("CLI could not add new admin to community wallet");

//     // Verify the admins remain unchanged
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_after_proposal = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_after_proposal, 3, "The number of signers should be 3");

//     // Get second signer private key
//     let private_key_of_second_signer = signers[2]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Singer 2 verify new admin
//     let cli_add_new_admin_proposal = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
//             community_wallet: comm_wallet_addr,
//             admin: new_admin_address,
//             drop: Some(true),
//             n: 2,
//             epochs: Some(10),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_second_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_add_new_admin_proposal.run()
//     .await
//     .expect("CLI could not add new admin to community wallet");

//     // Verify the admins have dropped
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_after_second_proposal = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_after_second_proposal, 4, "The number of signers should be 4");

//     // REMOVE NEW ADMIN //

//     // Create initial proposal
//     let private_key_of_first_signer = signers[1]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Verify the admins remain unchanged
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_before = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_before, 3, "The number of signers should be 3");

//     // Propose add admin
//     let cli_add_new_admin_proposal = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
//             community_wallet: comm_wallet_addr,
//             admin: new_admin_address,
//             drop: Some(false),
//             n: 2,
//             epochs: Some(10),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_first_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_add_new_admin_proposal.run()
//     .await
//     .expect("CLI could not add new admin to community wallet");

//     // Verify the admins remain unchanged
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_after_proposal = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     //TODO: This should be 4 when we have progressing epochs
//     assert_eq!(no_of_signers_after_proposal, 3, "The number of signers should be 4");

//     // Get second signer private key
//     let private_key_of_second_signer = signers[2]
//         .private_key()
//         .to_encoded_string()
//         .expect("cannot decode pri key");

//     // Singer 2 verify new admin
//     let cli_add_new_admin_proposal = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovAdmin(AdminTx {
//             community_wallet: comm_wallet_addr,
//             admin: new_admin_address,
//             drop: Some(false),
//             n: 2,
//             epochs: Some(10),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_second_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_add_new_admin_proposal.run()
//     .await
//     .expect("CLI could not add new admin to community wallet");

//     // Verify the admins have dropped
//     let comm_wallet_signers = query_view::get_view(&client, "0x1::multi_action::get_authorities", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");
//     let no_of_signers_after_second_proposal = comm_wallet_signers
//     .as_array()
//     .and_then(|outer_array| outer_array.get(0))
//     .and_then(|inner_array_value| inner_array_value.as_array())
//     .map_or(0, |inner_array| inner_array.len());

//     assert_eq!(no_of_signers_after_second_proposal, 3, "The number of signers should be 3");

//     Ok(())
// }

// // TODO: apply once we have a method to progress epochs
// // Veto a payment
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn cancel_community_wallet_payment() -> Result<(), anyhow::Error> {
//     let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;
//     let client = s.client();

//     // SETUP ADMIN SIGNERS
//     // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
//     // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

//     // SETUP COMMUNITY WALLET
//     // 3. Prepare a new admin account but do not immediately use it within the community wallet.
//     // 4. Create a community wallet specifying the first three of the newly funded accounts as its admins.
//     // 5. Confirm the successful creation of the community wallet and its recognition by the system.
//     // 6. Revoke the original creator account's access to ensure security and independence of the community wallet.

//     // ADD NEW ADMIN
//     // 7. Initiate the process to add a new admin to the community wallet by proposing through an existing admin.
//     // 8. Validate the addition of the new admin by checking the updated count of admins/signers in the wallet.

//     // PROPOSAL AND FUNDING OPERATIONS
//     // 6. Propose payments to the new worker account from the community wallet using two of the signers.
//     // 7. Assert the balance of the new worker account to ensure payments are not yet processed.
//     // 8. Check and assert the balance of the community wallet before depositing additional funds.
//     // 9. Fund the community wallet with an additional amount to assert governance.

//     // VETOING A PAYMENT
//     // 10. Veto the previously proposed payment, effectively canceling it.
//     // 11. (Optional steps for further validation and checks, e.g., verifying veto state, could be added here.)

//     // SETUP ADMIN SIGNERS //
//     // We set up 5 new accounts and also fund them from each of the 5 validators

//     let (signers, signer_addresses) = s.create_accounts(5).await?;

//     // Ensure there's a one-to-one correspondence between signers and private keys
//     if signer_addresses.len() != s.validator_private_keys.len() {
//         panic!("The number of signer addresses does not match the number of validator private keys.");
//     }

//     for (signer_address, validator_private_key) in signer_addresses.iter().zip(s.validator_private_keys.iter()) {
//         let to_account = signer_address.clone(); // Adjust this line if necessary

//         // Transfer funds to ensure the account exists on-chain using the specific validator's private key
//         let cli_transfer = TxsCli {
//             subcommand: Some(Transfer {
//                 to_account,
//                 amount: 10.0, // Adjust the amount as needed
//             }),
//             mnemonic: None,
//             test_private_key: Some(validator_private_key.clone()), // Use the corresponding validator's private key
//             chain_id: None,
//             config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//             url: Some(s.api_endpoint.clone()),
//             tx_profile: None,
//             tx_cost: Some(TxCost::default_baseline_cost()),
//             estimate_only: false,
//         };

//         // Execute the transfer
//         cli_transfer.run()
//             .await
//             .expect(&format!("CLI could not transfer funds to account {}", signer_address));
//     }

//     // SETUP COMMUNITY WALLET //

//     // Prepare new admin account
//     let new_worker = "0xDCD1AFDFB32A8EB0AADF169ECE2D9BA1552E96FA7D683934F280AC28F29D3611";

//     let new_worker_address = AccountAddress::from_hex_literal(new_worker)
//     .expect("Failed to parse account address");

//     // Fund with the last signer to avoid ancestry issues
//     let private_key_of_fifth_signer = signers[4]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Transfer funds to ensure the account exists on-chain
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: new_worker_address,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_fifth_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_transfer.run()
//     .await
//     .expect("CLI could not transfer funds to the new account");

//     // Get 3 signers to be admins
//     let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
//     .clone()
//     .into_iter()
//     .take(3)
//     .collect();

//     //create new community wallet
//     let cli_set_community_wallet = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
//             admins:first_three_signer_addresses,
//             migrate_n: None
//         }))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_set_community_wallet.run()
//         .await
//         .expect("CLI could not create community wallet");

//     // Verify if the account is a community wallet
//     let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
//         .await
//         .expect("Query failed: community wallet init check");

//     assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

//     // Remove the ability for the original account to access
//     let cli_finalize_cage = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::FinalizeAndCage(FinalizeCageTx {}))),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_finalize_cage.run()
//         .await
//         .expect("CLI could not finalize and cage the community wallet");

//     let private_key_of_first_signer = signers[1]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Propose payment signer 1
//     let cli_propose_payment = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
//             community_wallet: comm_wallet_addr,
//             recipient: new_worker_address,
//             amount: 10,
//             description: "Thanks Mate".to_string(),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_first_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_propose_payment.run()
//     .await
//     .expect("CLI could not propose payment from community wallet");

//     let bal = get_libra_balance(&client, new_worker_address).await?;
//     assert_eq!(bal.total, 1000000, "Balance of the account(1000000) should not be processed yet");

//     let private_key_of_second_signer = signers[2]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     // Propose payment signer 2
//     let cli_propose_payment_signer_two = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::Propose(ProposeTx {
//             community_wallet: comm_wallet_addr,
//             recipient: new_worker_address,
//             amount: 10,
//             description: "Thanks Mate".to_string(),
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_second_signer),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_propose_payment_signer_two.run()
//     .await
//     .expect("CLI could not propose payment from community wallet");

//     let bal = get_libra_balance(&client, new_worker_address).await?;
//     assert_eq!(bal.total, 1000000, "Balance of the account(1000000) should not be processed yet");

//     //TODO: How to get the uid to query a community wallet
//     // Check multisig proposal state
//     // let is_payment_scheduled_query_res = query_view::get_view(&s.client(), "0x1::donor_voice_tx::get_multisig_proposal_state", Some(comm_wallet_addr.to_string()), Some("0".to_string()))
//     // .await
//     // .expect("Query failed: Can not get proposal state");
//     // println!{"{:#?}", is_payment_scheduled_query_res};

//     //assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account state should be in scheduled");

//     // verify balance of community wallet prior to depositing
//     let bal = get_libra_balance(&client, comm_wallet_addr).await?;
//     assert_eq!(bal.total, 889049874, "Balance of the account(889049874) does not match");

//     // Fund with the community wallet with the last signer to assert governance on the community wallet
//     let private_key_of_forth_signer = signers[3]
//     .private_key()
//     .to_encoded_string()
//     .expect("cannot decode pri key");

//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: comm_wallet_addr,
//             amount: 1.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_forth_signer.clone()), // Use the corresponding validator's private key
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     // Execute the transfer
//     cli_transfer.run()
//         .await
//         .expect(&format!("CLI could not transfer funds to account {}", comm_wallet_addr));

//     let bal = get_libra_balance(&client, comm_wallet_addr).await?;
//     assert_eq!(bal.total, 890049874, "Balance of the account(890049874) after transfer does not match");

//     // Veto the payment to the community wallet
//     let cli_veto_tx = TxsCli {
//         subcommand: Some(TxsSub::Community(CommunityTxs::Veto(VetoTx {
//             community_wallet: comm_wallet_addr,
//             proposal_id: 0,
//         }))),
//         mnemonic: None,
//         test_private_key: Some(private_key_of_forth_signer.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     // Execute the VetoTx command
//     cli_veto_tx.run()
//     .await
//     .expect("CLI could not execute veto transaction");

//     //TODO: Check veto state
//     // Check multisig proposal state
//     // let vote_state_query_res = query_view::get_view(&s.client(), "donor_voice_governance::get_veto_tally", Some(comm_wallet_addr.to_string()), Some("0".to_string()))
//     // .await
//     // .expect("Query failed: Can not get veto state");
//     // println!{"{:#?}", vote_state_query_res};

//     //assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account state should be in scheduled");

//     Ok(())
// }

// // TODO: Create test
// // Liquidate a community wallet
// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn liquidate_community_wallet() {
// }

// UTILITY //

// async fn setup_environment() -> (LibraSmoke, TempPath, AccountAddress, AccountAddress) {
//     let dir = diem_temppath::TempPath::new();
//     let mut s = LibraSmoke::new(Some(5))
//         .await
//         .expect("Could not start libra smoke");

//     configure_validator::init_val_config_files(&mut s.swarm, 0, dir.path().to_owned())
//         .await
//         .expect("Could not initialize validator config");

//     let account_address = "0x029633a96b0c0e81cc26cf2baefdbd479dab7161fbd066ca3be850012342cdee";

//     let account_address_wrapped =
//         AccountAddress::from_hex_literal(account_address).expect("Failed to parse account address");

//     // Transfer funds to ensure the account exists on-chain
//     let cli_transfer = TxsCli {
//         subcommand: Some(Transfer {
//             to_account: account_address_wrapped,
//             amount: 100.0,
//         }),
//         mnemonic: None,
//         test_private_key: Some(s.encoded_pri_key.clone()),
//         chain_id: None,
//         config_path: Some(dir.path().to_owned().join("libra-cli-config.yaml")),
//         url: Some(s.api_endpoint.clone()),
//         tx_profile: None,
//         tx_cost: Some(TxCost::default_baseline_cost()),
//         estimate_only: false,
//     };

//     cli_transfer
//         .run()
//         .await
//         .expect("CLI could not transfer funds to the new account");

//     // get the address of the first node, the private key that was used to create the comm wallet
//     let first_node = s
//         .swarm
//         .validators()
//         .next()
//         .expect("no first validator")
//         .to_owned();
//     let comm_wallet_addr = first_node.peer_id();

//     (s, dir, account_address_wrapped, comm_wallet_addr)
// }
