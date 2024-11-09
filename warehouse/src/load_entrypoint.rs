use crate::{
    extract_transactions::extract_current_transactions,
    load_tx_cypher,
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

        let (merged, ignored) = try_load_one_archive(m, pool).await?;
        println!(
            "TOTAL transactions updated: {}, ignored: {}",
            merged, ignored
        );
    }

    Ok(())
}

pub async fn try_load_one_archive(man: &ManifestInfo, pool: &Graph) -> Result<(u64, u64)> {
    let mut records_updated = 0u64;
    let mut records_ignored = 0u64;
    match man.contents {
        crate::scan::BundleContent::Unknown => todo!(),
        crate::scan::BundleContent::StateSnapshot => todo!(),
        crate::scan::BundleContent::Transaction => {
            let (txs, _) = extract_current_transactions(&man.archive_dir).await?;
            let (merged, ignored) = load_tx_cypher::tx_batch(&txs, pool, 100).await?;
            records_updated += merged;
            records_ignored += ignored;
            // TODO: make debug log
            // println!("transactions updated: {}, ignored: {}", merged, ignored);
        }
        crate::scan::BundleContent::EpochEnding => todo!(),
    }
    Ok((records_updated, records_ignored))
}
