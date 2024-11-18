mod support;

use anyhow::Result;
use libra_warehouse::{
    neo4j_init::{get_neo4j_localhost_pool, maybe_create_indexes},
    queue,
    scan::scan_dir_archive,
};

use support::{fixtures, neo4j_testcontainer::start_neo4j_container};

#[tokio::test]
async fn test_queue() -> Result<()> {
    let c = start_neo4j_container();
    let port = c.get_host_port_ipv4(7687);
    let pool = get_neo4j_localhost_pool(port)
        .await
        .expect("could not get neo4j connection pool");
    maybe_create_indexes(&pool).await?;

    let start_here = fixtures::v7_tx_manifest_fixtures_path();

    let s = scan_dir_archive(&start_here, None)?;
    let (_, man_info) = s.0.first_key_value().unwrap();
    let batch = 0usize;

    let id = queue::update_task(&pool, &man_info.archive_id, false, batch).await?;
    assert!(id == man_info.archive_id);

    let list = queue::get_queued(&pool).await?;

    assert!(*"transaction_38100001-.541f" == list[0]);

    let c = queue::is_complete(&pool, "transaction_38100001-.541f", batch).await;
    assert!(!c?.unwrap());

    // Now we update the task, with ID and batch
    let _id = queue::update_task(&pool, &man_info.archive_id, true, batch).await?;

    let c = queue::is_complete(&pool, "transaction_38100001-.541f", batch).await;
    assert!(c?.unwrap());

    Ok(())
}
