mod experimental;
mod support;

use experimental::restaurant;
use support::fixtures;
use support::pg_testcontainer::get_test_pool;

use libra_warehouse::extract_snapshot::{extract_current_snapshot, extract_v5_snapshot};

#[tokio::test]
async fn test_e2e_load_v5_snapshot() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;

    let manifest_file = fixtures::v5_state_manifest_fixtures_path().join("state.manifest");
    assert!(manifest_file.exists());
    let wa_vec = extract_v5_snapshot(&manifest_file).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(wa_vec.len() == 17338);

    let res = experimental::load_account::batch_insert_account(&pool, &wa_vec, 1000).await?;

    assert!(res == 17338);
    Ok(())
}

#[tokio::test]
async fn test_e2e_load_v7_snapshot() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;

    let archive_dir = fixtures::v7_state_manifest_fixtures_path();
    let wa_vec = extract_current_snapshot(&archive_dir).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(wa_vec.len() == 24607);

    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let res = experimental::load_account::batch_insert_account(&pool, &wa_vec, 1000).await?;

    assert!(res == 24607);
    Ok(())
}

#[tokio::test]
async fn test_e2e_load_v7_snapshot_on_age_graph() -> anyhow::Result<()> {
    // USING APACHE/AGE
    let (pool, _c) = crate::support::age_testcontainer::get_test_age_pool().await?;

    let archive_dir = fixtures::v7_state_manifest_fixtures_path();
    let wa_vec = extract_current_snapshot(&archive_dir).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(wa_vec.len() == 24607);

    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let res = experimental::load_account::batch_insert_account(&pool, &wa_vec, 1000).await?;

    assert!(res == 24607);
    Ok(())
}

#[tokio::test]
async fn test_ingest_two_v6_snapshot_archives() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;

    // V6 archive should have two State Snapshot archives
    let archive_dir = fixtures::v6_tx_manifest_fixtures_path();

    let count_processed = restaurant::sushi_train(archive_dir.parent().unwrap(), &pool).await?;
    assert!(count_processed == 2);

    Ok(())
}
