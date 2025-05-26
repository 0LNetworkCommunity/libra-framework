//! Helper functions for querying account-related data using the Diem SDK client.

use diem_sdk::{
    rest_client::{
        diem_api_types::{Transaction, VersionedEvent, ViewRequest},
        Client,
    },
    types::{account_address::AccountAddress, validator_config::ValidatorConfig},
};
use libra_types::{
    move_resource::{gas_coin::SlowWalletBalance, txschedule::TxSchedule},
    type_extensions::client_ext::{entry_function_id, ClientExt},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Structured data for account vouch report
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountVouchReportData {
    pub account: String,
    pub cached_score: Option<u64>,
    pub fresh_score: Option<u64>,
    pub max_depth_reached: Option<u64>,
    pub accounts_processed: Option<u64>,
    pub max_vouches_by_score: Option<u64>,
    pub remaining_vouches_available: Option<u64>,
    pub errors: Vec<String>,
}

/// helper to get libra balance at a SlowWalletBalance type which shows
/// total balance and the unlocked balance. s
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

/// Retrieves the validator configuration for a given account.
pub async fn get_val_config(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<ValidatorConfig> {
    client.get_move_resource::<ValidatorConfig>(account).await
}

/// Retrieves events associated with a given account.
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

/// Retrieves transactions associated with a given account.
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

/// Checks if the community wallet for a given account has been migrated.
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

/// Retrieves signers for the community wallet associated with a given account.
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

/// Retrieves scheduled transactions for the community wallet associated with a given account.
pub async fn community_wallet_scheduled_transactions(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<TxSchedule> {
    client.get_move_resource::<TxSchedule>(account).await
}

/// Retrieves all multi_auth actions (pending, approved, expired) for a given multi_auth account.
pub async fn multi_auth_ballots(
    client: &Client,
    multi_auth_account: AccountAddress,
) -> anyhow::Result<Value> {
    let resource_path_str = "0x1::multi_action::Action<0x1::donor_voice_txs::Payment>";
    let proposal_state = client
        .get_account_resource(multi_auth_account, resource_path_str)
        .await?;
    let r = proposal_state.inner().clone().unwrap();

    Ok(r.data)
}

/// Calculates a fresh page rank trust score for an account without updating the cache.
/// Returns (score, max_depth_reached, accounts_processed) as a tuple.
pub async fn page_rank_calculate_score(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<(u64, u64, u64)> {
    let calculate_score_id = entry_function_id("page_rank_lazy", "calculate_score")?;
    let request = ViewRequest {
        function: calculate_score_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();

    // Parse the tuple response (score, max_depth_reached, accounts_processed)
    if res.len() != 3 {
        return Err(anyhow::anyhow!(
            "Expected 3 values from calculate_score, got {}",
            res.len()
        ));
    }

    // Handle string or numeric responses from the Move VM
    let score: u64 = match &res[0] {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid number format for score"))?,
        _ => return Err(anyhow::anyhow!("Unexpected response type for score")),
    };

    let max_depth_reached: u64 = match &res[1] {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid number format for max_depth"))?,
        _ => return Err(anyhow::anyhow!("Unexpected response type for max_depth")),
    };

    let accounts_processed: u64 = match &res[2] {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid number format for accounts_processed"))?,
        _ => return Err(anyhow::anyhow!("Unexpected response type for accounts_processed")),
    };

    Ok((score, max_depth_reached, accounts_processed))
}

/// Retrieves the cached page rank trust score for an account.
/// This returns the previously computed score without recalculation.
pub async fn page_rank_get_cached_score(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<u64> {
    let get_cached_score_id = entry_function_id("page_rank_lazy", "get_cached_score")?;
    let request = ViewRequest {
        function: get_cached_score_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();

    // Parse the single u64 response
    if res.is_empty() {
        return Err(anyhow::anyhow!("No values returned from get_cached_score"));
    }

    // Handle both string and numeric responses from the Move VM
    let score: u64 = match &res[0] {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid number format"))?,
        _ => return Err(anyhow::anyhow!("Unexpected response type for cached score")),
    };
    Ok(score)
}

/// Calculates the maximum number of vouches a user should be able to give based on their trust score.
/// This is determined by the user's page rank trust score relative to the maximum score in the system.
pub async fn vouch_limits_calculate_score_limit(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<u64> {
    let calculate_score_limit_id = entry_function_id("vouch_limits", "calculate_score_limit")?;
    let request = ViewRequest {
        function: calculate_score_limit_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();

    // Parse the single u64 response
    if res.is_empty() {
        return Err(anyhow::anyhow!(
            "No values returned from calculate_score_limit"
        ));
    }

    // Handle both string and numeric responses from the Move VM
    let limit: u64 = match &res[0] {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid number format"))?,
        _ => return Err(anyhow::anyhow!("Unexpected response type for score limit")),
    };
    Ok(limit)
}

/// Returns the number of vouches a user can still give based on system limits.
/// This takes into account all constraints: base maximum limit, score-based limit,
/// received vouches + 1 limit, and per-epoch limit.
pub async fn vouch_limits_get_vouch_limit(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<u64> {
    let get_vouch_limit_id = entry_function_id("vouch_limits", "get_vouch_limit")?;
    let request = ViewRequest {
        function: get_vouch_limit_id,
        type_arguments: vec![],
        arguments: vec![account.to_string().into()],
    };

    let res = client.view(&request, None).await?.into_inner();

    // Parse the single u64 response
    if res.is_empty() {
        return Err(anyhow::anyhow!("No values returned from get_vouch_limit"));
    }

    // Handle both string and numeric responses from the Move VM
    let limit: u64 = match &res[0] {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| anyhow::anyhow!("Invalid number format"))?,
        _ => return Err(anyhow::anyhow!("Unexpected response type for vouch limit")),
    };
    Ok(limit)
}

/// Creates a comprehensive vouch report for an account, combining page rank scores and vouch limits.
/// This function returns structured data that can be used for JSON output or further processing.
pub async fn account_vouch_report(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<AccountVouchReportData> {
    let mut errors = Vec::new();

    // Get page rank scores
    let cached_score = match page_rank_get_cached_score(client, account).await {
        Ok(score) => Some(score),
        Err(e) => {
            errors.push(format!("cached_score: {}", e));
            None
        }
    };

    let (fresh_score, max_depth_reached, accounts_processed) =
        match page_rank_calculate_score(client, account).await {
            Ok((score, max_depth_reached, accounts_processed)) => (
                Some(score),
                Some(max_depth_reached),
                Some(accounts_processed),
            ),
            Err(e) => {
                errors.push(format!("fresh_score: {}", e));
                (None, None, None)
            }
        };

    // Get vouch limits
    let max_vouches_by_score = match vouch_limits_calculate_score_limit(client, account).await {
        Ok(limit) => Some(limit),
        Err(e) => {
            errors.push(format!("max_vouches_by_score: {}", e));
            None
        }
    };

    let remaining_vouches_available = match vouch_limits_get_vouch_limit(client, account).await {
        Ok(limit) => Some(limit),
        Err(e) => {
            errors.push(format!("remaining_vouches_available: {}", e));
            None
        }
    };

    Ok(AccountVouchReportData {
        account: account.to_string(),
        cached_score,
        fresh_score,
        max_depth_reached,
        accounts_processed,
        max_vouches_by_score,
        remaining_vouches_available,
        errors,
    })
}

/// Prints a comprehensive vouch report for an account to the console.
/// This function provides a readable summary of an account's trust metrics and vouching capabilities.
pub async fn account_vouch_report_console(
    client: &Client,
    account: AccountAddress,
) -> anyhow::Result<()> {
    let report = account_vouch_report(client, account).await?;

    println!("=== Account Vouch Report for {} ===\n", account);

    // Page rank scores
    println!("Page Rank Trust Scores:");

    match report.cached_score {
        Some(score) => println!("  • Cached Trust Score: {}", score),
        None => println!("  • Cached Trust Score: Not available"),
    }

    match (report.fresh_score, report.max_depth_reached, report.accounts_processed) {
        (Some(score), Some(max_depth), Some(accounts_processed)) => {
            println!("  • Fresh Trust Score: {}", score);
            println!("  • Max Depth Reached: {}", max_depth);
            println!("  • Accounts Processed: {}", accounts_processed);
        }
        _ => println!("  • Fresh Trust Score: Not available"),
    }

    println!();

    // Vouch limits
    println!("Vouching Limits:");

    match report.max_vouches_by_score {
        Some(limit) => println!("  • Max Vouches (based on trust score): {}", limit),
        None => println!("  • Max Vouches (based on trust score): Not available"),
    }

    match report.remaining_vouches_available {
        Some(limit) => println!("  • Remaining Vouches Available: {}", limit),
        None => println!("  • Remaining Vouches Available: Not available"),
    }

    // Show errors if any
    if !report.errors.is_empty() {
        println!("\nErrors encountered:");
        for error in &report.errors {
            println!("  • {}", error);
        }
    }

    println!("\n=== End of Report ===");
    Ok(())
}
