use anyhow::Result;
use libra_warehouse::{scan::scan_dir_archive, unzip_temp::make_temp_unzipped};
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

fn v7_fixtures_gzipped() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = p.parent().unwrap();
    project_root.join("tools/storage/fixtures/v7/transaction_38100001-.541f_gzipped")
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

    Ok(())
}


#[test]
fn test_scan_dir_for_compressed_v7_manifests() -> Result<()> {
    let start_here = v7_fixtures_gzipped();

    let archives = scan_dir_archive(&start_here)?;

    // a normal scan should find no files.
    assert!(archives.0.iter().len() == 0);

    // This time the scan should find readable files
    let unzipped_dir = make_temp_unzipped(&start_here, true)?;

    let archives = scan_dir_archive(&unzipped_dir)?;
    assert!(archives.0.iter().len() > 0);

    Ok(())
}
