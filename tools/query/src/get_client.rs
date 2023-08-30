use url::Url;
use zapatos_sdk::rest_client::Client;
use zapatos_sdk::types::chain_id::ChainId;

const LOCAL_NODE_URL: &str = "http://localhost:8080";

pub async fn get_local_node() -> anyhow::Result<(Client, ChainId)> {
    let local_url = Url::parse(LOCAL_NODE_URL)?;
    let client = Client::new(local_url);
    match client.get_index().await {
        Ok(res) => Ok((client, ChainId::new(res.inner().chain_id))),
        Err(e) => Err(anyhow::anyhow!(
            "Failed to connect to the local node: {}",
            e
        )),
    }
}
