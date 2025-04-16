// Do a full restoration given a RestoreBundle with verified manifests

use anyhow::{bail, Context, Result};
use diem_logger::info;
use flate2::read::GzDecoder;
use glob::glob;
use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};

use crate::{
    dbtool_init::{run_restore, RestoreTypes},
    restore_bundle::RestoreBundle,
};

pub async fn maybe_decompress_gz_files(bundle_dir: &Path) -> Result<()> {
    info!(
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

        info!("Found compressed file: {}", gz_path.display());
        info!("Decompressing to: {}", output_path.display());

        let gz_file = File::open(&gz_path)?;
        let mut decoder = GzDecoder::new(gz_file);
        let mut outfile = File::create(&output_path)?;

        let bytes_copied = copy(&mut decoder, &mut outfile)?;
        info!("Decompressed {} bytes", bytes_copied);

        // Remove the original .gz file after successful decompression
        fs::remove_file(&gz_path)?;
        info!("Removed original gz file: {}", gz_path.display());
    }

    if !found_files {
        info!("No .gz files found in {}", bundle_dir.display());
    } else {
        info!("Decompression completed");
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

/// Perform a complete epoch restore from a bundle to a destination DB
pub async fn epoch_restore(bundle_path: PathBuf, destination_db: PathBuf) -> Result<PathBuf> {
    if !bundle_path.exists() {
        bail!("Bundle directory not found: {}", &bundle_path.display());
    }

    if destination_db.exists() {
        bail!(
            "Destination directory already exists and may contain conflicting state: {}",
            &destination_db.display()
        );
    }

    fs::create_dir_all(&destination_db)?;

    // Canonicalize paths to avoid issues with relative paths
    let bundle_path =
        fs::canonicalize(bundle_path).context("Failed to canonicalize bundle path")?;
    let destination_db =
        fs::canonicalize(destination_db).context("Failed to canonicalize destination path")?;

    // Decompress all .gz files in the bundle directory
    maybe_decompress_gz_files(&bundle_path)
        .await
        .context("Failed to decompress gz files")?;

    let mut bundle = RestoreBundle::new(bundle_path);
    bundle.load()?;

    full_restore(&destination_db, &bundle).await?;

    info!(
        "SUCCESS: restored to epoch: {}, version: {}",
        bundle.epoch, bundle.version
    );

    Ok(destination_db)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[tokio::test]
    async fn test_full_restore() -> Result<()> {
        // don't run restore directly on fixtures path please,
        // it can modify the files in the fixtures directory
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/v7");
        let temp = diem_temppath::TempPath::new();
        temp.create_as_dir()?;
        // Copy all files recursively from fixtures to temp using fs_extra
        let copy_options = fs_extra::dir::CopyOptions::new();
        // copy_options.copy_inside(true);
        fs_extra::dir::copy(&fixtures, temp.path(), &copy_options)?;

        let test_data = temp.path().join("v7");

        let mut bundle = RestoreBundle::new(test_data);
        bundle.load().unwrap();

        let db_output_path = temp.path().join("output");
        full_restore(&db_output_path, &bundle).await?;

        assert!(db_output_path.join("ledger_db").exists());
        assert!(db_output_path.join("state_merkle_db").exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_decompress_gz_files() -> Result<()> {
        let temp = diem_temppath::TempPath::new();
        temp.create_as_dir()?;
        let test_dir = temp.path();

        // Create a test .gz file
        let test_content = b"test content";
        let gz_path = test_dir.join("test.json.gz");
        let mut encoder =
            flate2::write::GzEncoder::new(File::create(&gz_path)?, flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, test_content)?;
        encoder.finish()?;

        // Decompress files
        maybe_decompress_gz_files(test_dir).await?;

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
