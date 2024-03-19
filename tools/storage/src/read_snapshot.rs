//! read-archive
use anyhow::{anyhow, Error, Result};
use diem_api_types::HashValue;
use diem_backup_cli::utils::read_record_bytes::ReadRecordBytes;
use diem_backup_cli::{
    backup_types::{
        epoch_ending::manifest::EpochEndingBackup, state_snapshot::manifest::StateSnapshotBackup,
    },
    storage::{FileHandle, FileHandleRef},
};
use diem_types::account_state::AccountState;
use tokio::{fs::OpenOptions, io::AsyncRead};
// use serde_json::json;
// use diem_crypto::HashValue;
// use diem_types::account_state_blob::AccountStateBlob;
use std::{
    fs,
    path::{Path, PathBuf},
};
// use tokio::{fs::OpenOptions, io::AsyncRead};

////// SNAPSHOT FILE IO //////
/// read snapshot manifest file into object
pub fn load_snapshot_manifest(path: &PathBuf) -> Result<StateSnapshotBackup, Error> {
    let config = std::fs::read_to_string(path).map_err(|e| {
        format!("Error: cannot read file {:?}, error: {:?}", &path, &e);
        e
    })?;

    let map: StateSnapshotBackup = serde_json::from_str(&config)?;

    Ok(map)
}

/// get the epoch manifest from file
/// NOTE: this file may sometimes be gzipped. So if it's not parsing, that may
/// be why.
// https://github.com/0LNetworkCommunity/diem/blob/restore-hard-forks/storage/backup/backup-cli/src/backup_types/state_snapshot/backup.rs#L260
pub fn load_epoch_manifest(p: &Path) -> Result<EpochEndingBackup> {
    let bytes = fs::read(p)?;
    Ok(serde_json::from_slice::<EpochEndingBackup>(&bytes)?)
}

/// parse each chunk of a state snapshot manifest
pub async fn read_account_state_chunk(
    file_handle: FileHandle,
    archive_path: &Path,
) -> Result<Vec<(HashValue, AccountState)>, Error> {
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

/// Tokio async parsing of state snapshot into blob
async fn accounts_from_snapshot_backup(
    manifest: StateSnapshotBackup,
    archive_path: &Path,
) -> anyhow::Result<Vec<AccountState>> {
    // parse AccountStateBlob from chunks of the archive
    let mut account_state_blobs: Vec<AccountState> = Vec::new();
    for chunk in manifest.chunks {
        let blobs = read_account_state_chunk(chunk.blobs, archive_path).await?;
        for (_key, blob) in blobs {
            account_state_blobs.push(blob)
        }
    }

    Ok(account_state_blobs)
}

#[test]
fn test_parse_manifest() {
    use std::str::FromStr;
    let mut this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_path.push("fixtures/state_epoch_79_ver_33217173.795d/state.manifest");
    let r = load_snapshot_manifest(&this_path).expect("parse manifest");
    dbg!(&r.epoch);
}

#[tokio::test]
async fn test_deserialize_account() {
    use std::str::FromStr;
    let mut this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_path.push("fixtures/state_epoch_79_ver_33217173.795d/state.manifest");
    let snapshot_manifest = load_snapshot_manifest(&this_path).expect("parse manifest");
    let archive_path = this_path.parent().unwrap();
    let r = accounts_from_snapshot_backup(snapshot_manifest, &archive_path)
        .await
        .expect("could not parse snapshot");
    dbg!(&r);
}
