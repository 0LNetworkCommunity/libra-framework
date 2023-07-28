use zapatos_sdk::{
  rest_client::{
    Client,
    aptos_api_types::ViewRequest
  },
  types::account_address::AccountAddress,
};
use libra_types::{
  type_extensions::client_ext::{ ClientExt, entry_function_id },
  move_resource::gas_coin::SlowWalletBalance,
  legacy_types::tower::TowerProofHistoryView
};
use libra_types::exports::AuthenticationKey;
/// helper to get libra balance at a SlowWalletBalance type which shows
/// total balance and the unlocked balance.
pub async fn get_account_balance_libra(client: &Client, account: AccountAddress) -> anyhow::Result<SlowWalletBalance> {

  let slow_balance_id = entry_function_id("slow_wallet", "balance")?;
  let request = ViewRequest {
      function: slow_balance_id,
      type_arguments: vec![],
      arguments: vec![account.to_string().into()],
  };

  let res = client.view(&request, None).await?.into_inner();

  SlowWalletBalance::from_value(res)
}

pub async fn get_tower_state(client: &Client, account: AccountAddress) -> anyhow::Result<TowerProofHistoryView>{

  client.get_move_resource::<TowerProofHistoryView>(account).await

}

/// Addresses will diverge from the keypair which originally created the address.
/// The Address and AuthenticationKey key are the same bytes: a sha3 hash
/// of the public key. If you rotate the keypair for that address, the implied (derived) public key, and thus authentication key will not be the same as the
///  Origial/originating address. For this reason, we need to look up the original address
/// Addresses are stored in the OriginatingAddress table, which is a table
/// that maps a derived address to the original address. This function
/// looks up the original address for a given derived address.
pub async fn lookup_originating_address(
    client: &Client,
    authentication_key: AuthenticationKey,
) -> anyhow::Result<AccountAddress> {
  // the move View will return the same address_key if it has an unmodified Authkey (never been rotated)
  // let bytes = authentication_key.to_vec();
  // let cast_address = AccountAddress::from_bytes(bytes.as_slice())?;

  let function_id = entry_function_id("account", "get_originating_address")?;
  let request = ViewRequest {
      function: function_id,
      type_arguments: vec![],
      arguments: vec![authentication_key.to_vec().into()],
  };

  let res = client.view(&request, None).await?.into_inner();
  let addr = serde_json::from_value(res[0].clone())?;
  Ok(addr)

}


// use libra_types::exports::AccountAddress;
// use zapatos::account::key_rotation::OriginatingResource;
// use zapatos_types::account_config::CORE_CODE_ADDRESS;
// pub async fn lookup_address(
//     rest_client: &Client,
//     address_key: AccountAddress,
// ) -> anyhow::Result<AccountAddress> {
//     let originating_resource: OriginatingResource = rest_client
//         .get_account_resource_bcs(CORE_CODE_ADDRESS, "0x1::account::OriginatingAddress")
//         .await?
//         .into_inner();

//     let table_handle = originating_resource.address_map.handle;

//     // The derived address that can be used to look up the original address
//     match rest_client
//         .get_table_item_bcs(
//             table_handle,
//             "address",
//             "address",
//             address_key.to_hex_literal(),
//         )
//         .await
//     {
//         Ok(inner) => Ok(inner.into_inner()),
//         Err(err) => bail!(err),
//     }
// }