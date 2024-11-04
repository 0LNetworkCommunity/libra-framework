use anyhow::{Context, Result};
use diem_temppath::TempPath;
use glob::glob;
use std::{
    fs::File,
    io::copy,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;

pub fn make_temp_unzipped(archive_path: &Path) -> Result<PathBuf> {
    let mut temp_dir = TempPath::new();
    temp_dir.create_as_dir()?;
    temp_dir.persist();
    let dst_dir = temp_dir.path();

    let pattern = format!(
        "{}/**/*.gz",
        archive_path.to_str().context("cannot parse starting dir")?
    );

    for entry in glob(&pattern)? {
        let _path = decompress_file(&entry?, dst_dir)?;
    }

    Ok(dst_dir.to_owned())
}

/// Decompresses a gzip-compressed file at `src_path` and saves the decompressed contents
/// to `dst_dir` with the same file name, but without the `.gz` extension.
fn decompress_file(src_path: &Path, dst_dir: &Path) -> Result<PathBuf> {
    // Open the source file in read-only mode
    let src_file = File::open(src_path)?;

    // Create a GzDecoder to handle the decompression
    let mut decoder = GzDecoder::new(src_file);

    // Generate the destination path with the destination directory and new file name
    let file_stem = src_path.file_stem().unwrap(); // removes ".gz"
    let dst_path = dst_dir.join(file_stem); // combines dst_dir with file_stem

    // Open the destination file in write mode
    let mut dst_file = File::create(&dst_path)?;

    // Copy the decompressed data into the destination file
    copy(&mut decoder, &mut dst_file)?;

    Ok(dst_path)
}
