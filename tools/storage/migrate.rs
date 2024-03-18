

/// take an archive file path and parse into a writeset
pub async fn archive_into_recovery(
    archive_path: &PathBuf,
    is_legacy: bool,
) -> Result<Vec<LegacyRecovery>, Error> {
    let manifest_json = archive_path.join("state.manifest");

    let backup = read_snapshot::read_from_json(&manifest_json)?;

    let account_blobs = accounts_from_snapshot_backup(backup, archive_path).await?;
    let r = if is_legacy {
        println!("Parsing account state from legacy, Libra structs");
        todo!();
    } else {
        println!("Parsing account state from Diem structs");
        accounts_into_recovery(&account_blobs)?
    };

    Ok(r)
}

/// Tokio async parsing of state snapshot into blob
async fn accounts_from_snapshot_backup(
    manifest: StateSnapshotBackup,
    archive_path: &PathBuf,
) -> Result<Vec<AccountStateBlob>, Error> {
    // parse AccountStateBlob from chunks of the archive
    let mut account_state_blobs: Vec<AccountStateBlob> = Vec::new();
    for chunk in manifest.chunks {
        // dbg!(&archive_path);
        let blobs = read_snapshot::read_account_state_chunk(chunk.blobs, archive_path).await?;
        // println!("{:?}", blobs);
        for (_key, blob) in blobs {
            account_state_blobs.push(blob)
        }
    }

    Ok(account_state_blobs)
}
