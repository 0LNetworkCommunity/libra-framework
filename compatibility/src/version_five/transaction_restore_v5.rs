use crate::version_five::state_snapshot_v5::open_for_read;

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::version_five::language_storage_v5::TypeTagV5;
use diem_backup_cli::storage::FileHandle;
use diem_backup_cli::utils::read_record_bytes::ReadRecordBytes;
use diem_types::contract_event::ContractEvent;
use diem_types::transaction::Transaction;
use diem_types::transaction::TransactionInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DummyTransactionV5 {
    /// Transaction submitted by the user. e.g: P2P payment transaction, publishing module
    /// transaction, etc.
    /// TODO: We need to rename SignedTransaction to SignedUserTransaction, as well as all the other
    ///       transaction types we had in our codebase.
    #[serde(with = "serde_bytes")]

    UserTransaction(Vec<u8>),

    /// Transaction that applies a WriteSet to the current storage, it's applied manually via db-bootstrapper.
    #[serde(with = "serde_bytes")]

    GenesisTransaction(Vec<u8>),

    /// Transaction to update the block metadata resource at the beginning of a block.
    #[serde(with = "serde_bytes")]
    BlockMetadata(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DummyTransactionInfoV5 {
    /// The hash of this transaction.
    #[serde(with = "serde_bytes")]
    transaction_hash: Vec<u8>,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    #[serde(with = "serde_bytes")]
    state_root_hash: Vec<u8>,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    #[serde(with = "serde_bytes")]
    event_root_hash: Vec<u8>,

    /// The amount of gas used.
    gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's recieve more detailed information. But other errors are generally
    /// categorized with no status code or other information
    #[serde(with = "serde_bytes")]
    status: Vec<u8>,
}

/// Support versioning of the data structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum DummyContractEventV5 {
    V0(ContractEventV0),
}

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ContractEventV0 {
    /// The unique key that the event was emitted to
    #[serde(with = "serde_bytes")]
    key: Vec<u8>,
    /// The number of messages that have been emitted to the path previously
    sequence_number: u64,
    /// The type of the data
    #[serde(with = "serde_bytes")]
    type_tag: Vec<u8>,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    event_data: Vec<u8>,
}

/// Byte layout for the transaction records produced by backup-cli
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxRecord(
    TransactionV5,
    DummyTransactionInfoV5,
    Vec<DummyContractEventV5>,
);

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct DummyTop(Vec<u8>, Vec<u8>, Vec<u8>);

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
        let txn: TxRecord = bcs::from_bytes(&record_bytes)?;
        txns.push(txn);
    }
    Ok(txns)
}
