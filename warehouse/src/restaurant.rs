//! in_jest
use std::path::Path;

use crate::extract_snapshot::extract_current_snapshot;
use crate::scan::scan_dir_archive;
use crate::scan::BundleContent::*;

use anyhow::Result;

/// ingest all the archives sequentially.
pub async fn sushi_train(parent_dir: &Path) -> Result<u64> {
    let s = scan_dir_archive(parent_dir)?;
    let mut archives_processed = 0u64;
    for (i, (p, m)) in s.0.iter().enumerate() {
        match m.contents {
            Unknown => {
                println!("unknown archive type found at {p:?}")
            }
            StateSnapshot => {
                let records = extract_current_snapshot(&m.archive_dir).await?;
                archives_processed += 1;
            }
            Transaction => {
                println!("transaction archive found at {p:?}")
            }
            EpochEnding => {
                println!("epoch ending archive found at {p:?}")
            }
        }
    }

    Ok(archives_processed)
}
