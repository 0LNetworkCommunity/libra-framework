mod support;
use libra_warehouse::unzip_temp;


#[test]
fn test_unzip() {
  let archive_path = support::fixtures::v7_tx_manifest_fixtures_path();
  let temp_unzipped_dir = unzip_temp::make_temp_unzipped(&archive_path, true).unwrap();
  assert!(temp_unzipped_dir.exists());
  assert!(temp_unzipped_dir.join("transaction.manifest").exists())
}
