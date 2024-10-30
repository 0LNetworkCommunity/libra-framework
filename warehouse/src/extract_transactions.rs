use anyhow::Result;
use libra_storage::read_tx_chunk::{load_chunk, load_tx_chunk_manifest};
use std::path::Path;

pub async fn extract_current_transactions(archive_path: &Path) -> Result<()> {
    let manifest_file = archive_path.join("transaction.manifest");
    assert!(
        manifest_file.exists(),
        "{}",
        &format!("transaction.manifest file not found at {:?}", archive_path)
    );
    let manifest = load_tx_chunk_manifest(&manifest_file)?;
    for each_chunk_manifest in manifest.chunks {
      let deserialized = load_chunk(archive_path, each_chunk_manifest).await?;

      for tx in deserialized.txns {
        dbg!(&tx);
      }
    }

    Ok(())
}
