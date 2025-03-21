// Do a full restoration given a RestoreBundle with verified manifests

use std::path::Path;
use std::fs::File;
use flate2::read::GzDecoder;
use std::io::copy;
use diem_temppath::TempPath;
use anyhow::Result;

use crate::{
    dbtool_init::{run_restore, RestoreTypes},
    restore_bundle::RestoreBundle,
};

async fn decompress_if_needed(source_path: &Path) -> Result<Option<TempPath>> {
    if source_path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let temp = TempPath::new();
        temp.create_as_file()?;

        let gz_file = File::open(source_path)?;
        let mut decoder = GzDecoder::new(gz_file);
        let mut outfile = File::create(temp.path())?;

        copy(&mut decoder, &mut outfile)?;

        Ok(Some(temp))
    } else {
        Ok(None)
    }
}

pub async fn full_restore(db_destination: &Path, bundle: &RestoreBundle) -> Result<()> {
    assert!(
        bundle.is_loaded(),
        "the restore bundle hasn't been checked yet"
    );

    // Create temporary paths that will be cleaned up when dropped
    let mut temp_paths = Vec::new();

    // Get manifest paths from bundle
    let manifest_paths = [
        &bundle.epoch_manifest,
        &bundle.snapshot_manifest,
        &bundle.transaction_manifest,
    ];

    // Process each manifest file
    for manifest_path in manifest_paths {
        let full_path = bundle.restore_bundle_dir.join(manifest_path);
        if let Some(temp_path) = decompress_if_needed(&full_path).await? {
            temp_paths.push(temp_path);
        }
    }

    // Run restores in sequence
    run_restore(RestoreTypes::Epoch, db_destination, bundle).await?;
    run_restore(RestoreTypes::Snapshot, db_destination, bundle).await?;
    run_restore(RestoreTypes::Transaction, db_destination, bundle).await?;

    Ok(())
}

#[tokio::test]
async fn test_full_restore() -> anyhow::Result<()> {
    use std::path::PathBuf;
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut b = RestoreBundle::new(dir.join("fixtures/v7"));
    b.load().unwrap();
    let mut db_temp = diem_temppath::TempPath::new();
    db_temp.persist();
    db_temp.create_as_dir()?;

    full_restore(db_temp.path(), &b).await?;

    assert!(db_temp.path().join("ledger_db").exists());
    assert!(db_temp.path().join("state_merkle_db").exists());
    Ok(())
}

// Add test for gz decompression
#[tokio::test]
async fn test_decompress_if_needed() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gz_file = dir.join("fixtures/v7/epoch_ending.json.gz");

    if let Some(temp_path) = decompress_if_needed(&gz_file).await? {
        assert!(temp_path.path().exists());
        assert_ne!(
            std::fs::metadata(&gz_file)?.len(),
            std::fs::metadata(temp_path.path())?.len()
        );
    }

    Ok(())
}
