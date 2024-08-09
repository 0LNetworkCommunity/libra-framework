use diem_temppath::TempPath;
use libra_storage::init_hack;
use std::path::PathBuf;

// Known from fixtures
const TARGET_VERSION: u64 = 38180075;

#[tokio::test]
async fn e2e_epoch() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let restore_files = dir.join("fixtures/v7");
    let db_temp = TempPath::new();
    db_temp.create_as_dir()?;

    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Epoch,
        db_temp.path(),
        &restore_files,
        "epoch_ending_116-.be9b",
        TARGET_VERSION,
    )?;

    s.run().await
}

#[tokio::test]
async fn e2e_snapshot() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let restore_files = dir.join("fixtures/v7");
    let db_temp = TempPath::new();
    db_temp.create_as_dir()?;

    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Snapshot,
        db_temp.path(),
        &restore_files,
        "state_epoch_116_ver_38180075.05af",
        TARGET_VERSION,
    )?;

    s.run().await
}

#[tokio::test]
async fn e2e_transaction() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let restore_files = dir.join("fixtures/v7");
    let db_temp = TempPath::new();
    db_temp.create_as_dir()?;

    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Transaction,
        db_temp.path(),
        &restore_files,
        "transaction_38100001-.541f",
        TARGET_VERSION,
    )?;

    s.run().await
}

#[tokio::test]
async fn e2e_everybody_sing_along() -> anyhow::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let restore_files = dir.join("fixtures/v7");
    let db_temp = TempPath::new();
    db_temp.create_as_dir()?;

    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Epoch,
        db_temp.path(),
        &restore_files,
        "epoch_ending_116-.be9b",
        TARGET_VERSION,
    )?;

    s.run().await?;

    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Snapshot,
        db_temp.path(),
        &restore_files,
        "transaction_38100001-.541f",
        TARGET_VERSION,
    )?;

    s.run().await?;

    let s = init_hack::hack_dbtool_init(
        init_hack::RestoreTypes::Transaction,
        db_temp.path(),
        &restore_files,
        "transaction_38100001-.541f",
        TARGET_VERSION,
    )?;

    s.run().await
}
