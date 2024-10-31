use crate::table_structs::{WarehouseDepositTx, WarehouseTxMaster};
use anyhow::Result;
use diem_types::account_config::WithdrawEvent;
use diem_types::transaction::Transaction;
use diem_types::{account_config::DepositEvent, transaction::SignedTransaction};
use libra_backwards_compatibility::sdk::v7_libra_framework_sdk_builder::EntryFunctionCall as V7EntryFunctionCall;
use libra_cached_packages::libra_stdlib::EntryFunctionCall;
use libra_storage::read_tx_chunk::{load_chunk, load_tx_chunk_manifest};
use serde_json::json;
use std::path::Path;

pub async fn extract_current_transactions(archive_path: &Path) -> Result<Vec<WarehouseTxMaster>> {
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
                .iter()
                .nth(i)
                .expect("could not index on tx_info chunk, vectors may not be same length");
            let tx_hash_info = tx_info.transaction_hash();

            let tx_events = chunk
                .event_vecs
                .iter()
                .nth(i)
                .expect("could not index on events chunk, vectors may not be same length");

            if let Some(signed_transaction) = tx.try_as_signed_user_txn() {
                let tx = make_master_tx(signed_transaction, epoch, round, timestamp)?;

                // sanity check that we are talking about the same block, and reading vectors sequentially.
                assert!(tx.tx_hash == tx_hash_info, "transaction hashes do not match in transaction vector and transaction_info vector");

                dbg!(&tx_events);
                tx_events.iter().for_each(|el| {
                    let we: Result<WithdrawEvent, _> = el.try_into();
                    let de: Result<DepositEvent, _> = el.try_into();
                    dbg!(&we);
                    dbg!(&de);
                });

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

    Ok(user_txs)
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
        sender: user_tx.sender(),
        epoch,
        round,
        block_timestamp,
        module: ef.module().to_string(),
        function: ef.function().to_string(),

        args: function_args_to_json(user_tx)?,
    };

    Ok(tx)
}

pub fn function_args_to_json(user_tx: &SignedTransaction) -> Result<serde_json::Value> {
    // TODO: match all decoding of functions from V5-V7.
    let json = match V7EntryFunctionCall::decode(user_tx.payload()) {
        Some(a) => {
            json!(a)
        }
        None => {
            // TODO: put a better error message here, but don't abort
            json!("could not parse")
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
