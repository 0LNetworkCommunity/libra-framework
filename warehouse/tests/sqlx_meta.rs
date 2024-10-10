// in cargo.toml...
// sqlx = { version = "0.8", features = [ "runtime-tokio", "tls-native-tls", "sqlite", "migrate"] }

use sqlx::{Pool, Row, Sqlite};

#[sqlx::test]
async fn basic_test(pool: Pool<Sqlite>) -> anyhow::Result<()> {
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

#[sqlx::test]
async fn test_migrate(pool: Pool<Sqlite>) -> anyhow::Result<()>{
    // The directory must be relative to the project root (the directory containing Cargo.toml), unlike include_str!() which uses compiler internals to get the path of the file where it was invoked.
    sqlx::migrate!("tests/mock_migrations")
        .run(&pool)
        .await?;

    let mut conn = pool.acquire().await?;


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

    let a = sqlx::query("SELECT * FROM foo").fetch_all(&mut *conn).await?;

    let q = a.first().unwrap().get_unchecked::<u64,_>(0);
    assert!(q == 1);


    Ok(())
}
