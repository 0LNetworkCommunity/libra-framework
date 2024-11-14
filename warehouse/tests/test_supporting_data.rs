mod support;
use log::error;
use support::neo4j_testcontainer::start_neo4j_container;

use std::path::PathBuf;

use anyhow::Result;
use libra_warehouse::{
    load_supporting_data, log_setup,
    neo4j_init::{get_neo4j_localhost_pool, maybe_create_indexes},
    supporting_data::{read_orders_from_file, Order},
};
use neo4rs::query;

#[test]
fn open_parse_file() {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = PathBuf::from(path).join("tests/fixtures/savedOlOrders2.json");
    let orders = read_orders_from_file(buf).unwrap();
    assert!(orders.len() == 25450);
}

#[tokio::test]
async fn test_swap_batch_cypher() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port).await?;
    // Three user ids exist in these two transactions
    let order1 = Order {
        user: 1234,
        accepter: 666,
        ..Default::default()
    };

    let order2 = Order {
        user: 4567,
        accepter: 666,
        ..Default::default()
    };

    let list = vec![order1.clone(), order2];
    let cypher_map = Order::to_cypher_map(&list);
    let insert_query = Order::cypher_batch_insert_str(cypher_map);

    let mut res1 = graph.execute(query(&insert_query)).await?;

    while let Some(row) = res1.next().await? {
        let count: i64 = row.get("merged_tx_count").unwrap();
        assert!(count == 2);
    }

    // now check data was loaded
    let mut result = graph
        .execute(query("MATCH (p:SwapAccount) RETURN count(p) as num"))
        .await?;

    // three accounts should have been inserted
    while let Some(row) = result.next().await? {
        let num: i64 = row.get("num").unwrap();
        assert!(num == 3);
    }

    Ok(())
}

#[tokio::test]
async fn e2e_swap_data() -> Result<()> {
    log_setup();
    error!("start container");

    let start = std::time::Instant::now();

    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port).await?;
    maybe_create_indexes(&graph).await?;
    dbg!(&start.elapsed());

    let start = std::time::Instant::now();

    let path = env!("CARGO_MANIFEST_DIR");
    let buf = PathBuf::from(path).join("tests/fixtures/savedOlOrders2.json");
    let orders = read_orders_from_file(buf).unwrap();

    assert!(orders.len() == 25450);
    dbg!(&start.elapsed());

    // load 100 orders
    load_supporting_data::swap_batch(&orders[..1000], &graph, 1000).await?;

    // now check data was loaded
    let mut result = graph
        .execute(query("MATCH (p:SwapAccount) RETURN count(p) as num"))
        .await?;

    // check accounts should have been inserted
    while let Some(row) = result.next().await? {
        let num: i64 = row.get("num").unwrap();
        dbg!(&num);
        // assert!(num == 3);
    }

    Ok(())
}
