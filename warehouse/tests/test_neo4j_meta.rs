mod support;

use anyhow::Result;
use chrono::{DateTime, Utc};
use libra_warehouse::neo4j_init::{
    create_indexes, get_credentials_from_env, get_neo4j_localhost_pool, get_neo4j_remote_pool,
};
use neo4rs::{query, Node};
use std::{collections::HashMap, str::FromStr};

use support::neo4j_testcontainer::start_neo4j_container;

#[tokio::test]
async fn test_neo4j_connect() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port).await?;

    let mut txn = graph.start_txn().await.unwrap();

    txn.run_queries([
        "MERGE (p:Person {name: 'alice', id: 123 })",
        "MERGE (p:Person {name: 'bob', id: 456 })",
        "MERGE (p:Person {name: 'carol', id: 789 })",
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap();

    let mut result = graph
        .execute(query("MATCH (p:Person {name: $this_name}) RETURN p").param("this_name", "alice"))
        .await
        .unwrap();
    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("p").unwrap();
        let id: u64 = node.get("id").unwrap();
        assert!(id == 123);
    }

    Ok(())
}

#[tokio::test]
async fn test_tx_insert() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port).await?;

    let mut txn = graph.start_txn().await.unwrap();

    txn.run_queries([
      "MERGE (from:Account {address: '0xa11ce'})-[r:Tx {txs_hash: '0000000'}]->(to:Account {address: '0x808'})"
    ]).await.unwrap();
    txn.commit().await.unwrap();

    let mut result = graph
        .execute(query("MATCH p=()-[:Tx {txs_hash: '0000000'}]->() RETURN p"))
        .await?;
    let mut found_rows = 0;
    while let Ok(Some(_row)) = result.next().await {
        found_rows += 1;
    }
    assert!(found_rows == 1);

    let mut result = graph
        .execute(query("MATCH (p:Account {address: '0xa11ce'}) RETURN p"))
        .await?;
    while let Ok(Some(row)) = result.next().await {
        let node: Node = row.get("p").unwrap();
        let id: String = node.get("address").unwrap();
        dbg!(&id);
        assert!(id == *"0xa11ce");
    }

    Ok(())
}

#[tokio::test]
async fn test_init_indices() {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    create_indexes(&graph).await.expect("could start index");
}

#[tokio::test]
async fn test_unwind_create() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    create_indexes(&graph).await?;

    // Build the query and add the transactions as a parameter
    let cypher_query = query(
        r#"WITH [
        {from_address: "0xa11ce", to_address: "0x808", tx_hash: "0000000"}, {from_address: "0xb0b", to_address: "0x909", tx_hash: "1111111"}
        ] AS tx_data
        UNWIND tx_data AS tx
        MERGE (from:Account {address: tx.from_address})
        MERGE (to:Account {address: tx.to_address})
        MERGE (from)-[:Tx {tx_hash: tx.tx_hash}]->(to)"#,
    );
    // Execute the query
    graph.run(cypher_query).await?;

    Ok(())
}

#[tokio::test]
async fn test_batch_with_hashmap() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    create_indexes(&graph).await?;

    // Define a batch of transactions as a vector of HashMaps
    let transactions = vec![
        {
            let mut map = HashMap::new();
            map.insert("from_address".to_string(), "0xa11ce".to_string());
            map.insert("to_address".to_string(), "0x808".to_string());
            map.insert("txs_hash".to_string(), "0000000".to_string());
            map
        },
        {
            let mut map = HashMap::new();
            map.insert("from_address".to_string(), "0xb0b".to_string());
            map.insert("to_address".to_string(), "0x909".to_string());
            map.insert("txs_hash".to_string(), "1111111".to_string());
            map
        },
        // Add more transactions as needed
    ];

    // Build the query and add the transactions as a parameter
    let cypher_query = query(
        "UNWIND $transactions AS tx
         MERGE (from:Account {address: tx.from_address})
         MERGE (to:Account {address: tx.to_address})
         MERGE (from)-[:Tx {txs_hash: tx.txs_hash}]->(to)",
    )
    .param("transactions", transactions); // Pass the batch as a parameter

    // Execute the query
    graph.run(cypher_query).await?;

    Ok(())
}

#[ignore]
#[tokio::test]
async fn get_remote_neo4j() -> Result<()> {
    let uri = "neo4j+s://b2600969.databases.neo4j.io";

    // TODO: get from ENV
    let user = "neo4j";
    let (_, _, pass) = get_credentials_from_env()?;
    let g = get_neo4j_remote_pool(uri, user, &pass).await?;

    let mut rows = g
        .execute("CREATE (p: Account {name: 'hi'})\n RETURN p".into())
        .await?;
    let r = rows.next().await?;
    dbg!(&r);

    Ok(())
}

#[tokio::test]
async fn test_timestamp() -> Result<()> {
    let this_time = DateTime::<Utc>::from_str("2024-11-01 00:00:00+0000")?;

    // mostly testing timestamp insertion
    let insert_query = format!(
        r#"
      CREATE (t:TestTime {{user: 1, timestamp_micro: {}, date_time: datetime("{}") }})
      RETURN t
    "#,
        this_time.timestamp_micros(),
        this_time.to_rfc3339(),
    );


    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port).await?;

    let mut res1 = graph.execute(query(&insert_query)).await?;

    while let Some(row) = res1.next().await? {
        let n: Node = row.get("t").unwrap();
        let d: DateTime::<Utc> = n.get("date_time").unwrap();
        assert!(d == this_time);

        let t: i64 = n.get("timestamp_micro").unwrap();
        assert!(t == this_time.timestamp_micros());
    }

    // Now try to query it
    let range_query = r#"
      MATCH (t:TestTime)
      WHERE t.date_time >= datetime("2024-10-01T00:00:00+0000")
      AND t.date_time <= datetime("2024-11-02T00:00:00+0000")
      RETURN t
    "#;

    // now check data was loaded
    let mut result = graph.execute(query(range_query)).await?;

    while let Some(row) = result.next().await? {
        let n: Node = row.get("t").unwrap();
        let d: DateTime::<Utc> = n.get("date_time").unwrap();
        assert!(d == this_time);

        let t: i64 = n.get("timestamp_micro").unwrap();
        assert!(t == this_time.timestamp_micros());
    }

    Ok(())
}
