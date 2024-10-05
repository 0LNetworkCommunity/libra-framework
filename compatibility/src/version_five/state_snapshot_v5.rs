//! read-archive

use crate::version_five::{account_blob_v5::AccountStateBlob, hash_value_v5::HashValue};

use anyhow::{anyhow, Context, Error, Result};
use diem_backup_cli::{
    backup_types::state_snapshot::manifest::StateSnapshotChunk,
    storage::{FileHandle, FileHandleRef},
    utils::read_record_bytes::ReadRecordBytes,
};
use diem_types::transaction::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{fs::OpenOptions, io::AsyncRead};

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
        dbg!(&record_bytes);
        let bcs = bcs::from_bytes(&record_bytes).context("could not deserialize bcs chunk")?;
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
fn test_string() {
    use super::account_blob_v5::AccountStateV5;

    use super::balance_v5::BalanceResource;
    use super::freezing_v5::FreezingBit;

    let bytes = b" \0\x03Z|\x96)\xb7\xe5.\x94\xdee\xe6\xa8\x92p\x1f\xe2\x83Q\x18d\x05\xbe\x96\xed#\xf4\xb1%/z\xd4\x03\x06\x1f\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x05Roles\x06RoleId\0\x08\n\0\0\0\0\0\0\0(\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x08Receipts\x0cUserReceipts\0\x04\0\0\0\0*\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x0bDiemAccount\x0bDiemAccount\0\x8d\x01 \x89EP$\xb0\xc9\x15|ja\x03\xaf-\xb4\x98\xf4\xc4\x8f\xd6\xf9\x82\x92\xda3\xb1\x1cHx\xb3m\xde\x1b\x01\xc4\x8f\xd6\xf9\x82\x92\xda3\xb1\x1cHx\xb3m\xde\x1b\x01\xc4\x8f\xd6\xf9\x82\x92\xda3\xb1\x1cHx\xb3m\xde\x1b\x01\0\0\0\0\0\0\0\x18\0\0\0\0\0\0\0\0\xc4\x8f\xd6\xf9\x82\x92\xda3\xb1\x1cHx\xb3m\xde\x1b\0\0\0\0\0\0\0\0\x18\x01\0\0\0\0\0\0\0\xc4\x8f\xd6\xf9\x82\x92\xda3\xb1\x1cHx\xb3m\xde\x1b\0\0\0\0\0\0\0\0-\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x05Event\x14EventHandleGenerator\0\x18\x02\0\0\0\0\0\0\0\xc4\x8f\xd6\xf9\x82\x92\xda3\xb1\x1cHx\xb3m\xde\x1b.\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x0fAccountFreezing\x0bFreezingBit\0\x01\0@\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x0bDiemAccount\x07Balance\x01\x07\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x03GAS\x03GAS\0\x08@B\x0f\0\0\0\0\0";

    let (_h, b): (HashValue, AccountStateBlob) = bcs::from_bytes(bytes).expect("cant decode");

    let acc_state: AccountStateV5 = bcs::from_bytes(&b.blob).unwrap();
    let ar = acc_state.get_account_resource().unwrap();
    dbg!(&ar);

    let s = acc_state.get_resource::<FreezingBit>().unwrap();
    dbg!(&s);

    let _balance_key = hex::decode("01000000000000000000000000000000010b4469656d4163636f756e740742616c616e636501070000000000000000000000000000000000000000000000000000000000000001034741530347415300").unwrap();
    let balance_key = hex::decode("01000000000000000000000000000000010b4469656d4163636f756e740742616c616e6365010700000000000000000000000000000001034741530347415300").unwrap();
    let b = acc_state.0.get(&balance_key).unwrap();
    dbg!(&b);
    let res: BalanceResource = bcs::from_bytes(b).unwrap();
    dbg!(&res);
}

#[test]
fn sanity_test() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Ip([u8; 4]);

    #[derive(Debug, Deserialize, Serialize)]
    struct Port(u16);

    #[derive(Debug, Deserialize, Serialize)]
    struct SocketAddr {
        ip: Ip,
        port: Port,
    }

    let s = SocketAddr {
        ip: Ip([127, 0, 0, 1]),
        port: Port(8001),
    };
    let b = bcs::to_bytes(&s).unwrap();
    dbg!(&b);
    let expected_bytes = vec![0x7f, 0x00, 0x00, 0x01, 0x41, 0x1f];

    assert!(b == expected_bytes, "not match");

    let socket_addr: SocketAddr = bcs::from_bytes(&expected_bytes).unwrap();

    dbg!(&socket_addr);

    assert_eq!(socket_addr.ip.0, [127, 0, 0, 1]);
    assert_eq!(socket_addr.port.0, 8001);
}

#[test]
fn decode_encode_struct_tag() {
    use super::language_storage_v5::StructTagV5;
    use super::legacy_address_v5::LEGACY_CORE_CODE_ADDRESS;
    use move_core_types::ident_str;
    let s = StructTagV5 {
        address: LEGACY_CORE_CODE_ADDRESS,
        module: ident_str!("DiemAccount").into(),
        name: ident_str!("DiemAccount").into(),
        type_params: vec![],
    };

    let bytes = bcs::to_bytes(&s).unwrap();
    let h = hex::encode(bytes);
    dbg!(&h);

    let expected_key =
        "000000000000000000000000000000010b4469656d4163636f756e740b4469656d4163636f756e7400";

    assert!(expected_key == h);

    let patch_expected_key =
        "01000000000000000000000000000000010b4469656d4163636f756e740b4469656d4163636f756e7400";
    // lookup will fail unless it's using the access_vector() bit
    // "resource keys" have prepended 01
    assert!(hex::encode(s.access_vector()) == patch_expected_key);
}
