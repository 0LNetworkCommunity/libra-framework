use anyhow::{Context, Result};
use libra_config::extension::client_ext::{ClientExt, DEFAULT_TIMEOUT_SECS};
use txs::{
    coin_client::{CoinClient, TransferOptions},
    constant::{DEFAULT_GAS_UNIT_PRICE, DEFAULT_MAX_GAS_AMOUNT},
    crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt},
    extension::ed25519_private_key_ext::Ed25519PrivateKeyExt,
    rest_client::Client,
    types::account_address::AccountAddress,
};

pub async fn run(
    to_account: &str,
    amount: u64,
    private_key: &str,
    max_gas: Option<u64>,
    gas_unit_price: Option<u64>,
) -> Result<()> {
    let client = Client::default()?;
    let coint_client = CoinClient::new(&client);
    let private_key = Ed25519PrivateKey::from_encoded_string(private_key)?;
    let mut from_account = private_key.get_account(None).await?;
    let to_account = AccountAddress::from_hex_literal(to_account).context(format!(
        "Failed to parse the recipient address {to_account}"
    ))?;
    let transfer_options = TransferOptions {
        max_gas_amount: max_gas.unwrap_or(DEFAULT_MAX_GAS_AMOUNT),
        gas_unit_price: gas_unit_price.unwrap_or(DEFAULT_GAS_UNIT_PRICE),
        timeout_secs: DEFAULT_TIMEOUT_SECS,
        coin_type: "0x1::aptos_coin::AptosCoin",
    };

    client
        .wait_for_transaction(
            &coint_client
                .transfer(
                    &mut from_account,
                    to_account,
                    amount,
                    Some(transfer_options),
                )
                .await?,
        )
        .await?;

    println!("Success!");
    Ok(())
}
