use libra_warehouse::extract_transactions::extract_current_transactions;

mod support;

#[tokio::test]
async fn test_extract_tx_from_archive() -> anyhow::Result<()>{
  let archive_path = support::fixtures::v7_tx_manifest_fixtures_path();
  let _tx = extract_current_transactions(&archive_path).await?;
  Ok(())
}
