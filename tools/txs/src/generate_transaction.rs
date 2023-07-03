use super::constants::{DEFAULT_GAS_UNIT_PRICE, DEFAULT_MAX_GAS_AMOUNT};
use anyhow::Result;
use zapatos_sdk::crypto::ed25519::Ed25519PrivateKey;
use zapatos_sdk::rest_client::Client;
use zapatos_types::transaction::SignedTransaction;

use libra_types::type_extensions::{
  client_ext::{ClientExt,TransactionOptions, DEFAULT_TIMEOUT_SECS},
  ed25519_private_key_ext::Ed25519PrivateKeyExt,
};

pub async fn _run(
    function_id: &str,
    private_key: &Ed25519PrivateKey,
    type_args: Option<String>,
    args: Option<String>,
    max_gas: Option<u64>,
    gas_unit_price: Option<u64>,
) -> Result<SignedTransaction> {
    let client = Client::default().await?;
    let mut account = private_key.get_account(None).await?;
    let options = TransactionOptions {
        max_gas_amount: max_gas.unwrap_or(DEFAULT_MAX_GAS_AMOUNT),
        gas_unit_price: gas_unit_price.unwrap_or(DEFAULT_GAS_UNIT_PRICE),
        timeout_secs: DEFAULT_TIMEOUT_SECS,
    };

    client
        .generate_transaction(&mut account, function_id, type_args, args, options)
        .await
}
