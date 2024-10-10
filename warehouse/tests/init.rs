
use sqlx::{PgPool};

#[sqlx::test]
async fn can_init(pool: PgPool) -> anyhow::Result<()>{
    libra_warehouse::migrate::maybe_init(&pool).await?;

    Ok(())
}
