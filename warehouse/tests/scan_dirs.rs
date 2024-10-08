use std::path::{Path, PathBuf};
use anyhow::Result;
use libra_warehouse::scan::scan_dir_archive;

fn fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let project_root = p.parent().unwrap();
  project_root.join("compatibility/fixtures")
}

#[test]

fn test_scan_dir_for_manifests() -> Result<()>{
  let start_here = fixtures_path();

  let s = scan_dir_archive(&start_here)?;

  assert!(s.0.len() == 2);
  Ok(())
}
