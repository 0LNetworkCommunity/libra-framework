// Do a full restoration given a RestoreBundle with verified manifests

use std::path::Path;

use crate::{
    dbtool_init::{run_restore, RestoreTypes},
    restore_bundle::RestoreBundle,
};

pub async fn full_restore(target_db: &Path, bundle: &RestoreBundle) -> anyhow::Result<()> {
    assert!(
        bundle.is_loaded(),
        "the restore bundle hasn't been checked yet"
    );

    run_restore(RestoreTypes::Epoch, target_db.to_owned(), bundle).await?;
    run_restore(RestoreTypes::Snapshot, target_db.to_owned(), bundle).await?;
    run_restore(RestoreTypes::Transaction, target_db.to_owned(), bundle).await?;
    Ok(())
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
