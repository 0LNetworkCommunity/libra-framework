//! scan
#![allow(dead_code)]

use anyhow::{Context, Result};
use glob::glob;
use libra_backwards_compatibility::version_five::transaction_manifest_v5::v5_read_from_transaction_manifest;
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
pub enum EncodingVersion {
    Unknown,
    V5,
    V6,
    V7,
}

#[derive(Clone, Debug)]
pub enum BundleContent {
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
                let dir = path.parent().context("no parent dir found")?.to_owned();
                let contents = test_content(&path);
                let m = ManifestInfo {
                    dir: dir.clone(),
                    version: test_version(&contents, &path),
                    contents,
                    processed: false,
                };

                archive.insert(path.clone(), m);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(ArchiveMap(archive))
}

/// find out the type of content in the manifest
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

fn test_version(content: &BundleContent, manifest_file: &Path) -> EncodingVersion {
    // can it read a v5 chunk?
    match content {
        BundleContent::Unknown => return EncodingVersion::Unknown,
        BundleContent::StateSnapshot => {}
        BundleContent::Transaction => {
            if v5_read_from_transaction_manifest(manifest_file).is_ok() {
                return EncodingVersion::V5;
            }
        }
        BundleContent::EpochEnding => {}
    }

    EncodingVersion::Unknown
}
