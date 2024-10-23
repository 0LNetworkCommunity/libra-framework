use libra_types::exports::AccountAddress;
use libra_warehouse::table_structs::WarehouseAccount;
use sqlx::SqlitePool;

#[sqlx::test]
async fn insert_one_account(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
    let marlon = AccountAddress::random();
    let acc = WarehouseAccount {
      address: marlon
    };

    libra_warehouse::load::insert_one_account(&pool, &acc).await?;

    // second time should error if we are using the same account
    assert!(libra_warehouse::load::insert_one_account(&pool, &acc).await.is_err());

    Ok(())
}
