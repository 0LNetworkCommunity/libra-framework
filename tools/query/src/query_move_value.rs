use crate::query_type::OutputType;
use anyhow::{anyhow, Result};
use diem_sdk::{rest_client::Client, types::account_address::AccountAddress};

pub async fn get_account_move_value(
    client: &Client,
    account: &AccountAddress,
    module_name: &str,
    struct_name: &str,
    key_name: &str,
) -> Result<OutputType> {
    let res = &client
        .get_account_resources(*account)
        .await?
        .into_inner()
        .into_iter()
        .map(|resource| {
            let mut map = serde_json::Map::new();
            map.insert(resource.resource_type.to_string(), resource.data);
            serde_json::Value::Object(map)
        })
        .collect::<Vec<serde_json::Value>>();

    let module_search_pattern = format!("::{}::", module_name);

    if let Some(module_struct) = res.iter().find(|value| {
        if let Some(map) = value.as_object() {
            map.keys()
                .any(|k| k.contains(&module_search_pattern) && k.ends_with(struct_name))
        } else {
            false
        }
    }) {
        if let Some(struct_data) = module_struct
            .as_object()
            .and_then(|map| map.values().next())
        {
            if let Some(key_value) = struct_data.get(key_name) {
                let parsed_value = key_value
                    .as_str()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or_default();

                let output_json = serde_json::json!({
                    "account": account,
                    "module_name": module_name,
                    "module_struct": struct_name,
                    key_name: parsed_value
                });

                let output_str = serde_json::to_string_pretty(&output_json)?;

                Ok(OutputType::Json(output_str))
            } else {
                Err(anyhow!(
                    "Key '{}' not found in struct '{}'",
                    key_name,
                    struct_name
                ))
            }
        } else {
            Err(anyhow!(
                "Struct '{}' not found in module '{}'",
                struct_name,
                module_name
            ))
        }
    } else {
        Err(anyhow!("Module '{}' not found", module_name))
    }
}
