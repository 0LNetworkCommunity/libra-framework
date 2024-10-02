//! read-archive
use std::path::PathBuf;

use anyhow::{anyhow, Error, Result};
use diem_backup_cli::{
  utils::read_record_bytes::ReadRecordBytes,
  storage::{FileHandle, FileHandleRef},
  backup_types::state_snapshot::manifest::StateSnapshotChunk,
};
use diem_crypto::HashValue;
use diem_types::transaction::Version;
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncRead};

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct AccountStateBlobV5 {
    blob: Vec<u8>,
    // #[serde(skip)]
    // hash: HashValue,
}


#[derive(Deserialize, Serialize)]
pub struct StateSnapshotBackupV5 {
    /// Version at which this state snapshot is taken.
    pub version: Version,
    /// Hash of the state tree root.
    pub root_hash: HashValue,
    /// All account blobs in chunks.
    pub chunks: Vec<StateSnapshotChunk>,
    /// BCS serialized
    /// `Tuple(TransactionInfoWithProof, LedgerInfoWithSignatures)`.
    ///   - The `TransactionInfoWithProof` is at `Version` above, and carries the same `root_hash`
    /// above; It proves that at specified version the root hash is as specified in a chain
    /// represented by the LedgerInfo below.
    ///   - The signatures on the `LedgerInfoWithSignatures` has a version greater than or equal to
    /// the version of this backup but is within the same epoch, so the signatures on it can be
    /// verified by the validator set in the same epoch, which can be provided by an
    /// `EpochStateBackup` recovered prior to this to the DB; Requiring it to be in the same epoch
    /// limits the requirement on such `EpochStateBackup` to no older than the same epoch.
    pub proof: FileHandle,
}


////// SNAPSHOT FILE IO //////
/// read snapshot manifest file into object
pub fn read_from_snaphot_manifest(path: &PathBuf) -> Result<StateSnapshotBackupV5, Error> {
    let config = std::fs::read_to_string(path).map_err(|e| {
        format!("Error: cannot read file {:?}, error: {:?}", &path, &e);
        e
    })?;

    let map: StateSnapshotBackupV5 = serde_json::from_str(&config)?;

    Ok(map)
}

/// parse each chunk of a state snapshot manifest
pub async fn read_account_state_chunk(
    file_handle: FileHandle,
    archive_path: &PathBuf,
) -> Result<Vec<(HashValue, AccountStateBlobV5)>, Error> {
    let full_handle = archive_path
        .parent()
        .expect("could not read archive path")
        .join(file_handle);
    let handle_str = full_handle.to_str().unwrap();
    let mut file = open_for_read(handle_str)
        .await
        .map_err(|e| anyhow!("snapshot chunk {:?}, {:?}", &handle_str, e))?;

    let mut chunk = vec![];

    while let Some(record_bytes) = file.read_record_bytes().await? {
        chunk.push(bcs::from_bytes(&record_bytes)?);
    }
    Ok(chunk)
}

async fn open_for_read(file_handle: &FileHandleRef) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
    let file = OpenOptions::new().read(true).open(file_handle).await?;
    Ok(Box::new(file))
}
