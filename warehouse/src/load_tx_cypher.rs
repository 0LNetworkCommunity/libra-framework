use anyhow::Result;
use neo4rs::{query, Graph};

use crate::{cypher_templates::write_batch_tx_string, table_structs::WarehouseTxMaster};

pub async fn tx_batch(
    txs: &[WarehouseTxMaster],
    pool: &Graph,
    batch_len: usize,
) -> Result<(u64, u64)> {
    let chunks: Vec<&[WarehouseTxMaster]> = txs.chunks(batch_len).collect();
    let mut merged_count = 0u64;
    let mut ignored_count = 0u64;

    for c in chunks {
        let (m, ig) = impl_batch_tx_insert(pool, c).await?;
        merged_count += m;
        ignored_count += ig;
    }

    Ok((merged_count, ignored_count))
}

pub async fn impl_batch_tx_insert(
    pool: &Graph,
    batch_txs: &[WarehouseTxMaster],
) -> Result<(u64, u64)> {
    let list_str = WarehouseTxMaster::slice_to_template(batch_txs);
    let cypher_string = write_batch_tx_string(list_str);

    // Execute the query
    let cypher_query = query(&cypher_string);
    let mut res = pool.execute(cypher_query).await?;

    let row = res.next().await?.unwrap();
    let merged: i64 = row.get("merged_tx_count").unwrap();
    let ignored: i64 = row.get("ignored_tx_count").unwrap();

    Ok((merged as u64, ignored as u64))
}
