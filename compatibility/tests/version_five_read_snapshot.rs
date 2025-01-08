use std::path::PathBuf;

use libra_backwards_compatibility::version_five::{
    balance_v5::BalanceResourceV5,
    freezing_v5::FreezingBit,
    state_snapshot_v5::{v5_accounts_from_snapshot_backup, v5_read_from_snapshot_manifest},
};

fn fixtures_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("fixtures/v5/state_ver_119757649.17a8");
    assert!(p.exists());
    p
}

#[test]
fn read_snapshot_manifest() {
    let mut p = fixtures_path();
    p.push("state.manifest");
    assert!(p.exists());

    let res = v5_read_from_snapshot_manifest(&p).unwrap();

    assert!(res.version == 119757649);
}

#[tokio::test]
async fn read_full_snapshot() -> anyhow::Result<()> {
    let mut p = fixtures_path();
    p.push("state.manifest");

    let man = v5_read_from_snapshot_manifest(&p)?;
    let archive_path = fixtures_path();
    let accts = v5_accounts_from_snapshot_backup(man, &archive_path).await?;

    assert!(accts.len() == 17339);

    let first_account = accts[0].to_account_state()?;
    let f = first_account.get_resource::<FreezingBit>()?;
    assert!(!f.is_frozen());
    let b = first_account.get_resource::<BalanceResourceV5>()?;
    assert!(b.coin() == 100135989588);
    let addr = first_account.get_address()?;
    assert!(addr.len() > 0);

    Ok(())
}
