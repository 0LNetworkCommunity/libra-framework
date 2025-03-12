use crate::submit_transaction::Sender as LibraSender;
use diem_logger::info;
use libra_cached_packages::libra_stdlib;
use std::time::Duration;

pub async fn epoch_tickle_poll(sender: &mut LibraSender, delay_secs: u64) -> anyhow::Result<()> {
    println!("polling epoch boundary");
    let client = sender.client().clone();
    loop {
        let res = libra_query::chain_queries::epoch_over_can_trigger(&client).await;

        match res {
            Ok(true) => {
                let payload = libra_stdlib::diem_governance_trigger_epoch();

                sender.sign_submit_wait(payload).await?;
            }
            _ => {
                info!("Not ready to call epoch.")
            }
        }

        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }
}
