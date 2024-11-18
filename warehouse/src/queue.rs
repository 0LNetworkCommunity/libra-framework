use anyhow::{bail, Context, Result};
use neo4rs::Graph;

pub async fn update_task(
    pool: &Graph,
    archive_id: &str,
    completed: bool,
    batch: usize,
) -> Result<String> {
    let cypher_string = format!(
        r#"MERGE (a:Queue {{ archive_id: "{}", batch: {} }})
        SET a.completed = {}
        RETURN a.archive_id AS archive_id"#,
        archive_id,
        batch,
        completed.to_string().to_lowercase(),
    );

    let cypher_query = neo4rs::query(&cypher_string);

    let mut res = pool
        .execute(cypher_query)
        .await
        .context("execute query error")?;

    let row = res.next().await?.context("no row returned")?;
    let task_id: String = row.get("archive_id").context("no created_accounts field")?;
    Ok(task_id)
}

pub async fn get_queued(pool: &Graph) -> Result<Vec<String>> {
    let cypher_string = r#"
      MATCH (a:Queue)
      WHERE a.completed = false
      RETURN DISTINCT a.archive_id
    "#;

    let cypher_query = neo4rs::query(cypher_string);

    let mut res = pool
        .execute(cypher_query)
        .await
        .context("execute query error")?;

    let mut archive_ids: Vec<String> = vec![];

    while let Some(row) = res.next().await? {
        // Extract `archive_id` as a String
        if let Ok(archive_name) = row.get::<String>("a.archive_id") {
            archive_ids.push(archive_name);
        }
    }

    Ok(archive_ids)
}

// Three options: Not found in DB, found and complete, found and incomplete
pub async fn is_complete(pool: &Graph, archive_id: &str, batch: usize) -> Result<Option<bool>> {
    let cypher_string = format!(
        r#"
        MATCH (a:Queue {{ archive_id: "{}", batch: {} }})
        RETURN DISTINCT a.completed
      "#,
        archive_id, batch
    );

    let cypher_query = neo4rs::query(&cypher_string);

    let mut res = pool
        .execute(cypher_query)
        .await
        .context("execute query error")?;

    if let Some(row) = res.next().await? {
        // Extract `archive_id` as a String
        Ok(row.get::<bool>("a.completed").ok())
    } else {
        bail!("not found")
    }
}

// clear queue
pub async fn clear(pool: &Graph) -> Result<()> {
    let cypher_string = r#"
        MATCH (a:Queue)
        DELETE a
      "#
    .to_string();

    let cypher_query = neo4rs::query(&cypher_string);

    let mut _res = pool
        .execute(cypher_query)
        .await
        .context("execute query error")?;
    Ok(())
}
