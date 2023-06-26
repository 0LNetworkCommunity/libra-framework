//! chain queries
use  anyhow::Context;
use zapatos_sdk::{
  rest_client::{
    Client,
    aptos_api_types::ViewRequest
  },
};
use libra_types::{
  type_extensions::client_ext::entry_function_id,
};

/// helper to get libra balance at a SlowWalletBalance type which shows
/// total balance and the unlocked balance.
pub async fn get_tower_difficulty(client: &Client) -> anyhow::Result<(u64, u64)> {

  let slow_balance_id = entry_function_id("tower_state", "get_difficulty")?;
  let request = ViewRequest {
      function: slow_balance_id,
      type_arguments: vec![],
      arguments: vec![],
  };
  
  let res = client.view(&request, None).await?.into_inner();

  let difficulty: u64 = serde_json::from_value(res.iter().nth(0).context("no difficulty returned")?.clone())?;
  let security: u64 = serde_json::from_value(res.iter().nth(0).context("no security param returned")?.clone())?;

  Ok((difficulty, security))
  
}
