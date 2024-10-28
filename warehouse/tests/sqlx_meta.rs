mod support;
use sqlx::{Pool, Row, Sqlite};

// NOTE: for reference, this is the sqlx test framework runtime, which can setup sqlite dbs. Left here for reference
#[sqlx::test]
async fn sql_insert_test(pool: Pool<Sqlite>) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    let a = sqlx::query("SELECT * FROM foo").execute(&mut *conn).await;

    // every time you run this there will be an empty db
    assert!(a.is_err()); // should error with no table called foo

    let id = sqlx::query(
        r#"
          CREATE TABLE foo (
            contact_id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL
          );
        "#,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    assert!(id == 0);

    let id = sqlx::query(
        r#"
          INSERT INTO foo (contact_id, first_name)
          VALUES
            (1, "hello");
        "#,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    assert!(id == 1);

    let res = sqlx::query(
        r#"
          SELECT * FROM foo
          WHERE contact_id=1
        "#,
    )
    .fetch_one(&mut *conn)
    .await?;

    // get() requires trait Row
    assert!(&res.get::<String, _>("first_name") == "hello");

    Ok(())
}


#[tokio::test]

async fn test_migrate_from_file_pg() -> anyhow::Result<()> {
    let (pool, _c) = support::pg_testcontainer::get_test_pool().await?;
    // The directory must be relative to the project root (the directory containing Cargo.toml), unlike include_str!() which uses compiler internals to get the path of the file where it was invoked.
    sqlx::migrate!("tests/mock_migrations").run(&pool).await?;

    let mut conn = pool.acquire().await?;

    let rows = sqlx::query(
        r#"
          INSERT INTO foo (contact_id, first_name)
          VALUES (1, 'hello');
        "#,
    )
    .execute(&mut *conn)
    .await?
    .rows_affected();

    assert!(rows == 1);

    let a = sqlx::query("SELECT * FROM foo")
        .fetch_all(&mut *conn)
        .await?;

    let q = a.first().unwrap().get_unchecked::<i64, _>(0);
    assert!(q == 1);
    Ok(())
}
