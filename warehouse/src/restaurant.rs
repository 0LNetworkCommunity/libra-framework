//! in_jest
use std::path::Path;

use crate::extract_snapshot::extract_current_snapshot;
use crate::load_account;
use crate::scan::scan_dir_archive;
use crate::scan::BundleContent::*;

use anyhow::Result;
use sqlx::PgPool;

/// ingest all the archives sequentially.
/// not very good, and made for the lazy
pub async fn sushi_train(parent_dir: &Path, pool: &PgPool) -> Result<u64> {
    let s = scan_dir_archive(parent_dir, None)?;
    let mut archives_processed = 0u64;
    for (p, m) in s.0.iter() {
        match m.contents {
            Unknown => {
                println!("unknown archive type found at {p:?}")
            }
            StateSnapshot => {
                let records = extract_current_snapshot(&m.archive_dir).await?;
                let _ = load_account::batch_insert_account(pool, &records, 1000).await?;

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
