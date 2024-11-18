use std::{path::Path, thread, time::Duration};

use anyhow::{Context, Result};
use log::{error, info, warn};
use neo4rs::{query, Graph};

use crate::{
    queue,
    supporting_data::{read_orders_from_file, SwapOrder},
};

pub async fn swap_batch(
    txs: &[SwapOrder],
    pool: &Graph,
    batch_len: usize,
    skip_to_batch: Option<usize>,
) -> Result<(u64, u64)> {
    let chunks: Vec<&[SwapOrder]> = txs.chunks(batch_len).collect();
    let mut merged_count = 0u64;
    let mut ignored_count = 0u64;

    let archive_id = "swap_orders";
    info!("archive: {}", archive_id);

    for (i, c) in chunks.iter().enumerate() {
        if let Some(skip) = skip_to_batch {
            if i < skip {
                continue;
            };
        };
        info!("batch #{}", i);

        match queue::is_complete(pool, archive_id, i).await {
            Ok(Some(true)) => {
                info!("...skipping, already loaded.");
                // skip this one
                continue;
            }
            Ok(Some(false)) => {
                // keep going
            }
            _ => {
                info!("...not found in queue, adding to queue.");

                // no task found in db, add to queue
                queue::update_task(pool, archive_id, false, i).await?;
            }
        }
        info!("...loading to db");

        match impl_batch_tx_insert(pool, c).await {
            Ok((m, ig)) => {
                queue::update_task(pool, archive_id, true, i).await?;

                info!("...success");
                info!("merged {}", m);
                info!("ignored {}", ig);

                merged_count += m;
                ignored_count += ig;
            }
            Err(e) => {
              error!("skipping batch, could not insert: {:?}", e);
              warn!("waiting 5 secs before retrying connection");
              thread::sleep(Duration::from_secs(5));
            },
        };
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
