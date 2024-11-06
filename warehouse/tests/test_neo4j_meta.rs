mod support;

use anyhow::Result;
use libra_warehouse::neo4j_init::{get_neo4j_pool, create_indexes};
// use libra_warehouse::table_structs::WarehouseTxMaster;
use neo4rs::{query, Node};
use support::neo4j_testcontainer::start_neo4j_container;


#[tokio::test]
async fn test_neo4j_connect() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_pool(port).await?;

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
    let graph = get_neo4j_pool(port).await?;

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
        assert!(id == "0xa11ce".to_owned());
    }

    Ok(())
}

#[tokio::test]
async fn test_init_indices() {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_pool(port).await.expect("could not get neo4j connection pool");
    create_indexes(&graph).await.expect("could start index");
}
