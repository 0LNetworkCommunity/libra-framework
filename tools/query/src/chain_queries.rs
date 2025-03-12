//! chain queries

use crate::query_view::{self, get_view};
use anyhow::Context;
use diem_sdk::rest_client::Client;

/// Retrieves the current epoch from the blockchain.
pub async fn get_epoch(client: &Client) -> anyhow::Result<u64> {
    let res = get_view(client, "0x1::epoch_helper::get_current_epoch", None, None).await?;

    let value: Vec<String> = serde_json::from_value(res)?;
    let num = value.first().unwrap().parse::<u64>()?;

    Ok(num)
}

// COMMIT NOTE: deprecated tower functions

/// Retrieves the ID of the next governance proposal.
pub async fn get_next_governance_proposal_id(client: &Client) -> anyhow::Result<u64> {
    let query_res = query_view::get_view(
        client,
        "0x1::diem_governance::get_next_governance_proposal_id",
        None,
        None,
    )
    .await?;
    // let id: Vec<String> = serde_json::from_value(query_res)?;
    let num: u64 = serde_json::from_value::<Vec<String>>(query_res)?
        .first()
        .context("could not get a response from view function get_next_governance_proposal_id")?
        .parse()?;
    Ok(num)
}

/// Checks if a governance proposal can be resolved.
pub async fn can_gov_proposal_resolve(client: &Client, id: u64) -> anyhow::Result<bool> {
    let query_res = query_view::get_view(
        client,
        "0x1::diem_governance::get_can_resolve",
        None,
        Some(id.to_string()),
    )
    .await?;

    serde_json::from_value::<bool>(query_res).context("cannot parse api res")
}

// TODO: code duplication
/// Checks if a governance proposal with the given ID has been resolved.
pub async fn is_gov_proposal_resolved(client: &Client, id: u64) -> anyhow::Result<bool> {
    let query_res = query_view::get_view(
        client,
        "0x1::diem_governance::is_resolved",
        None,
        Some(id.to_string()),
    )
    .await?;

    serde_json::from_value::<Vec<bool>>(query_res)?
        .into_iter()
        .next()
        .context("could not get a response from view function is_resolved")
}

// TODO: code duplication
/// Retrieves votes for a governance proposal with the given ID.
pub async fn get_gov_proposal_votes(client: &Client, id: u64) -> anyhow::Result<Vec<u128>> {
    let query_res = query_view::get_view(
        client,
        "0x1::diem_governance::get_votes",
        None,
        Some(id.to_string()),
    )
    .await?;

    Ok(serde_json::from_value::<Vec<u128>>(query_res)?)
}

/// Retrieves the current blockchain height.
pub async fn get_height(client: &Client) -> anyhow::Result<u64> {
    let res = get_view(client, "0x1::block::get_current_block_height", None, None).await?;

    let value: Vec<String> = serde_json::from_value(res)?;
    let height = value.first().unwrap().parse::<u64>()?;

    Ok(height)
}

/// Retrieves the current blockchain height.
pub async fn epoch_over_can_trigger(client: &Client) -> anyhow::Result<bool> {
    let res = get_view(client, "0x1::epoch_boundary::can_trigger", None, None).await?;

    let value: Vec<bool> = serde_json::from_value(res)?;

    Ok(value[0])
}

/// Retrieves if we are within the commit reveal window
pub async fn within_commit_reveal_window(client: &Client) -> anyhow::Result<bool> {
    let res = get_view(client, "0x1::secret_bid::in_reveal_window", None, None).await?;

    let value: Vec<bool> = serde_json::from_value(res)?;

    Ok(value[0])
}

/// Time remaining in epoch
pub async fn secs_remaining_in_epoch(client: &Client) -> anyhow::Result<u64> {
    let res = get_view(client, "0x1::reconfiguration::get_remaining_epoch_secs", None, None).await?;

    let value: Vec<String> = serde_json::from_value(res)?;
    let secs = value.first().unwrap().parse::<u64>()?;

    Ok(secs)
}
