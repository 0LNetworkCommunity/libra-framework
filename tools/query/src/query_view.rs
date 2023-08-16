use anyhow::Result;
use diem_sdk::rest_client::Client;
use libra_types::type_extensions::client_ext::ClientExt;
use serde_json::{Value,json};

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
        let key = function_id.split("::").last().unwrap_or("Result");

        // If the array has only one item, return it under the derived key.
        if arr.len() == 1 {
            return Ok(json!({ key: &arr[0] }));
        }

        // If the array has more than one item, return the entire array under the derived key.
        return Ok(json!({ key: arr }));
    }

    Ok(Value::String("Unable to parse response".to_string()))
}
