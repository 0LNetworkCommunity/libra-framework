use anyhow::{Error, Context};
use ol_types::legacy_recovery::{self, LegacyRecovery};
use std::path::PathBuf;

/// Make a recovery genesis blob
pub fn parse(recovery_json_path: PathBuf) -> Result<Vec<LegacyRecovery>, Error> {
    Ok(legacy_recovery::read_from_recovery_file(
        &recovery_json_path,
    ))
}

/// iterate over the recovery file and get the sum of all balances.
/// Note: this may not be the "total supply", since there may be coins in other structs beside an account::balance, e.g escrowed in contracts.
pub fn get_supply(rec: &Vec<LegacyRecovery>) -> anyhow::Result<u64> {
  rec.iter().try_fold(0u64, |acc, r| {
    
    let amount = match &r.balance {
        Some(b) => b.coin(),
        None => 0,
    };
    acc.checked_add(amount).context("cannot add balance")
  })
}

/// iterate over the recovery file and get the sum of slow wallet fields:
/// locked and unlocked.
pub fn get_slow_wallet_balance(rec: &Vec<LegacyRecovery>) -> anyhow::Result<u64> {
  rec.iter().try_fold(0u64, |acc, r| {
    
    let amount = match &r.balance {
        Some(b) => b.coin(),
        None => 0,
    };
    acc.checked_add(amount).context("cannot add balance")
  })
}


#[test]
fn parse_json() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");
    
    let r = parse(p).unwrap();
    if let Some(acc) = r[0].account {
        assert!(&acc.to_string() == "B78BA84A443873F2E324C80F3E4E2334");
    }
}

#[test]
fn test_get_supply() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    
    let r = parse(p).unwrap();

    let supply = get_supply(&r).unwrap();
    assert!(supply == 1569138150863961);
}