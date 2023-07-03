
use libra_types::legacy_types::legacy_recovery::{self, LegacyRecovery, AccountRole};
use std::path::PathBuf;
use anyhow::Context;
/// Make a recovery genesis blob
pub fn parse(recovery_json_path: PathBuf) -> anyhow::Result<Vec<LegacyRecovery>> {
    Ok(legacy_recovery::read_from_recovery_file(
        &recovery_json_path,
    ))
}



/// iterate over the recovery file and get the sum of slow wallet fields:
/// locked and unlocked.
pub fn get_slow_total(rec: &Vec<LegacyRecovery>) -> anyhow::Result<u64> {
  rec.iter().try_fold(0u64, |acc, r| {
    
    let amount = match &r.slow_wallet {
        Some(b) => {
          b.unlocked + r.balance.as_ref().context("can't get balance")?.coin
        },
        None => 0,
    };
    acc.checked_add(amount).context("cannot add balance")
  })
}


/// iterate over the recovery file and get the sum of all balances.
/// Note: this may not be the "total supply", since there may be coins in other structs beside an account::balance, e.g escrowed in contracts.
pub fn get_supply(rec: &Vec<LegacyRecovery>, role: Option<AccountRole>) -> anyhow::Result<u64> {
  rec.iter().try_fold(0u64, |acc, r| {
    
    let amount = match &r.balance {
        Some(b) => {
          if let Some(ro) = &role {
            if &r.role == ro {
              b.coin
            } else {
              0
            }
          } else {
            b.coin
          }
        }
        ,
        None => 0,
    };
    acc.checked_add(amount).context("cannot add balance")
  })
}

#[test]
fn parse_json_single() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");
    
    let r = parse(p).unwrap();
    if let Some(acc) = r[0].account {
        assert!(&acc.to_string() == "6BBF853AA6521DB445E5CBDF3C85E8A0");
    }
}


#[test]
fn test_get_supply() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    
    let r = parse(p).unwrap();

    let supply = get_supply(&r, None).unwrap();
    // dbg!(&supply);
    assert!(supply == 2_397_436_809_784_621);
}

#[test]
fn test_get_slow() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    
    let r = parse(p).unwrap();

    let supply = get_slow_total(&r).unwrap();
    assert!(supply == 1_403_630_044_928_617);
}