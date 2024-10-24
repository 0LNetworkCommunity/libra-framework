use crate::table_structs::{WarehouseAccount, WarehouseState};
use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, QueryBuilder, Sqlite, SqlitePool};

pub async fn load_account_state(pool: &SqlitePool, accounts: &Vec<WarehouseState>) -> Result<i64> {
    let mut rows = 0;
    // insert missing accounts
    for ws in accounts.iter() {
        let res = insert_one_account(pool, &ws.account).await?;
        rows = res.last_insert_rowid();
    }

    // increment the balance changes
    Ok(rows)
}

pub async fn insert_one_account(
    pool: &SqlitePool,
    acc: &WarehouseAccount,
) -> Result<SqliteQueryResult> {
    let res = sqlx::query(
        r#"
      INSERT INTO users (account_address, is_legacy)
      VALUES ($1,$2)
    "#,
    )
    .bind(acc.address.to_string())
    .bind(true)
    .execute(pool)
    .await?;

    Ok(res)
}

pub async fn batch_insert_account(
    pool: &SqlitePool,
    acc: &[WarehouseAccount],
    batch_len: usize,
) -> Result<()> {
    let chunks: Vec<&[WarehouseAccount]> = acc.chunks(batch_len).collect();

    for c in chunks {
        commit_batch_query(pool, c).await?;
    }

    Ok(())
}

// TODO: return specific commit errors for this batch
pub async fn commit_batch_query(pool: &SqlitePool, batch_accounts: &[WarehouseAccount]) -> Result<()> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
        "INSERT INTO users (account_address, is_legacy) ",
    );

    query_builder.push_values(batch_accounts, |mut b, acc| {
        b.push_bind(acc.address.to_hex_literal()).push_bind(true);
    });

    // makes sure the txs don't fail on repeated attempts to add users
    query_builder.push("ON CONFLICT (account_address) DO NOTHING");

    let query = query_builder.build();
    let _res = query.execute(pool).await?;

    Ok(())
}
