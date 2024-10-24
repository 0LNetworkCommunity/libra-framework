use crate::table_structs::WarehouseState;
use anyhow::Result;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};


pub async fn batch_insert_account(
    pool: &SqlitePool,
    acc: &[WarehouseState],
    batch_len: usize,
) -> Result<()> {
    let chunks: Vec<&[WarehouseState]> = acc.chunks(batch_len).collect();

    for c in chunks {
        impl_batch_coin_insert(pool, c).await?;
    }

    Ok(())
}

// TODO: return specific commit errors for this batch
pub async fn impl_batch_coin_insert(pool: &SqlitePool, batch_accounts: &[WarehouseState]) -> Result<()> {
    let filtered = batch_accounts.iter().filter(|el| {
      el.balance.is_some()
    });

    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
      r#"
      INSERT INTO balance (account_address, balance, chain_timestamp, db_version, epoch_number)
      "#,
    );

    query_builder.push_values(filtered, |mut b, acc| {
        b.push_bind(acc.account.address.to_hex_literal()).push_bind(true)
        .push_bind(acc.balance.as_ref().unwrap().legacy_balance.unwrap() as i64).push_bind(true)
        .push_bind(0) // todo
        .push_bind(0) // todo
        .push_bind(0); // todo
    });

    let query = query_builder.build();
    let _res = query.execute(pool).await?;

    Ok(())
}
