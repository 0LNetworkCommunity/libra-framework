use diem_sdk::{
    rest_client::{diem_api_types::ViewRequest, Client},
    types::account_address::AccountAddress,
};
use libra_types::{
    legacy_types::tower::TowerProofHistoryView,
    move_resource::gas_coin::SlowWalletBalance,
    type_extensions::client_ext::{entry_function_id, ClientExt},
};
use serde_json::json;
/// helper to get libra balance at a SlowWalletBalance type which shows
/// total balance and the unlocked balance.
pub async fn get_account_balance_libra(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<serde_json::Value> {
    let slow_balance_id = entry_function_id("slow_wallet", "balance")?;
    let request = ViewRequest {
        function: slow_balance_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();
    let balance = SlowWalletBalance::from_value(res)?;

    let json = json!({
        "account": account,
        "total_balance": balance.total,
        "unlocked_balance": balance.unlocked
    });

    Ok(json)
}

pub async fn get_tower_state(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<TowerProofHistoryView> {
    client
        .get_move_resource::<TowerProofHistoryView>(account)
        .await
}
