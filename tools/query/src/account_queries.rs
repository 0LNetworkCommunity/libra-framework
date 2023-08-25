
use libra_types::{
    legacy_types::tower::TowerProofHistoryView,
    move_resource::gas_coin::SlowWalletBalance,
    type_extensions::client_ext::{entry_function_id, ClientExt},
};
use zapatos_sdk::{
    rest_client::{diem_api_types::ViewRequest, Client},
    types::account_address::AccountAddress,
};
/// helper to get libra balance at a SlowWalletBalance type which shows
/// total balance and the unlocked balance.
pub async fn get_account_balance_libra(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<SlowWalletBalance> {
    let slow_balance_id = entry_function_id("slow_wallet", "balance")?;
    let request = ViewRequest {
        function: slow_balance_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();

    SlowWalletBalance::from_value(res)
}

pub async fn get_tower_state(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<TowerProofHistoryView> {
    client
        .get_move_resource::<TowerProofHistoryView>(account)
        .await
}
