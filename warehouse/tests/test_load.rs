use std::path::PathBuf;

use libra_types::exports::AccountAddress;
use libra_warehouse::extract::extract_v5_snapshot;
use libra_warehouse::table_structs::{
    WarehouseAccount, WarehouseBalance, WarehouseRecord, WarehouseTime,
};

use sqlx::SqlitePool;

fn v5_state_manifest_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = p.parent().unwrap();
    project_root.join("compatibility/fixtures/v5/state_ver_119757649.17a8/state.manifest")
}

#[sqlx::test]
async fn insert_one_account(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
    let marlon = AccountAddress::random();
    let acc = WarehouseAccount { address: marlon };

    let res = libra_warehouse::load_account::insert_one_account(&pool, &acc).await?;
    assert!(res.rows_affected() == 1);

    // second time should error if we are using the same account
    assert!(
        libra_warehouse::load_account::insert_one_account(&pool, &acc)
            .await
            .is_err()
    );

    Ok(())
}

#[sqlx::test]
async fn batch_insert_account(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
    let mut vec_acct: Vec<WarehouseRecord> = vec![];

    for _i in 0..3 {
        // each will be a unique account
        let acc = WarehouseRecord::new(AccountAddress::random());
        vec_acct.push(acc);
    }

    let res = libra_warehouse::load_account::impl_batch_insert(&pool, &vec_acct).await?;
    assert!(res.rows_affected() == 3);

    Ok(())
}

#[sqlx::test]
async fn batch_duplicates_fail_gracefully(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
    let mut vec_acct: Vec<WarehouseRecord> = vec![];

    // will create duplicates
    let marlon = AccountAddress::random();

    for _i in 0..3 {
        let acc = WarehouseRecord::new(marlon);
        vec_acct.push(acc);
    }

    // should not fail if duplicates exists on same batch
    let res = libra_warehouse::load_account::impl_batch_insert(&pool, &vec_acct).await?;
    assert!(res.rows_affected() == 1);
    // also should not fail if duplicates are on separate batches
    let res = libra_warehouse::load_account::impl_batch_insert(&pool, &vec_acct).await?;
    assert!(res.rows_affected() == 0);

    Ok(())
}

#[sqlx::test]

async fn test_e2e_load_v5_snapshot(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;

    let manifest_file = v5_state_manifest_fixtures_path();
    assert!(manifest_file.exists());
    let wa_vec = extract_v5_snapshot(&manifest_file).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(wa_vec.len() == 17338);

    let res = libra_warehouse::load_account::batch_insert_account(&pool, &wa_vec, 1000).await?;

    assert!(res == 17338);
    Ok(())
}

#[sqlx::test]
async fn batch_insert_coin(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
    let mut vec_state: Vec<WarehouseRecord> = vec![];

    for _i in 0..3 {
        let state = WarehouseRecord {
            account: WarehouseAccount {
                // uniques
                address: AccountAddress::random(),
            },
            time: WarehouseTime::default(),
            balance: Some(WarehouseBalance {
                balance: 0,
                legacy_balance: Some(10),
            }),
        };

        vec_state.push(state);
    }

    // fist must load accounts
    let res = libra_warehouse::load_account::load_account_state(&pool, &vec_state).await?;

    assert!(res == 3);

    let res = libra_warehouse::load_coin::impl_batch_coin_insert(&pool, &vec_state).await?;

    assert!(res.rows_affected() == 3);

    Ok(())
}

// The table should not update if the balance remains the same.
// new records are only inserted when the balance changes.
#[sqlx::test]
async fn increment_coin_noop(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
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
                balance: 0,
                // same balance
                legacy_balance: Some(10),
            }),
        };

        vec_state.push(state);
    }

    // fist must load accounts
    let _res = libra_warehouse::load_account::batch_insert_account(&pool, &vec_state, 10).await?;

    // since the balance never changed but the times changed, there are no updates to the table.
    let res = libra_warehouse::load_coin::alt_increment_one_balance(&pool, &vec_state[0]).await?;
    assert!(res.rows_affected() == 1);

    let res = libra_warehouse::load_coin::alt_increment_one_balance(&pool, &vec_state[1]).await?;
    assert!(res.rows_affected() == 0);

    let res = libra_warehouse::load_coin::alt_increment_one_balance(&pool, &vec_state[2]).await?;
    assert!(res.rows_affected() == 0);

    let res = libra_warehouse::query_balance::query_last_balance(&pool, marlon).await?;

    assert!(res.balance == 10);

    Ok(())
}

// Increment the balance table when there balance changes.
#[sqlx::test]
async fn increment_coin(pool: SqlitePool) -> anyhow::Result<()> {
    libra_warehouse::migrate::maybe_init(&pool).await?;
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
                balance: 0,
                // different balance each time
                legacy_balance: Some(10 * i),
            }),
        };

        vec_state.push(state);
    }

    // fist must load accounts
    let _res = libra_warehouse::load_account::batch_insert_account(&pool, &vec_state, 10).await?;

    // the balance CHANGES, so each increment will create a new record
    let res = libra_warehouse::load_coin::alt_increment_one_balance(&pool, &vec_state[0]).await?;
    assert!(res.rows_affected() == 1);

    let res = libra_warehouse::query_balance::query_last_balance(&pool, marlon).await?;
    assert!(res.balance == 0); // 10 * 0th

    let res = libra_warehouse::load_coin::alt_increment_one_balance(&pool, &vec_state[1]).await?;
    assert!(res.rows_affected() == 1);

    let res = libra_warehouse::load_coin::alt_increment_one_balance(&pool, &vec_state[2]).await?;
    assert!(res.rows_affected() == 1);

    let res = libra_warehouse::query_balance::query_last_balance(&pool, marlon).await?;
    assert!(res.balance == 20);

    Ok(())
}
