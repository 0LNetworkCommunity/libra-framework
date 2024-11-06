use anyhow::Result;
use neo4rs::Graph;

pub static ACCOUNT_CONSTRAINT: &str =
    "CREATE CONSTRAINT unique_address FOR (n:Account) REQUIRE n.address IS UNIQUE";

pub static TX_CONSTRAINT: &str =
    "CREATE CONSTRAINT unique_tx_hash FOR ()-[r:Tx]-() REQUIRE r.txs_hash IS UNIQUE";

// assumes the Account.address is stored as a hex string
// NOTE: hex numericals may query faster but will be hard to use in user interface
pub static INDEX_HEX_ADDR: &str =
    "CREATE TEXT INDEX hex_addr IF NOT EXISTS FOR (n:Account) ON (n.address)";

pub static INDEX_TX_TIMESTAMP: &str =
    "CREATE INDEX tx_timestamp IF NOT EXISTS FOR ()-[r:Tx]-() ON (r.block_timestamp)";

pub static INDEX_TX_FUNCTION: &str =
    "CREATE INDEX tx_function IF NOT EXISTS FOR ()-[r:Tx]-() ON (r.function)";

pub async fn init_neo4j(graph: Graph) -> Result<()>{
    let mut txn = graph.start_txn().await.unwrap();

    txn.run_queries([
      ACCOUNT_CONSTRAINT,
      TX_CONSTRAINT,
      INDEX_HEX_ADDR,
      INDEX_TX_TIMESTAMP,
      INDEX_TX_FUNCTION,
    ]).await?;
    txn.commit().await?;
    Ok(())
}
