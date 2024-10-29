use crate::table_structs::WarehouseRecord;
use anyhow::Result;
use sqlx::{postgres::PgQueryResult, PgPool, Postgres, QueryBuilder};

pub async fn batch_insert_account(
    pool: &PgPool,
    acc: &[WarehouseRecord],
    batch_len: usize,
) -> Result<()> {
    let chunks: Vec<&[WarehouseRecord]> = acc.chunks(batch_len).collect();

    for c in chunks {
        impl_batch_coin_insert(pool, c).await?;
    }

    Ok(())
}

// TODO: return specific commit errors for this batch
pub async fn impl_batch_coin_insert(
    pool: &PgPool,
    batch_accounts: &[WarehouseRecord],
) -> Result<PgQueryResult> {
    let filtered = batch_accounts.iter().filter(|el| el.balance.is_some());

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
      INSERT INTO balance (account_address, balance, chain_timestamp, db_version, epoch_number)
      "#,
    );

    query_builder.push_values(filtered, |mut b, acc| {
        let this_account = acc.account.address.to_hex_literal();
        let this_balance = acc.balance.as_ref().unwrap().balance as i64;
        let this_timestamp = acc.time.timestamp as i64;
        b.push_bind(this_account)
            .push_bind(this_balance)
            .push_bind(this_timestamp) // todo
            .push_bind(acc.time.version as i64) // todo
            .push_bind(acc.time.epoch as i64); // todo;
    });

    let query = query_builder.build();
    let res = query.execute(pool).await?;

    Ok(res)
}

pub async fn alt_increment_one_balance(
    pool: &PgPool,
    record: &WarehouseRecord,
) -> Result<PgQueryResult> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new(increment_balance_template(record));
    let query = query_builder.build();
    let res = query.execute(pool).await?;

    Ok(res)
}

fn increment_balance_template(record: &WarehouseRecord) -> String {
    let this_account = record.account.address.to_hex_literal();
    let this_balance = record.balance.as_ref().unwrap().balance as i64;
    let this_timestamp = record.time.timestamp as i64;
    let this_version = record.time.version as i64;
    let this_epoch = record.time.epoch as i64;

    let query_template = format!(
        r#"
  INSERT INTO balance (account_address, balance, chain_timestamp, db_version, epoch_number)
  SELECT '{this_account}', {this_balance}, {this_timestamp}, {this_version}, {this_epoch}
  WHERE NOT EXISTS (
      SELECT 1
      FROM balance
      WHERE account_address = '{this_account}'
      AND balance = {this_balance}
      ORDER BY chain_timestamp DESC
      LIMIT 1
  );"#
    );

    query_template
}

#[test]
fn test_format() {
    use crate::table_structs::{WarehouseAccount, WarehouseBalance, WarehouseTime};
    use libra_types::exports::AccountAddress;

    let record = WarehouseRecord {
        account: WarehouseAccount {
            // uniques
            address: AccountAddress::random(),
        },
        time: WarehouseTime::default(),
        balance: Some(WarehouseBalance {
            balance: 10,
            // legacy_balance: Some(10),
        }),
    };
    let s = increment_balance_template(&record);
    dbg!(&s);
}
