use zapatos_sdk::{
  rest_client::{
    Client,
    aptos_api_types::ViewRequest
  },
  types::account_address::AccountAddress,
};
use libra_types::{
  type_extensions::client_ext::{ ClientExt, entry_function_id },
  gas_coin::SlowWalletBalance,
  tower::TowerProofHistoryView
};

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
pub async fn lookup_address(
    rest_client: &Client,
    address_key: AccountAddress,
    must_exist: bool,
) -> Result<AccountAddress, RestError> {
    let originating_resource: OriginatingResource = rest_client
        .get_account_resource_bcs(CORE_CODE_ADDRESS, "0x1::account::OriginatingAddress")
        .await?
        .into_inner();

    let table_handle = originating_resource.address_map.handle;

    // The derived address that can be used to look up the original address
    match rest_client
        .get_table_item_bcs(
            table_handle,
            "address",
            "address",
            address_key.to_hex_literal(),
        )
        .await
    {
        Ok(inner) => Ok(inner.into_inner()),
        Err(RestError::Api(AptosErrorResponse {
            error:
                AptosError {
                    error_code: AptosErrorCode::TableItemNotFound,
                    ..
                },
            ..
        })) => {
            // If the table item wasn't found, we may check if the account exists
            if !must_exist {
                Ok(address_key)
            } else {
                rest_client
                    .get_account_bcs(address_key)
                    .await
                    .map(|_| address_key)
            }
        },
        Err(err) => Err(err),
    }
}
