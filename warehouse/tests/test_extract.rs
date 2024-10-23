use anyhow::Result;
use libra_warehouse::extract::extract_v5_snapshot;
use std::path::PathBuf;

fn v5_state_manifest_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = p.parent().unwrap();
    project_root.join("compatibility/fixtures/v5/state_ver_119757649.17a8/state.manifest")
}


#[tokio::test]

async fn test_extract_v5_manifest() -> Result<()> {
    let manifest_file = v5_state_manifest_fixtures_path();
    assert!(manifest_file.exists());
    let s = extract_v5_snapshot(&manifest_file).await?;
    // NOTE: the parsing drops 1 blob, which is the 0x1 account, because it would not have the DiemAccount struct on it as a user address would have.
    assert!(s.len() == 17338);
    Ok(())
}
