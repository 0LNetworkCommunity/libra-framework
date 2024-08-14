// Do a full restoration given a RestoreBundle with verified manifests

use std::path::Path;

use crate::{
    dbtool_init::{run_restore, RestoreTypes},
    restore_bundle::RestoreBundle,
};

pub async fn full_restore(db_destination: &Path, bundle: &RestoreBundle) -> anyhow::Result<()> {
    assert!(
        bundle.is_loaded(),
        "the restore bundle hasn't been checked yet"
    );

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
