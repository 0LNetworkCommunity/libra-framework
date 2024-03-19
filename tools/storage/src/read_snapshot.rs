//! read-archive
use anyhow::{Error, Result};
use diem_backup_cli::{
    backup_types::{epoch_ending::manifest::EpochEndingBackup, state_snapshot::manifest::StateSnapshotBackup},
    metadata::StateSnapshotBackupMeta,
    // storage::{FileHandle, FileHandleRef},
    // utils::read_record_bytes::ReadRecordBytes,
};
use serde_json::json;
// use diem_crypto::HashValue;
// use diem_types::account_state_blob::AccountStateBlob;
use std::{
    fs,
    path::{Path, PathBuf}, str::FromStr,
};
// use tokio::{fs::OpenOptions, io::AsyncRead};

////// SNAPSHOT FILE IO //////
/// read snapshot manifest file into object
pub fn read_from_json(path: &PathBuf) -> Result<StateSnapshotBackup, Error> {
    let config = std::fs::read_to_string(path).map_err(|e| {
        format!("Error: cannot read file {:?}, error: {:?}", &path, &e);
        e
    })?;

    let map: StateSnapshotBackup = serde_json::from_str(&config)?;

    Ok(map)
}

// https://github.com/0LNetworkCommunity/diem/blob/restore-hard-forks/storage/backup/backup-cli/src/backup_types/state_snapshot/backup.rs#L260
fn load_epoch_manifest(p: &Path) -> Result<EpochEndingBackup> {
    let bytes = fs::read(p)?;
    Ok(serde_json::from_slice::<EpochEndingBackup>(&bytes)?)
}

#[test]
fn test_parse_manifest() {
  let mut this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
  this_path.push("fixtures/epoch_ending_79-.166d/epoch_ending.manifest");
  let r = load_epoch_manifest(&this_path).expect("parse manifest");
  dbg!(json!(&r));
}
// /// parse each chunk of a state snapshot manifest
// pub async fn read_account_state_chunk(
//     file_handle: FileHandle,
//     archive_path: &PathBuf,
// ) -> Result<Vec<(HashValue, AccountStateBlob)>, Error> {
//     let full_handle = archive_path
//         .parent()
//         .expect("could not read archive path")
//         .join(file_handle);
//     let handle_str = full_handle.to_str().unwrap();
//     let mut file = open_for_read(handle_str)
//         .await
//         .map_err(|e| anyhow!("snapshot chunk {:?}, {:?}", &handle_str, e))?;

//     let mut chunk = vec![];

//     while let Some(record_bytes) = file.read_record_bytes().await? {
//         chunk.push(bcs::from_bytes(&record_bytes)?);
//     }
//     Ok(chunk)
// }

// async fn open_for_read(file_handle: &FileHandleRef) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
//     let file = OpenOptions::new().read(true).open(file_handle).await?;
//     Ok(Box::new(file))
// }
