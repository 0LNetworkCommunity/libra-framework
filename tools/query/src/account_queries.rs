use diem_api_types::Transaction;
use diem_sdk::{
    rest_client::{
        diem_api_types::{VersionedEvent, ViewRequest},
        Client,
    },
    types::{account_address::AccountAddress, validator_config::ValidatorConfig},
};
use libra_types::{
    legacy_types::tower::TowerProofHistoryView,
    move_resource::gas_coin::SlowWalletBalance,
    move_resource::txschedule::TxSchedule,
    type_extensions::client_ext::{entry_function_id, ClientExt},
};

use serde_json::json;
/// helper to get libra balance at a SlowWalletBalance type which shows
/// total balance and the unlocked balance.
pub async fn get_account_balance_libra(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<SlowWalletBalance> {
    let slow_balance_id = entry_function_id("ol_account", "balance")?;
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

pub async fn get_val_config(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<ValidatorConfig> {
    client.get_move_resource::<ValidatorConfig>(account).await
}

pub async fn get_events(
    client: &Client,
    account: AccountAddress,
    withdrawn_or_deposited: bool,
    seq_start: Option<u64>,
) -> anyhow::Result<Vec<VersionedEvent>> {
    let direction = if withdrawn_or_deposited {
        "withdraw_events"
    } else {
        "deposit_events"
    };
    let res = client
        .get_account_events(
            account,
            "0x1::coin::CoinStore<0x1::libra_coin::LibraCoin>",
            direction,
            seq_start,
            None,
        )
        .await?
        .into_inner();
    Ok(res)
}

pub async fn get_transactions(
    client: &Client,
    account: AccountAddress,
    txs_height: Option<u64>,
    txs_count: Option<u64>,
    _txs_type: Option<String>,
) -> anyhow::Result<Vec<Transaction>> {
    // TODO: filter by type (what type is it?)
    let res = client
        .get_account_transactions(account, txs_height, txs_count)
        .await?
        .into_inner();
    Ok(res)
}

pub async fn is_community_wallet_migrated(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<serde_json::Value> {
    let community_wallet_migrated_id = entry_function_id("community_wallet", "qualifies")?;
    let request = ViewRequest {
        function: community_wallet_migrated_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();
    Ok(json!(res))
}

pub async fn community_wallet_signers(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<serde_json::Value> {
    //they are empty for now
    let community_wallet_migrated_id = entry_function_id("multi_action", "get_authorities")?;
    let request = ViewRequest {
        function: community_wallet_migrated_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();
    Ok(json!(res))
}

pub async fn community_wallet_pending_transactions(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<TxSchedule> {
    client.get_move_resource::<TxSchedule>(account).await
}
