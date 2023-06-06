use zapatos_sdk::transaction_builder::TransactionBuilder;
use zapatos_sdk::types::{LocalAccount, AccountKey};
use zapatos_sdk::rest_client::Client;
use zapatos_sdk::types::account_address::AccountAddress;
use zapatos_sdk::types::chain_id::ChainId;
use libra_cached_packages::aptos_framework_sdk_builder::EntryFunctionCall::{AptosAccountSetAllowDirectCoinTransfers, AptosAccountTransfer};
use libra_config::extension::client_ext::{ClientExt, DEFAULT_TIMEOUT_SECS};

use std::time::{SystemTime, UNIX_EPOCH};

pub async fn run() -> anyhow::Result<()> {
    let client = Client::default()?;

    let legacy = libra_wallet::legacy::get_keys_from_prompt()?;

    let owner = legacy.child_0_owner;
    let new_key = AccountKey::from_private_key(owner.pri_key);

    let mut local_acct = LocalAccount::new(owner.account, new_key, 0);

    let payload = AptosAccountSetAllowDirectCoinTransfers {
        allow: true,
    }.encode();

    // let payload = AptosAccountTransfer {
    //   to: AccountAddress::from_hex_literal("0x88C0B80B70314FEC169492561C9B20B5")?,
    //   amount: 33
    // }.encode();


    let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let time = t + DEFAULT_TIMEOUT_SECS;
    let tb = TransactionBuilder::new(payload,time, ChainId::test());

    let signed = local_acct.sign_with_transaction_builder(tb);

    let res = client.submit_and_wait(&signed).await?;


    dbg!(&res);


    // client.submit_and_wait()?;
    // // Have Alice send Bob some coins.
    // let txn_hash = client
    //     .transfer(&mut alice, to_account, amount, None)
    //     .await
    //     .context("Failed to submit transaction to transfer coins")?;
    // client.wait_for_transaction(&txn_hash).await?;

    // // Print intermediate balances.
    // println!("\n=== Intermediate Balances ===");
    // println!(
    //     "Alice: {:?}",
    //     coin_client
    //         .get_account_balance(&alice.address())
    //         .await
    //         .context("Failed to get Alice's account balance the second time")?
    // );
    // println!(
    //     "Bob: {:?}",
    //     coin_client
    //         .get_account_balance(&bob.address())
    //         .await
    //         .context("Failed to get Bob's account balance the second time")?
    // );

    // // Have Alice send Bob some more coins.
    // let txn_hash = coin_client
    //     .transfer(&mut alice, bob.address(), 1_000, None)
    //     .await
    //     .context("Failed to submit transaction to transfer coins")?;

    // client.wait_for_transaction(&txn_hash).await?;

    // // Print final balances.
    // println!("\n=== Final Balances ===");
    // println!(
    //     "Alice: {:?}",
    //     coin_client
    //         .get_account_balance(&alice.address())
    //         .await
    //         .context("Failed to get Alice's account balance the second time")?
    // );
    // println!(
    //     "Bob: {:?}",
    //     coin_client
    //         .get_account_balance(&bob.address())
    //         .await
    //         .context("Failed to get Bob's account balance the second time")?
    // );

    Ok(())
}
