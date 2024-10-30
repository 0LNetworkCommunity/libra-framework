use anyhow::Result;
use sqlx::PgPool;

pub static AGE_TABLE_NAME: &str = "libra_graph";
// The AGE db requires initialization.
// this is based on the official docker implementation
// for PG 16, tag: release_PG16_1.5.0

pub async fn init_age_db(pool: &PgPool) -> Result<()> {
    // NOTE: the official docker container for AGE PGv16
    // already has the extension loaded.
    // Some steps in this reference can be skipped: https://age.apache.org/getstarted/
    // So these steps have already been run:
    // ```sql
    // CREATE EXTENSION age;
    // LOAD 'age';
    // ```

    let _query = sqlx::query(
        r#"
      SET search_path = ag_catalog, "$user", public;
    "#,
    )
    .execute(pool)
    .await?;

    // the graph namespace (ag_catalog) schema needs to be made available to this user.
    // as per: https://github.com/apache/age/issues/33
    let _query = sqlx::query(
        r#"
      GRANT USAGE ON SCHEMA ag_catalog TO my_user;
      "#,
    )
    .execute(pool)
    .await?;

    let query = sqlx::query(&format!("SELECT create_graph('{AGE_TABLE_NAME}');"))
        .execute(pool)
        .await?;

    assert!(query.rows_affected() == 1);

    Ok(())
}

pub fn cypher_template(raw: &str) -> String {
    format!(
        r#"
          SELECT *
          FROM cypher('{AGE_TABLE_NAME}', $$
          {raw}
          $$) as (v agtype);
        "#
    )
}
