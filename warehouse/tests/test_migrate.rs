use sqlx::SqlitePool;

#[sqlx::test]
async fn can_init(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
    let mut conn = pool.acquire().await?;

    let id = sqlx::query(
        r#"
      INSERT INTO users (account_address, is_legacy)
      VALUES ("00000000000000000000000000000000e8953084617dd5c6071cf2918215e183", TRUE)
      "#,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    assert!(id == 1);


    let id = sqlx::query(
        r#"
      INSERT INTO balance (account_address, balance, chain_timestamp, db_version, epoch_number)
      VALUES ("00000000000000000000000000000000e8953084617dd5c6071cf2918215e183", 11, 22222222, 600, 1)
      "#,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    assert!(id == 1);
    Ok(())
}