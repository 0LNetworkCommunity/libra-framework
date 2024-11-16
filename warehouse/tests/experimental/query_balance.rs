use anyhow::Result;
use libra_types::exports::AccountAddress;
use sqlx::PgPool;

use libra_warehouse::table_structs::WarehouseBalance;

// TODO: return specific commit errors for this batch
pub async fn query_last_balance(
    pool: &PgPool,
    account: AccountAddress,
) -> Result<WarehouseBalance> {
    let account_address = account.to_hex_literal();

    let query_template = format!(
        r#"
        SELECT balance, chain_timestamp, db_version, epoch_number
        FROM balance
        WHERE account_address = '{account_address}'
        ORDER BY chain_timestamp DESC
        LIMIT 1;
        "#
    );

    let row = sqlx::query(&query_template).fetch_one(pool).await?;
    dbg!(&row);

    let dummy = WarehouseBalance { balance: 0 };

    Ok(dummy)
}
