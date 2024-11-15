use crate::{
    extract_transactions::extract_current_transactions,
    load_tx_cypher::{self, BatchTxReturn},
    scan::{ArchiveMap, ManifestInfo},
};

use anyhow::Result;
use neo4rs::Graph;

/// takes all the archives from a map, and tries to load them sequentially
pub async fn ingest_all(archive_map: &ArchiveMap, pool: &Graph) -> Result<()> {
    for (_p, m) in archive_map.0.iter() {
        println!(
            "\nProcessing: {:?} with archive: {}",
            m.contents,
            m.archive_dir.display()
        );

        let batch_tx_return = try_load_one_archive(m, pool).await?;
        println!(
            "SUCCESS: {}", batch_tx_return
        );
    }

    Ok(())
}

pub async fn try_load_one_archive(man: &ManifestInfo, pool: &Graph) -> Result<BatchTxReturn> {
    let mut all_results = BatchTxReturn::new();
    match man.contents {
        crate::scan::BundleContent::Unknown => todo!(),
        crate::scan::BundleContent::StateSnapshot => todo!(),
        crate::scan::BundleContent::Transaction => {
            let (txs, _) = extract_current_transactions(&man.archive_dir).await?;
            let batch_res = load_tx_cypher::tx_batch(&txs, pool, 100).await?;
            all_results.increment(&batch_res);
            // TODO: make debug log
            // println!("transactions updated: {}, ignored: {}", merged, ignored);
        }
        crate::scan::BundleContent::EpochEnding => todo!(),
    }
    Ok(all_results)
}
