mod support;
use support::neo4j_testcontainer::start_neo4j_container;

use std::path::PathBuf;

use anyhow::Result;
use libra_warehouse::{
    neo4j_init::get_neo4j_localhost_pool,
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

    let list = vec![Order::default()];
    let cypher_map = Order::to_cypher_map(&list);

    // mostly testing timestamp insertion
    let query_str = format!(
        r#"
      WITH {} as orders
      UNWIND orders as o
      CREATE (maker:SwapId {{user: o.user, amount: o.amount, filled_at: o.filled_at}})
    "#,
        cypher_map
    );

    let _result = graph.execute(query(&query_str)).await.unwrap();

    // let mut result = graph
    //     .execute(query("MATCH (p:Person {name: $this_name}) RETURN p").param("this_name", "alice"))
    //     .await
    //     .unwrap();
    // while let Ok(Some(row)) = result.next().await {
    //     let node: Node = row.get("p").unwrap();
    //     let id: u64 = node.get("id").unwrap();
    //     assert!(id == 123);
    // }

    Ok(())
}
