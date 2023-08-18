use anyhow::Result;
use libra_types::type_extensions::client_ext::ClientExt;
use serde_json::{ Value, json };
use zapatos_sdk::rest_client::Client;
use crate::utils::{ colorize_and_print, print_colored_kv };


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
    args: Option<String>
) -> Result<serde_json::Value> {
    let res = client.view_ext(function_id, type_args, args).await?;
    let json = serde_json::to_value(res)?;

    if let Value::Array(arr) = &json {
        if arr.len() == 1 {

            let key = function_id.split("::").last().unwrap_or("Result");
            print_colored_kv(key, &arr[0].to_string());

            return Ok(json!({ key: &arr[0] }));
            
        }
    }

    let json_str = serde_json::to_string_pretty(&json).expect("Failed to serialize to JSON");
    colorize_and_print(&json_str)?; 
    Ok(Value::String("Success".to_string()))
}
