use anyhow::Result;
use libra_types::type_extensions::client_ext::ClientExt;
use zapatos_sdk::rest_client::Client;

pub async fn run(
    function_id: &str,
    type_args: Option<String>,
    args: Option<String>,
) -> Result<String> {
    let client = Client::default().await?;
    let result = client
        .view_ext(function_id, type_args, args)
        .await?
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    // println!("\n=======OUTPUT=======");
    Ok(format!("[{}]", result.join(", ")))
}
