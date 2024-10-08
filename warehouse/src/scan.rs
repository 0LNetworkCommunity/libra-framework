//! scan
use anyhow::{Context, Result};
use glob::glob;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct ArchiveMap(pub BTreeMap<PathBuf, ManifestInfo>);

#[derive(Clone, Debug)]

pub struct ManifestInfo {
    /// the enclosing directory of the .manifest file
    pub dir: PathBuf,
    /// what libra version were these files encoded with (v5 etc)
    pub version: EncodingVersion,
    /// contents of the manifest
    pub contents: BundleContent,
    /// processed
    pub processed: bool,
}

#[derive(Clone, Debug)]
enum EncodingVersion {
    Unknown,
    V5,
    V6,
    V7,
}

#[derive(Clone, Debug)]
enum BundleContent {
    Unknown,
    StateSnapshot,
    Transaction,
    EpochEnding,
}

/// Crawl a directory and find all .manifest files
pub fn scan_dir_archive(start_dir: &Path) -> Result<ArchiveMap> {
    let path = start_dir.canonicalize()?;

    let pattern = format!(
        "{}/**/*.manifest",
        path.to_str().context("cannot parse starting dir")?
    );

    let mut archive = BTreeMap::new();

    for entry in glob(&pattern)? {
        match entry {
            Ok(path) => {
                let m = ManifestInfo {
                    dir: path.parent().context("no parent dir found")?.to_owned(),
                    version: EncodingVersion::V5,
                    contents: test_content(&path),
                    processed: false,
                };

                archive.insert(path.clone(), m);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(ArchiveMap(archive))
}

fn test_content(manifest_path: &Path) -> BundleContent {
    let s = manifest_path.to_str().expect("path invalid");
    if s.contains("transaction.manifest") {
        return BundleContent::Transaction;
    };
    if s.contains("epoch_ending.manifest") {
        return BundleContent::EpochEnding;
    };
    if s.contains("state.manifest") {
        return BundleContent::StateSnapshot;
    };

    BundleContent::Unknown
}
