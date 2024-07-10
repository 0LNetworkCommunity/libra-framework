//! Module for handling recovery genesis blob creation

use libra_types::{
    exports::{AccountAddress, AuthenticationKey},
    legacy_types::legacy_recovery_v6::{self, AccountRole, LegacyRecoveryV6},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Make a recovery genesis blob
pub fn recovery_file_parse(recovery_json_path: PathBuf) -> anyhow::Result<Vec<LegacyRecoveryV6>> {
    let mut r = legacy_recovery_v6::read_from_recovery_file(&recovery_json_path);

    fix_slow_wallet(&mut r)?;

    Ok(r)
}

/// Fixes slow wallet issues in `LegacyRecoveryV6`.
fn fix_slow_wallet(r: &mut [LegacyRecoveryV6]) -> anyhow::Result<Vec<AccountAddress>> {
    let mut errs = vec![];
    r.iter_mut().for_each(|e| {
        if e.account.is_some() && e.balance.is_some() {
            if let Some(s) = e.slow_wallet.as_mut() {
                let balance = e.balance.as_ref().unwrap().coin;

                if s.unlocked > balance {
                    s.unlocked = balance;
                    errs.push(e.account.as_ref().unwrap().to_owned())
                }
            }
        }
    });

    Ok(errs)
}

/// Represents an account to be dropped from the legacy recovery data.
#[derive(Serialize, Deserialize)]
struct DropList {
    account: AccountAddress,
}

/// strip accounts from legacy
pub fn drop_accounts(r: &mut [LegacyRecoveryV6], drop_file: &Path) -> anyhow::Result<()> {
    let data = fs::read_to_string(drop_file).expect("Unable to read file");
    let list: Vec<DropList> = serde_json::from_str(&data).expect("Unable to parse");
    let mapped: Vec<AccountAddress> = list.into_iter().map(|e| e.account).collect();
    let tombstone = [9u8; 32];
    // let auth_key = b"Oh, is it too late now to say sorry?".to_vec();
    // tombstone.copy_from_slice(&auth_key);
    r.iter_mut().for_each(|e| {
        if let Some(account) = e.account {
            if mapped.contains(&account) {
                let mut dead = LegacyRecoveryV6 {
                    ..Default::default()
                };
                dead.account = Some(account);
                dead.auth_key = Some(AuthenticationKey::new(tombstone));
                dead.role = AccountRole::Drop;
                *e = dead;
            }
        }
    });

    let path = drop_file.parent().unwrap().join("migration_sanitized.json");
    let json = serde_json::to_string(r)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[test]
fn parse_json_single() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/single.json");

    let r = recovery_file_parse(p).unwrap();
    if let Some(acc) = r[0].account {
        assert_eq!(
            &acc.to_string(),
            "0000000000000000000000000000000045558bad546e6159020871f7e5d094d7"
        );
    }
    dbg!(&r.len());
}

#[test]
fn parse_json_all() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let mut r = recovery_file_parse(p).unwrap();

    // parse again to see if we got any errors back.
    let res = fix_slow_wallet(&mut r).unwrap();
    assert!(res.is_empty());

    // this is a case of an account that had to be patched.
    let a = r
        .iter()
        .find(|e| e.account.unwrap().to_hex_literal() == "0x7f10901425237ee607afa9cc80e5df3e")
        .expect("should have account");
    assert_eq!(
        a.balance.as_ref().unwrap().coin,
        a.slow_wallet.as_ref().unwrap().unlocked,
        "unlocked should equal balance"
    );
}

#[test]
fn includes_all_user_structs() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let r = recovery_file_parse(p).unwrap();

    let f = r.iter().filter(|e| e.burn_tracker.is_some());
    let b: Vec<&LegacyRecoveryV6> = f.collect();
    assert!(!b.is_empty());
}
