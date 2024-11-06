use anyhow::Result;
use neo4rs::Graph;

use crate::table_structs::WarehouseTxMaster;

pub async fn load_tx_cypher(txs: &[WarehouseTxMaster], pool: &Graph, batch_len: usize) -> Result<()> {
    let chunks: Vec<&[WarehouseTxMaster]> = txs.chunks(batch_len).collect();
    for c in chunks {
        impl_batch_tx_insert(pool, c).await?;
    }

    Ok(())
}

pub async fn impl_batch_tx_insert(pool: &Graph, batch_txs: &[WarehouseTxMaster]) -> Result<u64> {
    let mut queries: Vec<String> = vec![];

    for tx in batch_txs  {
      let mut this_query = tx.to_cypher();
      queries.append(&mut this_query);
    }

    let mut txn = pool.start_txn().await?;

    txn.run_queries(queries)
    .await?;

    txn.commit().await.unwrap();

    Ok(0)
}
