use std::path::PathBuf;

use libra_backwards_compatibility::version_five::transaction_manifest_v5::v5_read_from_transaction_manifest;

fn fixtures_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("fixtures/v5/transaction_141722729-.891d");
    assert!(p.exists());
    p
}

#[test]
fn read_transaction_manifest() {
    let mut p = fixtures_path();
    p.push("transaction.manifest");
    assert!(p.exists());

    let res = v5_read_from_transaction_manifest(&p).unwrap();
    assert!(res.first_version == 141722729);
    assert!(res.last_version == 141722729);
}
