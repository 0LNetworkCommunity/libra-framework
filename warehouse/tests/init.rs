use sqlx::SqlitePool;

#[sqlx::test]
async fn can_init(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;

    let mut conn = pool.acquire().await?;

    let id = sqlx::query(
        r#"
          INSERT INTO foo (contact_id, first_name)
          VALUES
            (1, "hello");
        "#,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    assert!(id == 1);

    Ok(())
}
