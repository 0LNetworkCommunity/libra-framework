use libra_types::legacy_types::app_cfg::AppCfg;
use url::Url;
use zapatos_sdk::rest_client::Client;
use zapatos_sdk::types::chain_id::ChainId;

const LOCAL_NODE_URL: &str = "http://localhost:8080";

pub async fn find_good_upstream(app_cfg: &AppCfg) -> anyhow::Result<(Client, ChainId)> {
    //check playlist
    if let Some(playlist) = app_cfg.network_playlist.first() {
        for node in &playlist.nodes {
            let client = Client::new(node.url.clone());
            if let Ok(res) = client.get_index().await {
                return Ok((client, ChainId::new(res.inner().chain_id)));
            }
        }
    }
    Err(anyhow::anyhow!("No working nodes found in upstream_nodes"))
}

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
