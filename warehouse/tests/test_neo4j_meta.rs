mod support;

use neo4rs::Graph;
use support::neo4j_testcontainer::start_neo4j_container;

pub async fn connect_neo4j(port: u16) {
    let uri = &format!("127.0.0.1:{port}");
    let user = "neo4j";
    let pass = "neo";
    let graph = Graph::new(&uri, user, pass).await.unwrap();
    // for _ in 1..=42 {
    //     let graph = graph.clone();
    //     tokio::spawn(async move {
    //         let mut result = graph.execute(
    //        query("MATCH (p:Person {name: $name}) RETURN p").param("name", "Mark")
    //     ).await.unwrap();
    //         while let Ok(Some(row)) = result.next().await {
    //         let node: Node = row.get("p").unwrap();
    //         let name: String = node.get("name").unwrap();
    //             println!("{}", name);
    //         }
    //     });
    // }

    //Transactions
    let mut txn = graph.start_txn().await.unwrap();
    txn.run_queries([
        "CREATE (p:Person {name: 'mark'})",
        "CREATE (p:Person {name: 'jake'})",
        "CREATE (p:Person {name: 'luke'})",
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap();
}

#[tokio::test]

fn test_neo4j_connect() {
    let c = start_neo4j_container();
    let p = c.get_host_port_ipv4(7687);
    connect_neo4j(p).await.unwrap();
}
