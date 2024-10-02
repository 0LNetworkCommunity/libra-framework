//! read-archive

use crate::version_five_account_blob::AccountStateBlob;

use std::path::PathBuf;
use anyhow::{anyhow, Context, Error, Result};
use diem_backup_cli::{
  utils::read_record_bytes::ReadRecordBytes,
  storage::{FileHandle, FileHandleRef},
  backup_types::state_snapshot::manifest::StateSnapshotChunk,
};
use diem_crypto::HashValue;
use diem_types::transaction::Version;
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncRead};

// #[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
// pub struct AccountStateBlob {
//     blob: Vec<u8>,
//     // #[serde(skip)]
//     hash: HashValue,
// }


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
pub fn v5_read_from_snapshot_manifest(path: &PathBuf) -> Result<StateSnapshotBackupV5, Error> {
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
) -> Result<Vec<(HashValue, AccountStateBlob)>, Error> {
    let full_handle = archive_path
        .parent()
        .expect("could not read archive path")
        .join(file_handle);
    let handle_str = full_handle.to_str().unwrap();
    assert!(full_handle.exists(), "file does not exist");

    let mut file = open_for_read(handle_str)
        .await
        .map_err(|e| anyhow!("snapshot chunk {:?}, {:?}", &handle_str, e))?;

    let mut chunk = vec![];

    while let Some(record_bytes) = file.read_record_bytes().await? {
        dbg!(&record_bytes.len());
        dbg!(&hex::encode(&record_bytes));
        let bcs = bcs_old::from_bytes(&record_bytes).context("could not deserialize bcs chunk")?;
        chunk.push(bcs);
    }
    Ok(chunk)
}

async fn open_for_read(file_handle: &FileHandleRef) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
    let file = OpenOptions::new().read(true).open(file_handle).await?;
    Ok(Box::new(file))
}


/// Tokio async parsing of state snapshot into blob
pub async fn v5_accounts_from_snapshot_backup(
    manifest: StateSnapshotBackupV5,
    archive_path: &PathBuf,
) -> Result<Vec<AccountStateBlob>, Error> {
    // parse AccountStateBlob from chunks of the archive
    let mut account_state_blobs: Vec<AccountStateBlob> = Vec::new();
    let mut i = 0;
    for chunk in manifest.chunks {
        dbg!(&i);

        let blobs = read_account_state_chunk(chunk.blobs, archive_path).await?;

        for (_key, blob) in blobs {
            account_state_blobs.push(blob)
        }
        i += 1;
    }

    Ok(account_state_blobs)
}

#[test]
fn test_hex() {
  use diem_crypto::HashValue;

  let st = "2000035a7c9629b7e52e94de65e6a892701fe28351186405be96ed23f4b1252f7ad403061f010000000000000000000000000000000105526f6c657306526f6c65496400080a000000000000002801000000000000000000000000000000010852656365697074730c5573657252656365697074730004000000002a01000000000000000000000000000000010b4469656d4163636f756e740b4469656d4163636f756e74008d012089455024b0c9157c6a6103af2db498f4c48fd6f98292da33b11c4878b36dde1b01c48fd6f98292da33b11c4878b36dde1b01c48fd6f98292da33b11c4878b36dde1b0100000000000000180000000000000000c48fd6f98292da33b11c4878b36dde1b0000000000000000180100000000000000c48fd6f98292da33b11c4878b36dde1b00000000000000002d0100000000000000000000000000000001054576656e74144576656e7448616e646c6547656e657261746f7200180200000000000000c48fd6f98292da33b11c4878b36dde1b2e01000000000000000000000000000000010f4163636f756e74467265657a696e670b467265657a696e674269740001004001000000000000000000000000000000010b4469656d4163636f756e740742616c616e63650107000000000000000000000000000000010347415303474153000840420f0000000000";
  let bytes = hex::decode(st).unwrap();
  // dbg!(&bytes);
  let a: (HashValue, AccountStateBlob) = bcs::from_bytes(&bytes).unwrap();
  dbg!(&a);

  // let decoded = bcs::from_bytes(&bytes)
  //   .map(|account_state: (HashValue, AccountStateBlob)| format!("{:#?}", account_state))
  //   .unwrap_or_else(|_| String::from("[fail]"));
  // let decoded: AccountStateBlob = bcs::from_bytes(&bytes).unwrap();
  // dbg!(&decoded);
}
