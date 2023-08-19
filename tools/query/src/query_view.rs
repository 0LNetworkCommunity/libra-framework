use anyhow::Result;
use libra_types::type_extensions::client_ext::ClientExt;
use serde_json::{json, Value};
use zapatos_sdk::rest_client::Client;

pub async fn get_view(
    client: &Client,
    function_id: &str,
    type_args: Option<String>,
    args: Option<String>,
) -> Result<Value> {
    client.view_ext(function_id, type_args, args).await
}

pub async fn fetch_and_display(
    client: &Client,
    function_id: &str,
    type_args: Option<String>,
    args: Option<String>,
) -> Result<serde_json::Value> {
    let res = client.view_ext(function_id, type_args, args).await?;
    let json = serde_json::to_value(res)?;

    if let Value::Array(arr) = &json {
        if arr.len() == 1 {
            let key = function_id.split("::").last().unwrap_or("Result");

            return Ok(json!({ key: &arr[0] }));
        }
    }

    Ok(Value::String("Success".to_string()))
}
