use anyhow::Context;
use clap::Args;
use indicatif::ProgressBar;
use libra_types::ol_progress::OLProgress;
use libra_types::{
    legacy_types::{legacy_address::LegacyAddress, legacy_recovery::LegacyRecovery},
    ONCHAIN_DECIMAL_PRECISION,
};
use std::time::Duration;

#[derive(Debug, Clone, Args)]
pub struct SupplySettings {
    #[clap(long)]
    /// what is the final supply units to be split to. This is an "unscaled" number, meaning you should use the expected integer units of the coin, without the decimal precision.
    pub target_supply: f64,
    #[clap(long)]
    /// for calculating escrow, what's the desired percent to future uses
    pub target_future_uses: f64,
    #[clap(long)]
    /// for calculating base case validator reward
    pub years_escrow: u64,
    #[clap(long)]
    /// for future uses calc, are there any donor directed wallets which require mapping to slow wallets
    pub map_dd_to_slow: Vec<LegacyAddress>,
}

impl Default for SupplySettings {
    fn default() -> Self {
        Self {
            target_supply: 100_000_000_000.0,
            target_future_uses: 0.0,
            years_escrow: 10,
            map_dd_to_slow: vec![],
        }
    }
}

impl SupplySettings {
    // convert to the correct coin scaling
    pub fn scale_supply(&self) -> f64 {
        self.target_supply * 10f64.powf(ONCHAIN_DECIMAL_PRECISION.into())
    }
}
#[derive(Debug, Clone, Default)]
pub struct Supply {
    pub total: f64,
    pub normal: f64,
    pub validator: f64, // will overlap with slow wallet
    pub slow_total: f64,
    pub slow_locked: f64,
    pub slow_validator_locked: f64,
    pub slow_unlocked: f64,
    pub donor_directed: f64,
    pub make_whole: f64,
    // which will compute later
    pub split_factor: f64,
    pub escrow_pct: f64,
    pub epoch_reward_base_case: f64,
}

impl Supply {
    // returns the ratios (split_factor, escrow_pct)
    pub fn set_ratios_from_settings(&mut self, settings: &SupplySettings) -> anyhow::Result<()> {
        // NOTE IMPORTANT: the CLI receives an unscaled integer number. And it should be scaled up to the Movevm decimal precision being used: 10^6
        self.split_factor = settings.scale_supply() / self.total;

        let target_future_uses = settings.target_future_uses * self.total;
        let remaining_to_fund = target_future_uses - self.donor_directed;
        self.escrow_pct = remaining_to_fund / self.slow_validator_locked;
        self.epoch_reward_base_case =
            remaining_to_fund / (365 * 100 * settings.years_escrow) as f64; // one hundred validators over 7 years every day. Note: discussed elsewhere: if this is an over estimate, the capital gets returned to community by the daily excess burn.

        Ok(())
    }
}

fn inc_supply(
    mut acc: Supply,
    r: &LegacyRecovery,
    dd_wallet_list: &[LegacyAddress],
) -> anyhow::Result<Supply> {
    // get balances
    let amount: f64 = match &r.balance {
        Some(b) => b.coin as f64,
        None => 0.0,
    };
    acc.total += amount;

    // get loose coins in make_whole
    if let Some(mk) = &r.make_whole {
        let user_credits = mk.credits.iter().fold(0, |sum, e| {
            if !e.claimed {
                return sum + e.coins.value;
            }
            sum
        }) as f64;

        acc.total += user_credits;
        acc.make_whole += user_credits;
    }

    // get donor directed
    if dd_wallet_list.contains(&r.account.unwrap()) {
        acc.donor_directed += amount;
    } else if let Some(sl) = &r.slow_wallet {
        acc.slow_total += amount;
        if sl.unlocked > 0 {
            acc.slow_unlocked += amount;
            if amount > sl.unlocked as f64 {
                // Note: the validator may have transferred everything out, and the unlocked may not have changed
                let locked = amount - sl.unlocked as f64;
                acc.slow_locked += locked;
                // if this is the special case of a validator account with slow locked balance
                if r.val_cfg.is_some() {
                    acc.validator += amount;
                    acc.slow_validator_locked += locked;
                }
            }
        }
    } else if r.cumulative_deposits.is_some() {
        // catches the cases of any dd wallets that were mapped to slow wallets
        acc.slow_locked += amount;
        acc.slow_total += amount;
    } else {
        acc.normal += amount;
    }
    Ok(acc)
}

/// iterate over the recovery file and get the sum of all balances.
/// there's an option to map certain donor-directed wallets to be counted as slow wallets
/// Note: this may not be the "total supply", since there may be coins in other structs beside an account::balance, e.g escrowed in contracts.
pub fn populate_supply_stats_from_legacy(
    rec: &[LegacyRecovery],
    map_dd_to_slow: &[LegacyAddress],
) -> anyhow::Result<Supply> {
    let pb = ProgressBar::new(1000)
        .with_style(OLProgress::spinner())
        .with_message("calculating coin supply");
    pb.enable_steady_tick(Duration::from_millis(100));
    let zeroth = Supply {
        total: 0.0,
        normal: 0.0,
        validator: 0.0,
        slow_total: 0.0,
        slow_locked: 0.0,
        slow_validator_locked: 0.0,
        slow_unlocked: 0.0,
        donor_directed: 0.0,
        make_whole: 0.0,
        split_factor: 0.0,
        escrow_pct: 0.0,
        epoch_reward_base_case: 0.0,
    };

    let dd_wallets = rec
        .iter()
        .find(|el| el.comm_wallet.is_some())
        .context("could not find 0x0 state in recovery file")?
        .comm_wallet
        .as_ref()
        .context("could not find list of community wallets")?;

    let dd_list: Vec<LegacyAddress> = dd_wallets
        .clone()
        .list
        .into_iter()
        .filter(|e| !map_dd_to_slow.contains(e))
        .collect();

    let s = rec
        .iter()
        .try_fold(zeroth, |acc, r| inc_supply(acc, r, &dd_list))?;
    pb.finish_and_clear();
    Ok(s)
}

#[test]
fn test_genesis_math() {
    use std::path::PathBuf;
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let r = crate::parse_json::recovery_file_parse(p).unwrap();

    let settings = SupplySettings {
        target_supply: 10_000_000_000.0,
        target_future_uses: 0.70,
        years_escrow: 10,
        map_dd_to_slow: vec![
            // FTW
            "3A6C51A0B786D644590E8A21591FA8E2"
                .parse::<LegacyAddress>()
                .unwrap(),
            // tip jar
            "2B0E8325DEA5BE93D856CFDE2D0CBA12"
                .parse::<LegacyAddress>()
                .unwrap(),
        ],
    };

    // confirm the supply of normal, slow, and donor directed will add up to 100%``

    let mut supply = populate_supply_stats_from_legacy(&r, &settings.map_dd_to_slow).unwrap();
    dbg!(&supply);

    println!("before");
    let pct_normal = supply.normal / supply.total;

    let pct_dd = supply.donor_directed / supply.total;

    let pct_slow = supply.slow_total / supply.total;

    let _pct_val_locked = supply.slow_validator_locked / supply.total;

    let sum_all_pct = pct_normal + pct_slow + pct_dd;
    assert!(sum_all_pct == 1.0);
    assert!(supply.total == 2397436809784621.0);

    // genesis infra escrow math
    // future uses is intended to equal 70% in this scenario.
    dbg!("after");
    supply.set_ratios_from_settings(&settings).unwrap();

    // escrow comes out of validator locked only
    let to_escrow = supply.escrow_pct * supply.slow_validator_locked;
    let new_slow = supply.slow_total - to_escrow;
    dbg!(&pct_normal);
    dbg!(&pct_dd);
    dbg!(new_slow / supply.total);
    dbg!(to_escrow / supply.total);

    let sum_all = to_escrow + new_slow + supply.normal + supply.donor_directed;
    assert!(supply.total == sum_all);
}
