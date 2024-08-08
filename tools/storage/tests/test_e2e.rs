use clap::Parser;
use std::path::PathBuf;

use diem_temppath::TempPath;
use libra_storage::storage_cli::StorageCli;

#[tokio::test]
async fn e2e_epoch() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = dir.join("fixtures/v7");
    let db_temp = TempPath::new();
    db_temp.create_as_dir()?;

    let cmd = format!("storage db restore oneoff epoch-ending --epoch-ending-manifest {manifest} --target-db-dir {db} --local-fs-dir {fs}",
    manifest = fixtures.join("epoch_ending_116-.be9b/epoch_ending.manifest").display(),
    db = db_temp.path().display(),
    fs = fixtures.display()
  );

    let to_vec: Vec<_> = cmd.split_whitespace().collect();
    let s = StorageCli::try_parse_from(to_vec)?;
    s.run().await
}

#[tokio::test]
async fn e2e_snapshot() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = dir.join("fixtures/v7");
    let db_temp = TempPath::new();
    db_temp.create_as_dir()?;

    let cmd = format!("storage db restore oneoff state-snapshot --state-manifest {manifest} --target-db-dir {db} --local-fs-dir {fs} --restore-mode default --state-into-version 1",
    manifest = fixtures.join("state_epoch_116_ver_38180075.05af/state.manifest").display(),
    db = db_temp.path().display(),
    fs = fixtures.display()
  );

    let to_vec: Vec<_> = cmd.split_whitespace().collect();
    let s = StorageCli::try_parse_from(to_vec)?;
    s.run().await
}
