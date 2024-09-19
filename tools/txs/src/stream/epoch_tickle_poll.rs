use diem_logger::info;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use libra_types::exports::Client;
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

pub fn epoch_tickle_poll(mut tx: Sender<TransactionPayload>, client: Client) {
    println!("polling epoch boundary");
    let handle = thread::spawn(move || loop {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // TODO: make the client borrow instead of clone
        let res = rt.block_on(libra_query::chain_queries::epoch_over_can_trigger(
            &client.clone(),
        ));
        dbg!(&res);

        match res {
            Ok(true) => {
                let func = libra_stdlib::diem_governance_trigger_epoch();

                tx.borrow_mut().send(func).unwrap();
            }
            _ => {
                info!("Not ready to call epoch.")
            }
        }

        thread::sleep(Duration::from_millis(60000));
    });
    handle.join().expect("cannot poll for epoch boundary");
}
