// Do a full restoration given a RestoreBundle with verified manifests

use std::path::Path;

use crate::{init_hack, restore_bundle::RestoreBundle};

pub async fn full_restore(target_db: &Path, bundle: &RestoreBundle) -> anyhow::Result<()> {
    assert!(bundle.is_loaded(), "the restore bundle hasn't been checked yet");

    // restore epoch
    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Epoch,
        target_db,
        &bundle.restore_bundle_dir,
        &bundle.epoch_manifest,
        bundle.version,
    )?;

    s.run().await?;

    // restore snapshot
    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Snapshot,
        target_db,
        &bundle.restore_bundle_dir,
        &bundle.snapshot_manifest,
        bundle.version,
    )?;

    s.run().await?;

    // restore transactions
    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Transaction,
        target_db,
        &bundle.restore_bundle_dir,
        &bundle.transaction_manifest,
        bundle.version,
    )?;

    s.run().await
}

#[tokio::test]
async fn test_full_restore() {
    use std::path::PathBuf;
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut b = RestoreBundle::new(dir.join("fixtures/v7"));
    b.load().unwrap();
    let db_temp = diem_temppath::TempPath::new();
    db_temp.create_as_dir().unwrap();
    full_restore(db_temp.path(), &b).await.unwrap();
}
