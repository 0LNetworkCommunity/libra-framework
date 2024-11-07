use crate::table_structs::{WarehouseDepositTx, WarehouseEvent, WarehouseTxMaster};
use anyhow::Result;
use diem_crypto::HashValue;
use diem_types::account_config::{NewBlockEvent, WithdrawEvent};
use diem_types::contract_event::ContractEvent;
use diem_types::{account_config::DepositEvent, transaction::SignedTransaction};
use libra_backwards_compatibility::sdk::v6_libra_framework_sdk_builder::EntryFunctionCall as V6EntryFunctionCall;
use libra_backwards_compatibility::sdk::v7_libra_framework_sdk_builder::EntryFunctionCall as V7EntryFunctionCall;
use libra_cached_packages::libra_stdlib::EntryFunctionCall;
use libra_storage::read_tx_chunk::{load_chunk, load_tx_chunk_manifest};
use serde_json::json;
use std::path::Path;

pub async fn extract_current_transactions(
    archive_path: &Path,
) -> Result<(Vec<WarehouseTxMaster>, Vec<WarehouseEvent>)> {
    let manifest_file = archive_path.join("transaction.manifest");
    assert!(
        manifest_file.exists(),
        "{}",
        &format!("transaction.manifest file not found at {:?}", archive_path)
    );
    let manifest = load_tx_chunk_manifest(&manifest_file)?;

    let mut user_txs_in_chunk = 0;
    let mut epoch = 0;
    let mut round = 0;
    let mut timestamp = 0;

    let mut user_txs: Vec<WarehouseTxMaster> = vec![];
    let mut events: Vec<WarehouseEvent> = vec![];

    for each_chunk_manifest in manifest.chunks {
        let chunk = load_chunk(archive_path, each_chunk_manifest).await?;

        for (i, tx) in chunk.txns.iter().enumerate() {
            // TODO: unsure if this is off by one
            // perhaps reverse the vectors before transforming

            // first increment the block metadata. This assumes the vector is sequential.
            if let Some(block) = tx.try_as_block_metadata() {
                // check the epochs are incrementing or not
                if epoch > block.epoch()
                    && round > block.round()
                    && timestamp > block.timestamp_usecs()
                {
                    dbg!(
                        epoch,
                        block.epoch(),
                        round,
                        block.round(),
                        timestamp,
                        block.timestamp_usecs()
                    );
                }

                epoch = block.epoch();
                round = block.round();
                timestamp = block.timestamp_usecs();
            }

            let tx_info = chunk
                .txn_infos
                .get(i)
                .expect("could not index on tx_info chunk, vectors may not be same length");
            let tx_hash_info = tx_info.transaction_hash();

            let tx_events = chunk
                .event_vecs
                .get(i)
                .expect("could not index on events chunk, vectors may not be same length");

            let mut decoded_events = decode_events(tx_hash_info, tx_events)?;
            events.append(&mut decoded_events);

            if let Some(signed_transaction) = tx.try_as_signed_user_txn() {
                let tx = make_master_tx(signed_transaction, epoch, round, timestamp)?;

                // sanity check that we are talking about the same block, and reading vectors sequentially.
                assert!(tx.tx_hash == tx_hash_info, "transaction hashes do not match in transaction vector and transaction_info vector");

                user_txs.push(tx);

                user_txs_in_chunk += 1;
            }
        }
    }
    // TODO: use logging
    assert!(
        user_txs_in_chunk == user_txs.len(),
        "don't parse same amount of user txs as in chunk"
    );
    println!("user transactions found in chunk: {}", user_txs_in_chunk);

    Ok((user_txs, events))
}

pub fn make_master_tx(
    user_tx: &SignedTransaction,
    epoch: u64,
    round: u64,
    block_timestamp: u64,
) -> Result<WarehouseTxMaster> {
    let tx_hash = user_tx.clone().committed_hash();
    let raw = user_tx.raw_transaction_ref();
    let p = raw.clone().into_payload().clone();
    let ef = p.into_entry_function();

    let tx = WarehouseTxMaster {
        tx_hash,
        expiration_timestamp: user_tx.expiration_timestamp_secs(),
        sender: user_tx.sender().to_hex_literal(),
        epoch,
        round,
        block_timestamp,
        function: format!("{}::{}", ef.module().short_str_lossless(), ef.function()),
        recipients: None,
        args: function_args_to_json(user_tx)?,
    };

    Ok(tx)
}

pub fn decode_events(
    tx_hash: HashValue,
    tx_events: &[ContractEvent],
) -> Result<Vec<WarehouseEvent>> {
    let list: Vec<WarehouseEvent> = tx_events
        .iter()
        .filter_map(|el| {
            // too much noise
            if NewBlockEvent::try_from_bytes(el.event_data()).is_ok() {
                return None;
            }

            let event_name = el.type_tag().to_canonical_string();

            let mut data = json!("unknown data");

            if let Ok(e) = WithdrawEvent::try_from_bytes(el.event_data()) {
                data = json!(e);
            }

            if let Ok(e) = DepositEvent::try_from_bytes(el.event_data()) {
                data = json!(e);
            }

            Some(WarehouseEvent {
                tx_hash,
                event_name,
                data,
            })
        })
        .collect();

    Ok(list)
}

pub fn function_args_to_json(user_tx: &SignedTransaction) -> Result<serde_json::Value> {
    // TODO: match all decoding of functions from V5-V7.
    let json = match V7EntryFunctionCall::decode(user_tx.payload()) {
        Some(a) => {
            json!(a)
        }
        _ => {
            // NOW Try the V6 payloads
            match V6EntryFunctionCall::decode(user_tx.payload()) {
                Some(a) => json!(a),
                _ => {
                    // TODO: put a better error message here, but don't abort
                    json!({ "error": "could not decode function bytes"})
                }
            }
        }
    };

    Ok(json)
}

// TODO: unsure if this needs to happen on Rust side
fn _try_decode_deposit_tx(user_tx: &SignedTransaction) -> Result<WarehouseDepositTx> {
    let (to, amount) = match EntryFunctionCall::decode(user_tx.payload()) {
        Some(EntryFunctionCall::OlAccountTransfer { to, amount }) => (to, amount),
        // many variants
        _ => anyhow::bail!("not a deposit tx"),
    };

    Ok(WarehouseDepositTx {
        tx_hash: user_tx.clone().committed_hash(),
        to,
        amount,
    })
}
