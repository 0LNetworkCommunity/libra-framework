//! scan
#![allow(dead_code)]

use anyhow::{Context, Result};
use glob::glob;
use libra_backwards_compatibility::version_five::{
    state_snapshot_v5::v5_read_from_snapshot_manifest,
    transaction_manifest_v5::v5_read_from_transaction_manifest,
};
use libra_storage::read_snapshot::load_snapshot_manifest;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub struct ArchiveMap(pub BTreeMap<PathBuf, ManifestInfo>);

#[derive(Clone, Debug)]

pub struct ManifestInfo {
    /// the enclosing directory of the local .manifest file
    pub archive_dir: PathBuf,
    /// the name of the directory, as a unique archive identifier
    pub archive_id: String,
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

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum BundleContent {
    Unknown,
    StateSnapshot,
    Transaction,
    EpochEnding,
}
impl BundleContent {
    pub fn filename(&self) -> String {
        match self {
            BundleContent::Unknown => "*.manifest".to_string(),
            BundleContent::StateSnapshot => "state.manifest".to_string(),
            BundleContent::Transaction => "transaction.manifest".to_string(),
            BundleContent::EpochEnding => "epoch_ending.manifest".to_string(),
        }
    }
}

/// Crawl a directory and find all .manifest files.
/// Optionally find
pub fn scan_dir_archive(
    parent_dir: &Path,
    content_opt: Option<BundleContent>,
) -> Result<ArchiveMap> {
    let path = parent_dir.canonicalize()?;
    let filename = content_opt.unwrap_or(BundleContent::Unknown).filename();
    let pattern = format!(
        "{}/**/{}",
        path.to_str().context("cannot parse starting dir")?,
        filename,
    );

    let mut archive = BTreeMap::new();

    for entry in glob(&pattern)? {
        match entry {
            Ok(path) => {
                let dir = path.parent().context("no parent dir found")?.to_owned();
                let contents = test_content(&path);
                let archive_id = dir.file_name().unwrap().to_str().unwrap().to_owned();
                let m = ManifestInfo {
                    archive_dir: dir.clone(),
                    archive_id,
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
    match content {
        BundleContent::Unknown => return EncodingVersion::Unknown,
        BundleContent::StateSnapshot => {
            // first check if the v7 manifest will parse
            if load_snapshot_manifest(manifest_file).is_ok() {
                return EncodingVersion::V7;
            }

            if v5_read_from_snapshot_manifest(manifest_file).is_ok() {
                return EncodingVersion::V5;
            }
        }
        BundleContent::Transaction => {
            // TODO: v5 manifests appear to have the same format this is a noop
            if v5_read_from_transaction_manifest(manifest_file).is_ok() {
                return EncodingVersion::V5;
            }
        }
        BundleContent::EpochEnding => {}
    }

    EncodingVersion::Unknown
}
