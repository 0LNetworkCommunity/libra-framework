use anyhow::Result;
use libra_types::type_extensions::client_ext::ClientExt;
use serde_json::Value;
use diem_sdk::rest_client::Client;

pub async fn get_view(
    client: &Client,
    function_id: &str,
    type_args: Option<String>,
    args: Option<String>,
) -> Result<Value> {
    client.view_ext(function_id, type_args, args).await
}

// helper to turn a serde_json value to string
// TODO: must be a better way
pub fn display_view(res: Vec<Value>) -> Result<String> {
    let values_to_string = res.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    if values_to_string.len() > 1 {
        Ok(format!("[{}]", values_to_string.join(", ")))
    } else {
        Ok(format!(
            "[{}]",
            values_to_string.first().expect("api didn't return a value")
        ))
    }
}
