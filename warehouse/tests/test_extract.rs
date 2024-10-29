mod support;

use anyhow::Result;
use libra_warehouse::extract::{extract_current_snapshot, extract_v5_snapshot};
use support::fixtures::{v5_state_manifest_fixtures_path, v7_state_manifest_fixtures_path};

#[tokio::test]
async fn test_extract_v5_manifest() -> Result<()> {
    let manifest_file = v5_state_manifest_fixtures_path().join("state.manifest");
    assert!(manifest_file.exists());
    let s = extract_v5_snapshot(&manifest_file).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(s.len() == 17338);
    Ok(())
}

#[tokio::test]
async fn test_extract_v7_manifest() -> Result<()> {
    let archive_dir = v7_state_manifest_fixtures_path();

    let s = extract_current_snapshot(&archive_dir).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(s.len() == 24607);
    Ok(())
}
