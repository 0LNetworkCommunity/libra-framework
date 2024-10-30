use anyhow::Result;
use diem_types::transaction::SignedTransaction;
use libra_cached_packages::libra_stdlib::EntryFunctionCall;
use libra_storage::read_tx_chunk::{load_chunk, load_tx_chunk_manifest};
use serde_json::json;
use std::path::Path;

use crate::table_structs::{WarehouseDepositTx, WarehouseGenericTx, WarehouseTxMeta};

pub async fn extract_current_transactions(archive_path: &Path) -> Result<()> {
    let manifest_file = archive_path.join("transaction.manifest");
    assert!(
        manifest_file.exists(),
        "{}",
        &format!("transaction.manifest file not found at {:?}", archive_path)
    );
    let manifest = load_tx_chunk_manifest(&manifest_file)?;

    let mut user_txs = 0;

    for each_chunk_manifest in manifest.chunks {
        let deserialized = load_chunk(archive_path, each_chunk_manifest).await?;

        for tx in deserialized.txns {
            match tx {
                diem_types::transaction::Transaction::UserTransaction(signed_transaction) => {
                    // maybe_decode_entry_function(&signed_transaction);
                    if let Ok(t) = try_decode_deposit_tx(&signed_transaction) {
                        dbg!(&t);
                    }
                    user_txs += 1;
                }
                diem_types::transaction::Transaction::GenesisTransaction(_write_set_payload) => {}
                diem_types::transaction::Transaction::BlockMetadata(_block_metadata) => {}
                diem_types::transaction::Transaction::StateCheckpoint(_hash_value) => {}
            }
        }
    }
    // TODO: use logging
    println!("user transactions found in chunk: {}", user_txs);
    Ok(())
}

pub fn decode_tx_meta(user_tx: &SignedTransaction) -> Result<WarehouseTxMeta> {
    let timestamp = user_tx.expiration_timestamp_secs();
    let sender = user_tx.sender();

    let raw = user_tx.raw_transaction_ref();
    let p = raw.clone().into_payload().clone();
    let ef = p.into_entry_function();
    let module = ef.module().to_string();
    let function = ef.function().to_string();

    Ok(WarehouseTxMeta {
        sender,
        module,
        function,
        timestamp,
    })
}

pub fn maybe_decode_generic_entry_function(
    user_tx: &SignedTransaction,
) -> Result<WarehouseGenericTx> {
    Ok(WarehouseGenericTx {
        meta: decode_tx_meta(user_tx)?,
        args: json!(""),
    })
}

pub fn try_decode_deposit_tx(user_tx: &SignedTransaction) -> Result<WarehouseDepositTx> {
    let (to, amount) = match EntryFunctionCall::decode(user_tx.payload()) {
        Some(EntryFunctionCall::OlAccountTransfer { to, amount }) => (to, amount),
        // many variants
        _ => anyhow::bail!("not a deposit tx"),
    };

    Ok(WarehouseDepositTx {
        meta: decode_tx_meta(user_tx)?,
        to,
        amount,
    })
}
