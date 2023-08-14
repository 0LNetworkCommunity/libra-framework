use anyhow::bail;
use libra_cached_packages::aptos_stdlib;
use zapatos_forge::AptosPublicInfo;
use zapatos_sdk::rest_client::{aptos::Balance, Client, Response};
use zapatos_types::account_address::AccountAddress;

/// Get the balance of the 0L coin. Client methods are hardcoded for vendor
pub async fn get_libra_balance(
    client: &Client,
    address: AccountAddress,
) -> anyhow::Result<Response<Balance>> {
    let resp = client
        .get_account_resource(address, "0x1::coin::CoinStore<0x1::gas_coin::GasCoin>")
        .await?;
    resp.and_then(|resource| {
        if let Some(res) = resource {
            let b = serde_json::from_value::<Balance>(res.data)?;
            Ok(b)
        } else {
            bail!("No data returned")
        }
    })
    // bail!("No data returned");
}

pub async fn mint_libra(
    public_info: &mut AptosPublicInfo<'_>,
    addr: AccountAddress,
    amount: u64,
) -> anyhow::Result<()> {
    let payload = public_info
        .transaction_factory()
        .payload(aptos_stdlib::gas_coin_mint_to_impl(addr, amount));

    let mint_txn = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info.client().submit_and_wait(&mint_txn).await?;
    Ok(())
}
