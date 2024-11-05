mod support;

use std::{thread, time::Duration};

use libra_warehouse::age_init::{cypher_template, init_age_db};
use support::age_testcontainer::get_test_age_pool;

#[tokio::test]
async fn test_cypher_tx_load() {
    let (pool, _c) = get_test_age_pool()
        .await
        .expect("test could not start container");

    init_age_db(&pool).await.expect("could not init AGE db");

    // thread::sleep(Duration::from_secs(10));

    // let query = sqlx::query("SELECT 1+1;").execute(&pool).await.unwrap();
    // // assert!(query.rows_affected() == 1);

    // // let query = sqlx::query(SET_SEARCH_PATH).execute(&pool).await.unwrap();

    // let query = sqlx::query(&cypher_template(
    //     r#"
    //       CREATE (a:label)
    //     "#,
    // ))
    // .execute(&pool)
    // .await
    // .unwrap();

    // // TODO: unclear why rows affected is 0, but it looks like the correct behavior
    // assert!(query.rows_affected() == 0);
}
