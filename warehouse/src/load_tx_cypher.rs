use anyhow::{Context, Result};
use neo4rs::{query, Graph};
use std::fmt::Display;

use crate::{cypher_templates::write_batch_tx_string, table_structs::WarehouseTxMaster};

/// response for the batch insert tx
#[derive(Debug, Clone)]
pub struct BatchTxReturn {
    pub created_accounts: u64,
    pub modified_accounts: u64,
    pub unchanged_accounts: u64,
    pub created_tx: u64,
}

impl Display for BatchTxReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Total Transactions - created accounts: {}, modified accounts: {}, unchanged accounts: {}, transactions created: {}",
          self.created_accounts,
          self.modified_accounts,
          self.unchanged_accounts,
          self.created_tx
        )
    }
}

impl Default for BatchTxReturn {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchTxReturn {
    pub fn new() -> Self {
        Self {
            created_accounts: 0,
            modified_accounts: 0,
            unchanged_accounts: 0,
            created_tx: 0,
        }
    }
    pub fn increment(&mut self, new: &BatchTxReturn) {
        self.created_accounts += new.created_accounts;
        self.modified_accounts += new.modified_accounts;
        self.unchanged_accounts += new.unchanged_accounts;
        self.created_tx += new.created_tx;
    }
}

pub async fn tx_batch(
    txs: &[WarehouseTxMaster],
    pool: &Graph,
    batch_len: usize,
) -> Result<BatchTxReturn> {
    let chunks: Vec<&[WarehouseTxMaster]> = txs.chunks(batch_len).collect();
    let mut all_results = BatchTxReturn::new();

    for c in chunks {
        let batch = impl_batch_tx_insert(pool, c).await?;
        all_results.increment(&batch);
    }

    Ok(all_results)
}

pub async fn impl_batch_tx_insert(
    pool: &Graph,
    batch_txs: &[WarehouseTxMaster],
) -> Result<BatchTxReturn> {
    let list_str = WarehouseTxMaster::to_cypher_map(batch_txs);
    let cypher_string = write_batch_tx_string(list_str);

    // Execute the query
    let cypher_query = query(&cypher_string);
    let mut res = pool
        .execute(cypher_query)
        .await
        .context("execute query error")?;

    let row = res.next().await?.context("no row returned")?;
    let created_accounts: u64 = row
        .get("created_accounts")
        .context("no created_accounts field")?;
    let modified_accounts: u64 = row
        .get("modified_accounts")
        .context("no modified_accounts field")?;
    let unchanged_accounts: u64 = row
        .get("unchanged_accounts")
        .context("no unchanged_accounts field")?;
    let created_tx: u64 = row.get("created_tx").context("no created_tx field")?;

    Ok(BatchTxReturn {
        created_accounts,
        modified_accounts,
        unchanged_accounts,
        created_tx,
    })
}
