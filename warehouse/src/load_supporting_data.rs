use std::path::Path;

use anyhow::{Context, Result};
use log::info;
use neo4rs::{query, Graph};

use crate::supporting_data::{read_orders_from_file, SwapOrder};

pub async fn swap_batch(
    txs: &[SwapOrder],
    pool: &Graph,
    batch_len: usize,
    skip_to_batch: Option<usize>,
) -> Result<(u64, u64)> {
    let chunks: Vec<&[SwapOrder]> = txs.chunks(batch_len).collect();
    let mut merged_count = 0u64;
    let mut ignored_count = 0u64;

    for (i, c) in chunks.iter().enumerate() {
        if let Some(skip) = skip_to_batch {
            if i < skip {
                continue;
            };
        };

        info!("batch #{}", i);

        let (m, ig) = impl_batch_tx_insert(pool, c).await?;
        info!("merged {}", m);
        info!("ignored {}", ig);

        merged_count += m;
        ignored_count += ig;
    }

    Ok((merged_count, ignored_count))
}

pub async fn impl_batch_tx_insert(pool: &Graph, batch_txs: &[SwapOrder]) -> Result<(u64, u64)> {
    let list_str = SwapOrder::to_cypher_map(batch_txs);
    let cypher_string = SwapOrder::cypher_batch_insert_str(list_str);

    // Execute the query
    let cypher_query = query(&cypher_string);
    let mut res = pool
        .execute(cypher_query)
        .await
        .context("execute query error")?;

    let row = res.next().await?.context("no row returned")?;
    let merged: i64 = row.get("merged_tx_count").context("no merged_tx field")?;
    let ignored: i64 = row.get("ignored_tx_count").context("no ignored_tx_count")?;

    Ok((merged as u64, ignored as u64))
}

pub async fn load_from_json(
    path: &Path,
    pool: &Graph,
    batch_len: usize,
    skip_to_batch: Option<usize>,
) -> Result<(u64, u64)> {
    let orders = read_orders_from_file(path)?;
    swap_batch(&orders, pool, batch_len, skip_to_batch).await
}
