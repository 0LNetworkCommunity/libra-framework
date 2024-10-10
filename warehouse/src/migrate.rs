use sqlx::SqlitePool;

pub async fn maybe_init(pool: &SqlitePool) -> anyhow::Result<()> {
    // run through all the migrations in ./sql/migrations.
    // note: the path is relative to project root
    // and the naming of the files follow a pattern, see migrations/README.md
    sqlx::migrate!("sql/migrations").run(pool).await?;
    Ok(())
}
