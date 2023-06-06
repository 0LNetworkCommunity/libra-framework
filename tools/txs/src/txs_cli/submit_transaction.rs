use anyhow::Result;
use libra_config::extension::client_ext::ClientExt;
use libra_txs::{rest_client::Client, types::transaction::SignedTransaction};

pub async fn submit(signed_trans: &SignedTransaction) -> Result<()> {
    let client = Client::default()?;
    let pending_trans = client.submit(signed_trans).await?.into_inner();
    let res = client.wait_for_transaction(&pending_trans).await?;

    match res.inner().success() {
        true => {
          println!("transaction success!")
        },
        false => {
          println!("transaction not successful, status: {:?}", &res.inner().vm_status());
        },
    }
    Ok(())
}
