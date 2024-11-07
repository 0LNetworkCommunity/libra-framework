mod support;
use anyhow::Result;
use libra_warehouse::cypher_templates::write_batch_tx_string;
use libra_warehouse::load_tx_cypher::load_tx_cypher;
use libra_warehouse::table_structs::WarehouseTxMaster;
use libra_warehouse::{
    extract_transactions::extract_current_transactions,
    neo4j_init::{create_indexes, get_neo4j_pool},
};
use neo4rs::query;
use support::neo4j_testcontainer::start_neo4j_container;

#[tokio::test]
async fn test_parse_tx_to_neo4j() -> anyhow::Result<()> {
    let archive_path = support::fixtures::v6_tx_manifest_fixtures_path();
    let list = extract_current_transactions(&archive_path).await?;
    assert!(list.0.len() == 705);
    assert!(list.1.len() == 52);

    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    create_indexes(&graph).await.expect("could start index");

    // load in batches
    load_tx_cypher(&list.0, &graph, 100).await?;

    Ok(())
}

#[tokio::test]
async fn test_bolt_serialize() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    create_indexes(&graph).await?;

    // Define a batch of transactions as a vector of HashMaps
    let transactions = vec![WarehouseTxMaster::default()];
    let bolt_list = WarehouseTxMaster::slice_to_bolt_list(&transactions);

    // Build the query and add the transactions as a parameter
    let cypher_query = query(
        "UNWIND $transactions AS tx
         MERGE (from:Account {address: tx.sender})
         MERGE (to:Account {address: tx.recipient})
         MERGE (from)-[:Tx {tx_hash: tx.tx_hash}]->(to)",
    )
    .param("transactions", bolt_list); // Pass the batch as a parameter

    // Execute the query
    graph.run(cypher_query).await?;

    // get the sum of all transactions in db
    let cypher_query = query(
        "MATCH ()-[r:Tx]->()
         RETURN count(r) AS total_tx_count",
    );

    // Execute the query
    let mut result = graph.execute(cypher_query).await?;

    // Fetch the first row only
    let row = result.next().await?.unwrap();
    let total_tx_count: i64 = row.get("total_tx_count").unwrap();
    assert!(total_tx_count == 1);

    Ok(())
}

#[tokio::test]
async fn insert_with_cypher_string() -> Result<()> {
    let tx = WarehouseTxMaster::default();
    let list = vec![tx];
    let list_str = WarehouseTxMaster::slice_to_template(&list);
    let cypher_string = write_batch_tx_string(list_str);

    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    create_indexes(&graph).await?;

    // Execute the query
    let cypher_query = query(&cypher_string);
    graph.run(cypher_query).await?;

    // get the sum of all transactions in db
    let cypher_query = query(
        "MATCH ()-[r:Tx]->()
         RETURN count(r) AS total_tx_count",
    );

    // Execute the query
    let mut result = graph.execute(cypher_query).await?;

    // Fetch the first row only
    let row = result.next().await?.unwrap();
    let total_tx_count: i64 = row.get("total_tx_count").unwrap();
    assert!(total_tx_count == 1);
    Ok(())
}
