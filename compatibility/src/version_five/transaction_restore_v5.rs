use crate::version_five::state_snapshot_v5::open_for_read;

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use diem_backup_cli::storage::FileHandle;
use diem_backup_cli::utils::read_record_bytes::ReadRecordBytes;
use diem_types::contract_event::ContractEvent;
use diem_types::transaction::Transaction;
use diem_types::transaction::TransactionInfo;
use serde::{Deserialize, Serialize};


/// Byte layout for the transaction records produced by backup-cli
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxRecord(Transaction, TransactionInfo, Vec<ContractEvent>);

/// parse each chunk of a state snapshot manifest
pub async fn read_transaction_chunk(
    file_handle: &FileHandle,
    archive_path: &PathBuf,
) -> Result<Vec<TxRecord>> {
    let full_handle = archive_path
        .parent()
        .expect("could not read archive path")
        .join(file_handle);
    let handle_str = full_handle.to_str().unwrap();
    assert!(full_handle.exists(), "file does not exist");

    let mut file = open_for_read(handle_str)
        .await
        .map_err(|e| anyhow!("transaction chunk {:?}, {:?}", &handle_str, e))?;

    let mut txns = vec![];
    while let Some(record_bytes) = file.read_record_bytes().await? {
        dbg!(&record_bytes);
        std::fs::write(archive_path.join("one_tx_string.txt"), &record_bytes);
        let txn: TxRecord = bcs::from_bytes(&record_bytes)?;
        txns.push(txn);
    }
    Ok(txns)
}
