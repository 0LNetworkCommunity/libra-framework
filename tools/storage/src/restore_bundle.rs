use anyhow::bail;
use diem_backup_cli::backup_types::{
    epoch_ending::manifest::EpochEndingBackup, transaction::manifest::TransactionBackup,
};
use diem_logger::info;
use diem_types::waypoint::Waypoint;
use glob::glob;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Clone)]
/// Struct to organize the required bundle files for a given epoch archive.
pub struct RestoreBundle {
    /// the directory of the restore bundle
    pub restore_bundle_dir: PathBuf,
    /// epoch we are restoring to
    pub epoch: u64,
    /// the blockchain version to restore to
    pub version: u64,
    /// waypoint
    pub waypoint: Option<Waypoint>,
    /// epoch manifest file location (under restore_bundle_dir)
    pub epoch_manifest: PathBuf,
    /// snapshot manifest file location (under restore_bundle_dir)
    pub snapshot_manifest: PathBuf,
    /// transaction manifest file location (under restore_bundle_dir)
    pub transaction_manifest: PathBuf,
}

impl RestoreBundle {
    pub fn new(restore_bundle_dir: PathBuf) -> Self {
        Self {
            restore_bundle_dir,
            epoch: 0,
            version: 0,
            waypoint: None,
            epoch_manifest: PathBuf::new(),
            snapshot_manifest: PathBuf::new(),
            transaction_manifest: PathBuf::new(),
        }
    }

    /// searches and checks that the manifests would work for this epoch upgrade
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.any_epoch_manifest()?;
        self.set_version()?;
        self.search_snapshot_manifest()?;
        self.search_transaction_manifest()?;
        Ok(())
    }

    /// if we have the manifests checked
    pub fn is_loaded(&self) -> bool {
        self.epoch > 0
            && self.version > 0
            && self.epoch_manifest.exists()
            && self.snapshot_manifest.exists()
            && self.transaction_manifest.exists()
    }

    /// in the default case the user only has one epoch bundle in the directory
    pub fn any_epoch_manifest(&mut self) -> anyhow::Result<()> {
        let file_list = glob(&format!(
            "{}/epoch_ending*/epoch_ending.manifest",
            &self.restore_bundle_dir.display(),
        ))?;

        if let Some(p) = file_list.flatten().max() {
            self.epoch_manifest = p;
            let content = fs::read_to_string(&self.epoch_manifest)?;
            // Update paths and write back
            let updated_content = Self::update_manifest_paths(&content);
            fs::write(&self.epoch_manifest, &updated_content)?; // Add & here

            let epoch_manifest: EpochEndingBackup = serde_json::from_str(&updated_content)?;
            self.epoch = epoch_manifest.first_epoch;
            self.waypoint = epoch_manifest.waypoints.clone().pop();
        }

        info!(
            "using bundle for epoch: {}, manifest: {}",
            self.epoch,
            self.epoch_manifest.display()
        );
        Ok(())
    }

    /// if the directory has many bundles, pick a specific epoch
    pub fn specific_epoch_manifest(&mut self, epoch: u64) -> anyhow::Result<()> {
        let file_list = glob(&format!(
            "{}/*_{}*/epoch_ending.manifest",
            &self.restore_bundle_dir.display(),
            epoch
        ))?;

        if let Some(p) = file_list.flatten().max() {
            self.epoch_manifest = p;
        }

        Ok(())
    }

    pub fn set_version(&mut self) -> anyhow::Result<()> {
        dbg!(&self.epoch_manifest);
        assert!(
            self.epoch_manifest.exists(),
            "this epoch manifest file does not exist"
        );

        let s = fs::read_to_string(&self.epoch_manifest)?;
        let epoch_manifest: EpochEndingBackup = serde_json::from_str(&s)?;
        if let Some(wp) = epoch_manifest.waypoints.first() {
            self.version = wp.version();
            self.epoch = epoch_manifest.first_epoch
        }
        Ok(())
    }

    pub fn search_snapshot_manifest(&mut self) -> anyhow::Result<()> {
        assert!(
            self.epoch_manifest.exists(),
            "this epoch manifest file does not exist"
        );
        assert!(
            self.version > 0,
            "you haven't yet set the version of the epoch restore"
        );
        let file_list = glob(&format!(
            "{}/state_epoch*{}*{}*/state.manifest",
            &self.restore_bundle_dir.display(),
            &self.epoch,
            &self.version,
        ))?;

        if let Some(p) = file_list.flatten().max() {
            self.snapshot_manifest = p;
            let content = fs::read_to_string(&self.snapshot_manifest)?;
            let updated_content = Self::update_manifest_paths(&content);
            fs::write(&self.snapshot_manifest, &updated_content)?; // Add & here
        }

        Ok(())
    }

    pub fn search_transaction_manifest(&mut self) -> anyhow::Result<()> {
        assert!(
            self.epoch_manifest.exists(),
            "this epoch manifest file does not exist"
        );
        assert!(
            self.version > 0,
            "you haven't yet set the version of the epoch restore"
        );
        let file_list = glob(&format!(
            "{}/**/transaction.manifest",
            &self.restore_bundle_dir.display(),
        ))?;

        for entry in file_list.flatten() {
            let content = fs::read_to_string(&entry)?;
            let updated_content = Self::update_manifest_paths(&content);
            fs::write(&entry, &updated_content)?; // Add & here
            verify_valid_transaction_list(&entry, self.version)?;

            self.transaction_manifest = entry;
        }
        Ok(())
    }

    fn update_manifest_paths(manifest_content: &str) -> String {
        let mut manifest: Value = serde_json::from_str(manifest_content).unwrap();

        if let Some(obj) = manifest.as_object_mut() {
            // Update chunks paths
            if let Some(chunks) = obj.get_mut("chunks").and_then(|c| c.as_array_mut()) {
                for chunk in chunks {
                    if let Some(chunk_obj) = chunk.as_object_mut() {
                        for (_, value) in chunk_obj.iter_mut() {
                            if let Some(path_str) = value.as_str() {
                                if path_str.ends_with(".gz") {
                                    *value = Value::String(
                                        path_str.strip_suffix(".gz").unwrap().to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // Handle top-level proof field (specific to state manifest)
            if let Some(proof) = obj.get_mut("proof") {
                if let Some(path_str) = proof.as_str() {
                    if path_str.ends_with(".gz") {
                        *proof = Value::String(path_str.strip_suffix(".gz").unwrap().to_string());
                    }
                }
            }
        }

        serde_json::to_string_pretty(&manifest).unwrap()
    }
}

pub fn verify_valid_transaction_list(
    transaction_manifest: &Path,
    version: u64,
) -> anyhow::Result<()> {
    let s = fs::read_to_string(transaction_manifest)?;
    let tm: TransactionBackup = serde_json::from_str(&s)?;
    dbg!(&version);
    dbg!(&tm.last_version);
    if tm.last_version < version {
        bail!("the transaction you are looking for is newer than the last version in this bundle. Get a newer transaction backup");
    };

    if tm.first_version > version {
        bail!("the transaction you are looking for is older than the last version in this bundle. Get an older transaction backup.");
    }
    println!("OK: transaction bundle should have this transaction");
    Ok(())
}

#[test]
fn get_specific_epoch() {
    let mut b = RestoreBundle::default();

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    b.restore_bundle_dir = dir.join("fixtures/v7");

    b.specific_epoch_manifest(116).unwrap();
    b.set_version().unwrap();
    b.search_snapshot_manifest().unwrap();
    b.search_transaction_manifest().unwrap();
}

#[test]
fn test_load_any() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut b = RestoreBundle::new(dir.join("fixtures/v7"));
    b.load().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_manifest_paths() {
        // Test state manifest format
        let state_manifest = r#"{
            "version": 117583050,
            "chunks": [
                {
                    "blobs": "state_epoch_339_ver_117583050.05f3/0-.chunk.gz",
                    "proof": "state_epoch_339_ver_117583050.05f3/0-133368.proof.gz"
                }
            ],
            "proof": "state_epoch_339_ver_117583050.05f3/state.proof.gz"
        }"#;

        let updated = RestoreBundle::update_manifest_paths(state_manifest);
        assert!(!updated.contains(".gz"));
        assert!(updated.contains("state.proof"));
        assert!(updated.contains("0-.chunk"));
        assert!(updated.contains("0-133368.proof"));
    }
}
