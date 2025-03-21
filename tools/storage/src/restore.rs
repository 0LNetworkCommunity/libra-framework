// Do a full restoration given a RestoreBundle with verified manifests

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use glob::glob;
use std::fs::{self, File};
use std::io::copy;
use std::path::Path;

use crate::{
    dbtool_init::{run_restore, RestoreTypes},
    restore_bundle::RestoreBundle,
};

pub async fn maybe_decompress_gz_files(bundle_dir: &Path) -> Result<()> {
    println!(
        "Starting decompression in directory: {}",
        bundle_dir.display()
    );

    // Find all .gz files in the bundle directory and subdirectories
    let gz_pattern = bundle_dir.join("**/*.gz");
    let pattern = gz_pattern.to_str().context("Invalid path pattern")?;

    let mut found_files = false;
    for entry in glob(pattern)? {
        found_files = true;
        let gz_path = entry?;
        let output_path = gz_path.with_extension("");

        println!("Found compressed file: {}", gz_path.display());
        println!("Decompressing to: {}", output_path.display());

        let gz_file = File::open(&gz_path)?;
        let mut decoder = GzDecoder::new(gz_file);
        let mut outfile = File::create(&output_path)?;

        let bytes_copied = copy(&mut decoder, &mut outfile)?;
        println!("Decompressed {} bytes", bytes_copied);

        // Remove the original .gz file after successful decompression
        fs::remove_file(&gz_path)?;
        println!("Removed original gz file: {}", gz_path.display());
    }

    if !found_files {
        println!("No .gz files found in {}", bundle_dir.display());
    } else {
        println!("Decompression completed");
    }

    Ok(())
}

pub async fn full_restore(db_destination: &Path, bundle: &RestoreBundle) -> Result<()> {
    assert!(
        bundle.is_loaded(),
        "the restore bundle hasn't been checked yet"
    );

    println!("Starting full restore process");
    println!("Bundle directory: {}", bundle.restore_bundle_dir.display());

    // Run restores in sequence
    run_restore(RestoreTypes::Epoch, db_destination, bundle).await?;
    run_restore(RestoreTypes::Snapshot, db_destination, bundle).await?;
    run_restore(RestoreTypes::Transaction, db_destination, bundle).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[tokio::test]
    async fn test_full_restore() -> Result<()> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut b = RestoreBundle::new(dir.join("fixtures/v7"));
        b.load().unwrap();
        let mut db_temp = diem_temppath::TempPath::new();
        db_temp.persist();
        db_temp.create_as_dir()?;

        full_restore(db_temp.path(), &b).await?;

        assert!(db_temp.path().join("ledger_db").exists());
        assert!(db_temp.path().join("state_merkle_db").exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_decompress_gz_files() -> Result<()> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let test_dir = dir.join("fixtures/v7");

        // Create a test .gz file
        let test_content = b"test content";
        let gz_path = test_dir.join("test.json.gz");
        let mut encoder =
            flate2::write::GzEncoder::new(File::create(&gz_path)?, flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, test_content)?;
        encoder.finish()?;

        // Decompress files
        maybe_decompress_gz_files(&test_dir).await?;

        // Verify decompression
        let decompressed = fs::read_to_string(test_dir.join("test.json"))?;
        assert_eq!(decompressed, "test content");
        assert!(
            !gz_path.exists(),
            "gz file should be removed after decompression"
        );

        // Cleanup
        fs::remove_file(test_dir.join("test.json"))?;

        Ok(())
    }
}
