use crate::submit_transaction::Sender as LibraSender;
use diem_logger::info;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub async fn epoch_tickle_poll(
    mut tx: Sender<TransactionPayload>,
    sender: Arc<Mutex<LibraSender>>,
    delay_secs: u64,
) -> anyhow::Result<()> {
    println!("polling epoch boundary");

    loop {
        let client = sender.lock().unwrap().client().clone();

        // TODO: make the client borrow instead of clone
        let res = libra_query::chain_queries::epoch_over_can_trigger(&client.clone()).await;

        match res {
            Ok(true) => {
                let func = libra_stdlib::diem_governance_trigger_epoch();

                tx.borrow_mut()
                    .send(func)
                    .expect("could not send message to channel");
            }
            _ => {
                info!("Not ready to call epoch.")
            }
        }

        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }
}
