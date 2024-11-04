//! in_jest
use std::path::Path;

use crate::extract_snapshot::extract_current_snapshot;
use crate::scan::scan_dir_archive;
use crate::scan::BundleContent::*;

use anyhow::Result;

/// ingest all the archives sequentially.
pub async fn sushi_train(parent_dir: &Path) -> Result<()> {
    let s = scan_dir_archive(parent_dir)?;

    for (i, (_p, m)) in s.0.iter().enumerate() {
        match m.contents {
            Unknown => todo!(),
            StateSnapshot => {
                dbg!(&i);

                let records = extract_current_snapshot(&m.archive_dir).await?;
                dbg!(&records.len());
            }
            Transaction => todo!(),
            EpochEnding => todo!(),
        }
    }

    Ok(())
}
