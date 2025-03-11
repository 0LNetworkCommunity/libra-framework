use crate::submit_transaction::Sender as LibraSender;
use diem_logger::info;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use libra_types::exports::Client;
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub fn epoch_tickle_poll(
    mut tx: Sender<TransactionPayload>,
    sender: Arc<Mutex<LibraSender>>,
    delay_secs: u64,
) {
    println!("polling epoch boundary");

    let handle = thread::spawn(move || loop {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let client = sender.lock().unwrap().client().clone();

        // TODO: make the client borrow instead of clone
        let res = rt.block_on(libra_query::chain_queries::epoch_over_can_trigger(
            &client.clone(),
        ));

        match res {
            Ok(true) => {
                let func = libra_stdlib::diem_governance_trigger_epoch();

                tx.borrow_mut().send(func).unwrap();
            }
            _ => {
                info!("Not ready to call epoch.")
            }
        }

        thread::sleep(Duration::from_secs(delay_secs));
    });
    handle.join().expect("cannot poll for epoch boundary");
}
