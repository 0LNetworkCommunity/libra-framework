use std::path::PathBuf;

use libra_types::exports::AccountAddress;
use libra_warehouse::table_structs::WarehouseAccount;
use libra_warehouse::extract::extract_v5_snapshot;

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
    let acc = WarehouseAccount {
      address: marlon
    };

    libra_warehouse::load::insert_one_account(&pool, &acc).await?;

    // second time should error if we are using the same account
    assert!(libra_warehouse::load::insert_one_account(&pool, &acc).await.is_err());

    Ok(())
}

#[sqlx::test]
async fn batch_insert(pool: SqlitePool) -> anyhow::Result<()>{
  libra_warehouse::migrate::maybe_init(&pool).await?;
    let mut vec_acct: Vec<WarehouseAccount> = vec![];

    for _i in [..3] {
      let acc = WarehouseAccount {
        // uniques
        address: AccountAddress::random()
      };
      vec_acct.push(acc);
    }

  libra_warehouse::load::commit_batch_query(&pool, &vec_acct).await?;
  Ok(())
}

#[sqlx::test]
async fn batch_duplicates_fail_gracefully(pool: SqlitePool) -> anyhow::Result<()>{
  libra_warehouse::migrate::maybe_init(&pool).await?;
    let mut vec_acct: Vec<WarehouseAccount> = vec![];

    // will create duplicates
    let marlon = AccountAddress::random();

    for _i in [..3] {
      let acc = WarehouseAccount {
        address: marlon
      };
      vec_acct.push(acc);
    }

  libra_warehouse::load::commit_batch_query(&pool, &vec_acct).await?;
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

    let res = libra_warehouse::load::load_account_state(&pool, &wa_vec).await?;

    assert!(res == 17338);
    Ok(())
}
