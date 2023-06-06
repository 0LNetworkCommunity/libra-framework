use libra_txs::extension::client_ext::ClientExt as ClientExtDupl;
use libra_txs::extension::ed25519_private_key_ext::Ed25519PrivateKeyExt;
use zapatos_sdk::transaction_builder::TransactionBuilder;
use zapatos_sdk::types::{LocalAccount, AccountKey};
use zapatos_sdk::rest_client::Client;
use zapatos_sdk::types::account_address::AccountAddress;
use zapatos_sdk::types::chain_id::ChainId;
use libra_cached_packages::aptos_framework_sdk_builder::EntryFunctionCall::{ OlAccountTransfer};
use libra_config::extension::client_ext::{ClientExt, DEFAULT_TIMEOUT_SECS};
use std::time::{SystemTime, UNIX_EPOCH};

use super::submit_transaction::submit;

pub async fn run(to: AccountAddress, amount: u64) -> anyhow::Result<()> {
    let client = Client::default()?;

    let legacy = libra_wallet::legacy::get_keys_from_prompt()?;

    let owner = legacy.child_0_owner;
    let new_key = AccountKey::from_private_key(owner.pri_key);

    let seq = client.get_sequence_number(owner.account).await?;
    let mut local_acct = LocalAccount::new(owner.account, new_key, seq);


    let payload = OlAccountTransfer {
      to,
      amount,
    }.encode();

    // let payload = DemoPrintThis{}.encode();


    let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let time = t + DEFAULT_TIMEOUT_SECS*10;
    let tb = TransactionBuilder::new(payload,time, ChainId::test());

    let signed = local_acct.sign_with_transaction_builder(tb);

    submit(&signed)
}
