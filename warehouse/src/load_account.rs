use crate::table_structs::{WarehouseAccount, WarehouseRecord};
use anyhow::Result;
use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder};

pub async fn insert_one_account(pool: &PgPool, acc: &WarehouseAccount) -> Result<PgQueryResult> {
    let res = sqlx::query(
        r#"
      INSERT INTO users (account_address, is_legacy)
      VALUES ($1,$2)
    "#,
    )
    .bind(acc.address.to_hex_literal())
    .bind(true)
    .execute(pool)
    .await?;

    Ok(res)
}

pub async fn batch_insert_account(
    pool: &PgPool,
    acc: &[WarehouseRecord],
    batch_len: usize,
) -> Result<u64> {
    let chunks: Vec<&[WarehouseRecord]> = acc.chunks(batch_len).collect();
    let mut rows = 0;
    for c in chunks {
        let res = impl_batch_insert_pg(pool, c).await?;
        rows += res.rows_affected();
    }

    Ok(rows)
}

// // TODO: return specific commit errors for this batch
// pub async fn impl_batch_insert(
//     pool: &PgPool,
//     batch_accounts: &[WarehouseRecord],
// ) -> Result<PgQueryResult> {
//     let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
//         // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
//         "INSERT INTO users (account_address, is_legacy) ",
//     );

//     query_builder.push_values(batch_accounts, |mut b, acc| {
//         b.push_bind(acc.account.address.to_hex_literal())
//             .push_bind(true);
//     });

//     // makes sure the txs don't fail on repeated attempts to add users
//     query_builder.push("ON CONFLICT (account_address) DO NOTHING");

//     let query = query_builder.build();
//     let res = query.execute(pool).await?;

//     Ok(res)
// }

// TODO: return specific commit errors for this batch
pub async fn impl_batch_insert_pg(
    pool: &PgPool,
    batch_accounts: &[WarehouseRecord],
) -> Result<PgQueryResult> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
        "INSERT INTO users (account_address, is_legacy) ",
    );

    query_builder.push_values(batch_accounts, |mut b, acc| {
        b.push_bind(acc.account.address.to_hex_literal())
            .push_bind(true);
    });

    // makes sure the txs don't fail on repeated attempts to add users
    query_builder.push("ON CONFLICT (account_address) DO NOTHING");

    let query = query_builder.build();
    let res = query.execute(pool).await?;

    Ok(res)
}

pub async fn load_account_state_depr(pool: &PgPool, accounts: &[WarehouseRecord]) -> Result<u64> {
    let mut rows = 0;
    // insert missing accounts
    for ws in accounts.iter() {
        let res = insert_one_account(pool, &ws.account).await?;
        rows += res.rows_affected();
    }

    // increment the balance changes
    Ok(rows)
}
