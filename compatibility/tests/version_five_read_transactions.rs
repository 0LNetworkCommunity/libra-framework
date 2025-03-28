use std::path::PathBuf;

use libra_backwards_compatibility::version_five::{
    transaction_manifest_v5::v5_read_from_transaction_manifest,
    transaction_restore_v5::read_transaction_chunk,
};

fn fixtures_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("fixtures/v5/transaction_141722729-.891d");
    assert!(p.exists());
    p
}

#[test]
fn read_transaction_manifest() {
    let archive = fixtures_path();
    let manifest = archive.join("transaction.manifest");
    assert!(manifest.exists());

    let res = v5_read_from_transaction_manifest(&manifest).unwrap();
    assert!(res.first_version == 141722729);
    assert!(res.last_version == 141722729);
}

#[tokio::test]
async fn parse_tx_chunk() {
    let archive = fixtures_path();
    let manifest = archive.join("transaction.manifest");
    assert!(manifest.exists());

    let res = v5_read_from_transaction_manifest(&manifest).unwrap();
    let _tx_chunk = read_transaction_chunk(&res.chunks[0].transactions, &archive)
        .await
        .unwrap();
    // TODO: add assert here
}
