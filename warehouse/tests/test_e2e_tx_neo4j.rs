mod support;
use libra_warehouse::load_tx_cypher::load_tx_cypher;
use libra_warehouse::{
    extract_transactions::extract_current_transactions,
    neo4j_init::{create_indexes, get_neo4j_pool},
};
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
