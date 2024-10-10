use sqlx::PgPool;

pub async fn maybe_init(pool: &PgPool) -> anyhow::Result<()> {
    // run through all the migrations in ./sql/migrations.
    // note: the path is relative to project root
    // and the naming of the files follow a pattern, see migrations/README.md
    sqlx::migrate!("tests/mock_migrations").run(pool).await?;
    Ok(())
}
