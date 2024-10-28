use anyhow::Result;
use libra_warehouse::scan::scan_dir_archive;
use std::path::PathBuf;

fn v5_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = p.parent().unwrap();
    project_root.join("compatibility/fixtures")
}

fn v7_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = p.parent().unwrap();
    project_root.join("tools/storage/fixtures/v7")
}

#[test]

fn test_scan_dir_for_v5_manifests() -> Result<()> {
    let start_here = v5_fixtures_path();

    let s = scan_dir_archive(&start_here)?;

    assert!(s.0.len() == 2);
    Ok(())
}

#[test]

fn test_scan_dir_for_v7_manifests() -> Result<()> {
    let start_here = v7_fixtures_path();

    let s = scan_dir_archive(&start_here)?;

    let archives = s.0;
    assert!(archives.len() == 3);

    for (p, a) in archives.iter() {
        dbg!(&p);
        dbg!(&a);
    }

    Ok(())
}
