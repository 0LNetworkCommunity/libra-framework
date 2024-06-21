use std::path::PathBuf;

use diem_crypto::ValidCryptoMaterialStringExt;
use diem_types::account_address::AccountAddress;
use diem_temppath::TempPath;
use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::txs_cli::{TxsCli, TxsSub, TxsSub::Transfer};
use libra_txs::txs_cli_community::{
    CommunityTxs, InitTx, ClaimTx, CageTx
};
use libra_types::legacy_types::app_cfg::TxCost;
use url::Url;

// Create a V7 community wallet
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn create_community_wallet() -> Result<(), anyhow::Error> {
    let (mut s, dir, _account_address, comm_wallet_addr) = setup_environment().await;
    let config_path = dir.path().to_owned().join("libra-cli-config.yaml");

    // SETUP ADMIN SIGNERS
    // 1. Generate and fund 5 new accounts from validators to ensure their on-chain presence for signing operations.
    // 2. Transfer funds to the newly created signer accounts to enable their transactional capabilities.

    // SETUP COMMUNITY WALLET
    // 3. Prepare a new admin account but do not immediately use it within the community wallet.
    // 4. Create a community wallet offering the first three of the newly funded accounts as its admins.
    // 5. Admins claim the offer.
    // 6. Donor finalize and cage the community wallet to ensure its independence and security.

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
       run_cli_transfer(to_account, 10.0, validator_private_key.clone(), s.api_endpoint.clone(), config_path.clone()).await;
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
    run_cli_transfer(new_admin_address, 1.0, private_key_of_fifth_signer.clone(), s.api_endpoint.clone(), config_path.clone()).await;

    // Get 3 signers to be admins
    let first_three_signer_addresses: Vec<AccountAddress> = signer_addresses
        .clone()
        .into_iter()
        .take(3)
        .collect();

    // Create new community wallet and offer it to the first three signers
    let donor_private_key = s.encoded_pri_key.clone();
    run_cli_community_init(donor_private_key.clone(), first_three_signer_addresses.clone(), 3, s.api_endpoint.clone(), config_path.clone()).await;

    // Verify if the account is not a community wallet yet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert!(!is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should not be a community wallet yet");

    // Check offer proposed
    let proposed_query_res = query_view::get_view(&s.client(), "0x1::multi_action::get_offer_proposed", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet proposed offer");

    // Assert authorities are the three proposed
    let authorities = proposed_query_res.as_array().unwrap()[0].as_array().unwrap();
    assert_eq!(authorities.len(), 3, "There should be 3 authorities");
    for i in 0..3 {
        assert_eq!(authorities[i].as_str().unwrap().trim_start_matches("0x"), first_three_signer_addresses[i].to_string(), "Authority should be the same");
    }

    // Admins claim the offer
    for j in 0..3 {
        let auth = &signers[j];
        // print private key
        let authority_pk = auth.private_key().to_encoded_string().expect("cannot decode pri key");
        run_cli_claim_offer(authority_pk, comm_wallet_addr.clone(), s.api_endpoint.clone(), config_path.clone()).await;
    }

    // Check offer claimed
    let proposed_query_res = query_view::get_view(&s.client(), "0x1::multi_action::get_offer_claimed", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet offer claimed");

    // Assert authorities are the three proposed
    let authorities = proposed_query_res.as_array().unwrap()[0].as_array().unwrap();
    assert_eq!(authorities.len(), 3, "There should be 3 authorities");
    for i in 0..3 {
        assert_eq!(authorities[i].as_str().unwrap().trim_start_matches("0x"), first_three_signer_addresses[i].to_string(), "Authority should be the same");
    }

    // Donor finalize and cage the community wallet
    run_cli_community_cage(donor_private_key.clone(), 3, s.api_endpoint.clone(), config_path.clone()).await;

    // Ensure the account is now a community wallet
    let is_comm_wallet_query_res = query_view::get_view(&s.client(), "0x1::community_wallet::is_init", None, Some(comm_wallet_addr.clone().to_string()))
        .await
        .expect("Query failed: community wallet init check");

    assert!(is_comm_wallet_query_res.as_array().unwrap()[0].as_bool().unwrap(), "Account should be a community wallet");

    Ok(())
}

// UTILITY //

async fn run_cli_transfer(
    to_account: AccountAddress,
    amount: f64,
    private_key: String,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    // Build the CLI command
    let cli_transfer = TxsCli {
        subcommand: Some(Transfer {
            to_account,
            amount,
        }),
        mnemonic: None,
        test_private_key: Some(private_key),
        chain_id: None,
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
        .expect(&format!("CLI could not transfer funds to account {}", to_account.to_string()));
}

async fn run_cli_community_init(
    donor_private_key: String,
    auhtorities: Vec<AccountAddress>,
    num_signers: u64,
    api_endpoint: Url,
    config_path: PathBuf,
) {
    // Build the CLI command
    let cli_set_community_wallet = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovInit(InitTx {
            admins: auhtorities,
            num_signers: num_signers,
        }))),
        mnemonic: None,
        test_private_key: Some(donor_private_key),
        chain_id: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    // Execute the transaction
    cli_set_community_wallet.run()
        .await
        .expect("CLI could not create community wallet");
}

async fn run_cli_claim_offer(
    signer_pk: String,
    community_address: AccountAddress,
    api_endpoint: Url,
    config_path: PathBuf
) {
    let cli_claim_offer = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovClaim(ClaimTx {
            community_wallet: community_address,
        }))),
        mnemonic: None,
        test_private_key: Some(signer_pk),
        chain_id: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_claim_offer.run()
        .await
        .expect("CLI could not claim offer");
}

async fn run_cli_community_cage(
    donor_private_key: String,
    num_signers: u64,
    api_endpoint: Url,
    config_path: PathBuf
) {
    let cli_finalize_cage = TxsCli {
        subcommand: Some(TxsSub::Community(CommunityTxs::GovCage(CageTx {
            num_signers: num_signers,
        }))),
        mnemonic: None,
        test_private_key: Some(donor_private_key),
        chain_id: None,
        config_path: Some(config_path),
        url: Some(api_endpoint),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli_finalize_cage.run()
        .await
        .expect("CLI could not finalize and cage community wallet");
}

async fn setup_environment() -> (LibraSmoke, TempPath, AccountAddress, AccountAddress) {
    let dir = diem_temppath::TempPath::new();
    let mut s = LibraSmoke::new(Some(5), None)
        .await
        .expect("Could not start libra smoke");

    configure_validator::init_val_config_files(&mut s.swarm, 0, dir.path().to_owned())
        .await
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
        chain_id: None,
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

    (s, dir, account_address_wrapped, comm_wallet_addr)
}
