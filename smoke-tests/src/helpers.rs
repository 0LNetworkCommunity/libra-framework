use diem_forge::DiemPublicInfo;
use diem_sdk::rest_client::Client;
use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;
use libra_types::type_extensions::client_ext::ClientExt;

// /// Get the balance of the 0L coin. Client methods are hardcoded for vendor
// pub async fn get_libra_balance(
//     client: &Client,
//     address: AccountAddress,
// ) -> anyhow::Result<Response<Balance>> {
//     let resp = client
//         .get_account_resource(address, "0x1::coin::CoinStore<0x1::gas_coin::GasCoin>")
//         .await?;
//     resp.and_then(|resource| {
//         if let Some(res) = resource {
//             let b = serde_json::from_value::<Balance>(res.data)?;
//             Ok(b)
//         } else {
//             bail!("No data returned")
//         }
//     })
//     // bail!("No data returned");
// }

pub async fn get_libra_balance(
    client: &Client,
    address: AccountAddress,
) -> anyhow::Result<Vec<u64>> {
    let res = client
        .view_ext("0x1::slow_wallet::balance", None, Some(address.to_string()))
        .await?;
    let mut move_tuple = serde_json::from_value::<Vec<String>>(res)?;

    let parsed = move_tuple
        .iter_mut()
        .filter_map(|e| e.parse::<u64>().ok())
        .collect::<Vec<u64>>();
    Ok(parsed)
}

pub async fn mint_libra(
    public_info: &mut DiemPublicInfo<'_>,
    addr: AccountAddress,
    amount: u64,
) -> anyhow::Result<()> {
    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::gas_coin_mint_to_impl(addr, amount));

    let mint_txn = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info.client().submit_and_wait(&mint_txn).await?;
    Ok(())
}
