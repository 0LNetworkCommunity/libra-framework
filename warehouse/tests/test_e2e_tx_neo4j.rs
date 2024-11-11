mod support;
use anyhow::Result;
use diem_crypto::HashValue;
use libra_warehouse::cypher_templates::write_batch_tx_string;
use libra_warehouse::load_entrypoint::try_load_one_archive;
use libra_warehouse::load_tx_cypher::tx_batch;
use libra_warehouse::scan::scan_dir_archive;
use libra_warehouse::table_structs::WarehouseTxMaster;
use libra_warehouse::{
    extract_transactions::extract_current_transactions,
    neo4j_init::{get_neo4j_localhost_pool, maybe_create_indexes},
};
use neo4rs::query;
use support::neo4j_testcontainer::start_neo4j_container;

#[tokio::test]
async fn test_parse_archive_into_neo4j() -> anyhow::Result<()> {
    let archive_path = support::fixtures::v6_tx_manifest_fixtures_path();
    let (txs, events) = extract_current_transactions(&archive_path).await?;
    assert!(txs.len() == 705);
    assert!(events.len() == 52);

    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    maybe_create_indexes(&graph)
        .await
        .expect("could start index");

    // load in batches
    let (merged, ignored) = tx_batch(&txs, &graph, 100).await?;
    assert!(merged == 705);
    assert!(ignored == 0);

    // CHECK
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
    assert!(total_tx_count == txs.len() as i64);

    Ok(())
}

#[tokio::test]
async fn test_load_entry_point_tx() -> anyhow::Result<()> {
    let archive_path = support::fixtures::v6_tx_manifest_fixtures_path();
    let archive = scan_dir_archive(&archive_path, None)?;
    let (_, man) = archive.0.first_key_value().unwrap();

    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    maybe_create_indexes(&graph)
        .await
        .expect("could start index");

    let (merged, ignored) = try_load_one_archive(man, &graph).await?;
    assert!(merged == 705);
    assert!(ignored == 0);
    Ok(())
}

#[tokio::test]
async fn insert_with_cypher_string() -> Result<()> {
    let tx1 = WarehouseTxMaster {
        tx_hash: HashValue::random(),
        ..Default::default()
    };

    let tx2 = WarehouseTxMaster {
        tx_hash: HashValue::random(),
        ..Default::default()
    };

    // two tx records
    let list = vec![tx1, tx2];

    let list_str = WarehouseTxMaster::to_cypher_map(&list);
    let cypher_string = write_batch_tx_string(list_str);

    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    maybe_create_indexes(&graph).await?;

    // Execute the query
    let cypher_query = query(&cypher_string);
    let mut res = graph.execute(cypher_query).await?;

    let row = res.next().await?.unwrap();
    let merged: i64 = row.get("merged_tx_count").unwrap();
    assert!(merged == 2);

    let existing: i64 = row.get("ignored_tx_count").unwrap();
    assert!(existing == 0);

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
    assert!(total_tx_count == 2);
    Ok(())
}

#[ignore] // For reference deprecated in favor of string templates
#[tokio::test]
async fn test_bolt_serialize() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    maybe_create_indexes(&graph).await?;

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
