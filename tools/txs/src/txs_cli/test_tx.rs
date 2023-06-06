// use anyhow::{Context, Result};
// use libra_config::extension::{client_ext::ClientExt, faucet_client_ext::FaucetClientExt};
// use libra_txs::{
//     coin_client::CoinClient,
//     crypto::ed25519::Ed25519PrivateKey,
//     rest_client::{Client, FaucetClient},
//     types::{AccountKey, LocalAccount},
// };

// use libra

use zapatos_sdk::transaction_builder::TransactionBuilder;
use zapatos_sdk::types::{LocalAccount, AccountKey};
use zapatos_sdk::{rest_client::Client, 
  // transaction_builder::TransactionFactory
};
use zapatos_sdk::types::chain_id::ChainId;
use libra_cached_packages::aptos_framework_sdk_builder::EntryFunctionCall::AptosAccountSetAllowDirectCoinTransfers;
use libra_config::extension::client_ext::ClientExt;

pub async fn run() -> anyhow::Result<()> {
    let client = Client::default()?;

    let legacy = libra_wallet::legacy::get_keys_from_prompt()?;

    let owner = legacy.child_0_owner;
    let new_key = AccountKey::from_private_key(owner.pri_key);

    let mut local_acct = LocalAccount::new(owner.account, new_key, 0);

    let payload = AptosAccountSetAllowDirectCoinTransfers {
        allow: true,
    }.encode();

    // let account;
    // let factory = TransactionFactory::new(ChainId::test());
    
    let tb = TransactionBuilder::new(payload,10000, ChainId::test());

    let signed = local_acct.sign_with_transaction_builder(tb);

    // let signed = factory
    // .payload(payload)
    // .build()
    // .sign(private_key, public_key)?
    // .into_inner();

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
