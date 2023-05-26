use anyhow::Result;
use libra_config::extension::client_ext::ClientExt;
use libra_txs::{rest_client::Client, types::transaction::SignedTransaction};

pub async fn run(signed_trans: &SignedTransaction) -> Result<()> {
    let client = Client::default()?;
    let pending_trans = client.submit(signed_trans).await?.into_inner();
    client.wait_for_transaction(&pending_trans).await?;
    Ok(())
}
