use anyhow::Result;
use libra_config::extension::client_ext::ClientExt;
use libra_txs::{extension::client_ext::ClientExt as ConfigClientExt, rest_client::Client};

pub async fn run(
    function_id: &str,
    type_args: Option<String>,
    args: Option<String>,
) -> Result<String> {
    let client = Client::default()?;
    let result = client
        .view_ext(function_id, type_args, args)
        .await?
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    println!("\n=======OUTPUT=======");
    Ok(format!("[{}]", result.join(", ")))
}
