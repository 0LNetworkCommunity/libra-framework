use anyhow::Result;
use neo4rs::{query, Graph};

use crate::table_structs::WarehouseTxMaster;

pub async fn load_tx_cypher(
    txs: &[WarehouseTxMaster],
    pool: &Graph,
    batch_len: usize,
) -> Result<()> {
    let chunks: Vec<&[WarehouseTxMaster]> = txs.chunks(batch_len).collect();
    for c in chunks {
        impl_batch_tx_insert(pool, c).await?;
    }

    Ok(())
}

pub async fn impl_batch_tx_insert(pool: &Graph, batch_txs: &[WarehouseTxMaster]) -> Result<u64> {
    let transactions = WarehouseTxMaster::slice_to_bolt_list(batch_txs);

    // for tx in batch_txs {
    //     let mut this_query = tx.to_hashmap();
    //     transactions.push(this_query);
    // }

    let mut txn = pool.start_txn().await?;

    let q = query(
        "UNWIND $transactions AS tx
         MERGE (from:Account {address: tx.sender})
         MERGE (to:Account {address: tx.recipient})
         MERGE (from)-[:Tx {tx_hash: tx.tx_hash}]->(to)",
    )
    .param("transactions", transactions);

    txn.run(q).await?;
    txn.commit().await?;

    Ok(0)
}
