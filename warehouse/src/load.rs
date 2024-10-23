use crate::table_structs::{WarehouseAccount, WarehouseState};
use anyhow::Result;
use sqlx::{sqlite::SqliteQueryResult, SqlitePool};

pub async fn load_account_state(pool: &SqlitePool, accounts: &Vec<WarehouseState>) -> Result<i64> {
    let mut rows = 0;
    // insert missing accounts
    for ws in accounts.iter() {
        let res= insert_one_account(pool, &ws.account).await?;
        rows = res.last_insert_rowid();
    }

    // increment the balance changes
    Ok(rows)
}

pub async fn insert_one_account(pool: &SqlitePool, acc: &WarehouseAccount) -> Result<SqliteQueryResult> {

    let res = sqlx::query(r#"
      INSERT INTO users (account_address, is_legacy)
      VALUES ($1,$2)
    "#)
    .bind(acc.address.to_string())
    .bind(true)
    .execute(pool)
    .await?;

    Ok(res)
}
