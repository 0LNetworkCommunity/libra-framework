mod experimental;
mod support;

use support::pg_testcontainer::get_test_pool;

use libra_types::exports::AccountAddress;
use libra_warehouse::table_structs::{
    WarehouseAccount, WarehouseBalance, WarehouseRecord, WarehouseTime,
};

#[tokio::test]
async fn insert_one_account() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let marlon = AccountAddress::random();
    let acc = WarehouseAccount { address: marlon };

    let res = experimental::load_account::insert_one_account(&pool, &acc).await?;
    assert!(res.rows_affected() == 1);

    // second time should error if we are using the same account
    assert!(experimental::load_account::insert_one_account(&pool, &acc)
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn batch_insert_account() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let mut vec_acct: Vec<WarehouseRecord> = vec![];

    for _i in 0..3 {
        // each will be a unique account
        let acc = WarehouseRecord::new(AccountAddress::random());
        vec_acct.push(acc);
    }

    let res = experimental::load_account::impl_batch_insert_pg(&pool, &vec_acct).await?;
    assert!(res.rows_affected() == 3);

    Ok(())
}

#[tokio::test]
async fn batch_duplicates_fail_gracefully() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let mut vec_acct: Vec<WarehouseRecord> = vec![];

    // will create duplicates
    let marlon = AccountAddress::random();

    for _i in 0..3 {
        let acc = WarehouseRecord::new(marlon);
        vec_acct.push(acc);
    }

    // should not fail if duplicates exists on same batch
    let res = experimental::load_account::impl_batch_insert_pg(&pool, &vec_acct).await?;
    assert!(res.rows_affected() == 1);
    // also should not fail if duplicates are on separate batches
    let res = experimental::load_account::impl_batch_insert_pg(&pool, &vec_acct).await?;
    assert!(res.rows_affected() == 0);

    Ok(())
}

#[tokio::test]
async fn batch_insert_coin() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;

    let mut vec_state: Vec<WarehouseRecord> = vec![];

    for _i in 0..3 {
        let state = WarehouseRecord {
            account: WarehouseAccount {
                // uniques
                address: AccountAddress::random(),
            },
            time: WarehouseTime::default(),
            balance: Some(WarehouseBalance { balance: 10 }),
        };

        vec_state.push(state);
    }

    // fist must load accounts
    let res = experimental::load_account::load_account_state_depr(&pool, &vec_state).await?;
    assert!(res == 3);

    let res = experimental::load_coin::impl_batch_coin_insert(&pool, &vec_state).await?;
    dbg!(&res);
    assert!(res.rows_affected() == 3);

    Ok(())
}

// The table should not update if the balance remains the same.
// new records are only inserted when the balance changes.
#[tokio::test]
async fn increment_coin_noop() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let mut vec_state: Vec<WarehouseRecord> = vec![];
    let marlon = AccountAddress::random();
    // same user, and same balance, but incremental timestamps
    for i in 0..3 {
        let state = WarehouseRecord {
            account: WarehouseAccount {
                // same user
                address: marlon,
            },
            time: WarehouseTime {
                timestamp: 12 * i,
                version: 2 * i,
                epoch: 3 * i,
            },
            balance: Some(WarehouseBalance {
                balance: 10,
                // same balance
                // legacy_balance: Some(10),
            }),
        };

        vec_state.push(state);
    }

    // fist must load accounts
    let _res = experimental::load_account::batch_insert_account(&pool, &vec_state, 10).await?;

    // since the balance never changed but the times changed, there are no updates to the table.
    let res = experimental::load_coin::alt_increment_one_balance(&pool, &vec_state[0]).await?;
    assert!(res.rows_affected() == 1);

    let res = experimental::load_coin::alt_increment_one_balance(&pool, &vec_state[1]).await?;
    assert!(res.rows_affected() == 0);

    let res = experimental::load_coin::alt_increment_one_balance(&pool, &vec_state[2]).await?;
    assert!(res.rows_affected() == 0);

    let res = experimental::query_balance::query_last_balance(&pool, marlon).await?;

    assert!(res.balance == 10);

    Ok(())
}

// Only increment the balance table when their balance changes.
#[tokio::test]
async fn increment_coin() -> anyhow::Result<()> {
    let (pool, _c) = get_test_pool().await?;
    experimental::pg_migrate::maybe_init_pg(&pool).await?;
    let mut vec_state: Vec<WarehouseRecord> = vec![];
    let marlon = AccountAddress::random();
    // same user, and same balance, but incremental timestamps
    for i in 0..3 {
        let state = WarehouseRecord {
            account: WarehouseAccount {
                // same user
                address: marlon,
            },
            time: WarehouseTime {
                timestamp: 12 * i,
                version: 2 * i,
                epoch: 3 * i,
            },
            balance: Some(WarehouseBalance { balance: 10 * i }),
        };

        vec_state.push(state);
    }

    // fist must load accounts
    let _res = experimental::load_account::batch_insert_account(&pool, &vec_state, 10).await?;

    // the balance CHANGES, so each increment will create a new record
    let res = experimental::load_coin::alt_increment_one_balance(&pool, &vec_state[0]).await?;
    assert!(res.rows_affected() == 1);

    let res = experimental::query_balance::query_last_balance(&pool, marlon).await?;
    assert!(res.balance == 0); // 10 * 0th

    let res = experimental::load_coin::alt_increment_one_balance(&pool, &vec_state[1]).await?;
    assert!(res.rows_affected() == 1);

    let res = experimental::load_coin::alt_increment_one_balance(&pool, &vec_state[2]).await?;
    assert!(res.rows_affected() == 1);

    let res = experimental::query_balance::query_last_balance(&pool, marlon).await?;
    assert!(res.balance == 20);

    Ok(())
}
