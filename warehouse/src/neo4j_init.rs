use anyhow::{Context, Result};
use neo4rs::Graph;

pub static URI_ENV: &str = "LIBRA_GRAPH_DB_URI";
pub static USER_ENV: &str = "LIBRA_GRAPH_DB_USER";
pub static PASS_ENV: &str = "LIBRA_GRAPH_DB_PASS";

pub static ACCOUNT_UNIQUE: &str =
    "CREATE CONSTRAINT unique_address FOR (n:Account) REQUIRE n.address IS UNIQUE";

// TODO: not null requires enterprise neo4j :/
// pub static ACCOUNT_NOT_NULL: &str =
//   "CREATE CONSTRAINT account_not_null FOR (n:Account) REQUIRE n.address IS NOT NULL";

pub static TX_CONSTRAINT: &str =
    "CREATE CONSTRAINT unique_tx_hash FOR ()-[r:Tx]-() REQUIRE r.tx_hash IS UNIQUE";

// assumes the Account.address is stored as a hex string
// NOTE: hex numericals may query faster but will be hard to use in user interface
pub static INDEX_HEX_ADDR: &str =
    "CREATE TEXT INDEX hex_addr IF NOT EXISTS FOR (n:Account) ON (n.address)";

pub static INDEX_TX_TIMESTAMP: &str =
    "CREATE INDEX tx_timestamp IF NOT EXISTS FOR ()-[r:Tx]-() ON (r.block_timestamp)";

pub static INDEX_TX_FUNCTION: &str =
    "CREATE INDEX tx_function IF NOT EXISTS FOR ()-[r:Tx]-() ON (r.function)";

pub static INDEX_SWAP_ID: &str =
    "CREATE INDEX swap_account_id IF NOT EXISTS FOR (n:SwapAccount) ON (n.swap_id)";

/// get the testing neo4j connection
pub async fn get_neo4j_localhost_pool(port: u16) -> Result<Graph> {
    let uri = format!("127.0.0.1:{port}");
    let user = "neo4j";
    let pass = "neo";
    Ok(Graph::new(uri, user, pass).await?)
}

/// get the driver connection object
pub async fn get_neo4j_remote_pool(uri: &str, user: &str, pass: &str) -> Result<Graph> {
    Ok(Graph::new(uri, user, pass).await?)
}

pub fn get_credentials_from_env() -> Result<(String, String, String)> {
    let uri = std::env::var(URI_ENV).context(format!("could not get env var {}", URI_ENV))?;
    let user = std::env::var(USER_ENV).context(format!("could not get env var {}", USER_ENV))?;
    let pass = std::env::var(PASS_ENV).context(format!("could not get env var {}", PASS_ENV))?;

    Ok((uri, user, pass))
}

pub async fn maybe_create_indexes(graph: &Graph) -> Result<()> {
    let mut txn = graph.start_txn().await.unwrap();

    txn.run_queries([
        ACCOUNT_UNIQUE,
        TX_CONSTRAINT,
        INDEX_HEX_ADDR,
        INDEX_TX_TIMESTAMP,
        INDEX_TX_FUNCTION,
        INDEX_SWAP_ID,
    ])
    .await?;
    txn.commit().await?;
    Ok(())
}
