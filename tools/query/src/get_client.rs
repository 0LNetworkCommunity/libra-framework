// TODO:

use anyhow::Context;
use libra_types::legacy_types::app_cfg::AppCfg;
use diem_sdk::rest_client::Client;
use diem_sdk::types::chain_id::ChainId;

/// Finds a good upstream client and its associated chain ID.
pub async fn find_good_upstream(app_cfg: &AppCfg) -> anyhow::Result<(Client, ChainId)> {

  let nodes = &app_cfg.profile.upstream_nodes;
  let url = nodes.iter().next().context("cannot get url")?;
  let client = Client::new(url.to_owned());
  let res = client.get_index().await?;

  Ok((client, ChainId::new(res.inner().chain_id)))
}
