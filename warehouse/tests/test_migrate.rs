mod support;

#[tokio::test]
async fn can_init_pg() -> anyhow::Result<()> {
    let (pool, _c) = support::pg_testcontainer::get_test_pool().await?;
    libra_warehouse::migrate::maybe_init_pg(&pool).await?;
    let mut conn = pool.acquire().await?;

    let id = sqlx::query(
        r#"
      INSERT INTO users (account_address, is_legacy)
      VALUES ('00000000000000000000000000000000e8953084617dd5c6071cf2918215e183', TRUE)
      "#,
    )
    .execute(&mut *conn)
    .await?
    .rows_affected();

    assert!(id == 1);

    let id = sqlx::query(
      r#"
      INSERT INTO balance (account_address, balance, chain_timestamp, db_version, epoch_number)
      VALUES ('00000000000000000000000000000000e8953084617dd5c6071cf2918215e183', 11, 192837564738291845, 600, 1)
      "#,
    )
    .execute(&mut *conn)
    .await?
    .rows_affected();

    assert!(id == 1);
    Ok(())
}
