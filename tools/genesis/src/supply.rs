use indicatif::ProgressBar;
use libra_types::{legacy_types::legacy_recovery_v6::LegacyRecoveryV6, ol_progress::OLProgress};
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct Supply {
    pub total: f64,
    pub normal: f64,
    pub validator: f64, // will overlap with slow wallet
    pub slow_total: f64,
    pub slow_locked: f64,
    pub slow_validator_locked: f64,
    pub slow_unlocked: f64,
    pub donor_voice: f64,
    pub make_whole: f64,
    // which will compute later
    pub split_factor: f64,
    pub escrow_pct: f64,
    pub epoch_reward_base_case: f64,
    pub expected_user_balance: f64,
    pub expected_user_ratio: f64,
    pub expected_circulating: f64,
    pub expected_circulating_ratio: f64,
}

impl Supply {
    fn inc_supply(&mut self, r: &LegacyRecoveryV6) -> &mut Self {
        // get balances
        let user_total: f64 = match &r.balance {
            Some(b) => b.coin as f64,
            None => 0.0,
        };
        self.total += user_total;

        // sum all accounts
        if let Some(sl) = &r.slow_wallet {
            // is it a slow wallet?
            self.slow_total += user_total;
            if sl.unlocked > 0 {
                // safety check, the unlocked should always be lower than total balance
                if user_total > sl.unlocked as f64 {
                    self.slow_unlocked += sl.unlocked as f64;
                    // Note: the validator may have transferred everything out, and the unlocked may not have changed
                    let locked = user_total - sl.unlocked as f64;
                    self.slow_locked += locked;
                    // if this is the special case of a validator account with slow locked balance
                    if r.val_cfg.is_some() {
                        self.validator += user_total;
                        self.slow_validator_locked += locked;
                    }
                } else {
                    // we shouldn't have more unlocked coins than the actual balance
                    self.slow_unlocked += user_total;
                }
            }
        } else if r.cumulative_deposits.is_some() {
            // catches the cases of any dd wallets that were mapped to slow wallets
            self.slow_locked += user_total;
            self.slow_total += user_total;
        } else {
            self.normal += user_total;
        }
        self
    }
}

/// iterate over the recovery file and get the sum of all balances.
/// there's an option to map certain donor-directed wallets to be counted as slow wallets
/// Note: this may not be the "total supply", since there may be coins in other structs beside an account::balance, e.g escrowed in contracts.
pub fn populate_supply_stats_from_legacy(rec: &[LegacyRecoveryV6]) -> anyhow::Result<Supply> {
    let pb = ProgressBar::new(1000)
        .with_style(OLProgress::spinner())
        .with_message("calculating coin supply");
    pb.enable_steady_tick(Duration::from_millis(100));
    let mut supply = Supply {
        total: 0.0,
        normal: 0.0,
        validator: 0.0,
        slow_total: 0.0,
        slow_locked: 0.0,
        slow_validator_locked: 0.0,
        slow_unlocked: 0.0,
        donor_voice: 0.0,
        make_whole: 0.0,
        split_factor: 0.0,
        escrow_pct: 0.0,
        epoch_reward_base_case: 0.0,
        expected_user_balance: 0.0,
        expected_user_ratio: 0.0,
        expected_circulating: 0.0,
        expected_circulating_ratio: 0.0,
    };

    rec.iter().for_each(|r| {
        supply.inc_supply(r);
    });
    pb.finish_and_clear();
    Ok(supply)
}
