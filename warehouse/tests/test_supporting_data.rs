mod support;
use support::neo4j_testcontainer::start_neo4j_container;

use std::path::PathBuf;

use anyhow::Result;
use libra_warehouse::{
    neo4j_init::get_neo4j_localhost_pool,
    supporting_data::{read_orders_from_file, Order},
};
use neo4rs::{query, Node};

#[test]
fn open_parse_file() {
    let path = env!("CARGO_MANIFEST_DIR");
    let buf = PathBuf::from(path).join("tests/fixtures/savedOlOrders2.json");
    let orders = read_orders_from_file(buf).unwrap();
    assert!(orders.len() == 25450);
}

#[test]
fn test_cypher_query_string() {
    let list = vec![Order::default()];
    let s = Order::to_cypher_map(&list);
    dbg!(&s);
}

#[tokio::test]
async fn test_order_cypher_insert() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let graph = get_neo4j_localhost_pool(port).await?;

    let order = Order::default();
    let list = vec![order.clone()];
    let cypher_map = Order::to_cypher_map(&list);
    dbg!(&cypher_map);
    // mostly testing timestamp insertion
    let insert_query = format!(
        r#"
      WITH {} as orders
      UNWIND orders as o
      MERGE (u:SwapId {{user: o.user, amount: o.amount, filled_at: o.filled_at}})

      ON CREATE SET u.created = true
      ON MATCH SET u.created = false
      WITH o, u
      RETURN
          COUNT(CASE WHEN u.created = true THEN 1 END) AS merged_tx_count,
          COUNT(CASE WHEN u.created = false THEN 1 END) AS ignored_tx_count
    "#,
        cypher_map
    );

    let mut res1 = graph.execute(query(&insert_query)).await?;

    while let Some(row) = res1.next().await? {
        dbg!(&row);
        let count: i64 = row.get("merged_tx_count").unwrap();
        assert!(count == 1);
    }

    // now check data was loaded
    let mut result = graph.execute(query("MATCH (p:SwapId) RETURN p")).await?;
    println!("hi");

    while let Some(row) = result.next().await? {
        dbg!(&row);
        let n: Node = row.get("p").unwrap();
        let count: i64 = n.get("user").unwrap();
        assert!(count == 0);
    }

    Ok(())
}
