use indicatif::ProgressBar;
use libra_types::legacy_types::legacy_recovery_v6::LegacyRecoveryV6;
use libra_types::ol_progress::OLProgress;
use std::time::Duration;

// #[derive(Debug, Clone, Args)]
// pub struct SupplySettings {
//     #[clap(long)]
//     /// what is the final supply units to be split to. This is an "unscaled" number, meaning you should use the expected integer units of the coin, without the decimal precision.
//     pub target_supply: f64,
//     #[clap(long)]
//     /// for calculating escrow, what's the desired percent to future uses
//     pub target_future_uses: f64,
//     #[clap(long)]
//     /// for calculating base case validator reward
//     pub years_escrow: u64,
//     #[clap(long)]
//     /// for future uses calc, are there any donor directed wallets which require mapping to slow wallets
//     pub map_dd_to_slow: Vec<LegacyAddress>,
// }

// impl Default for SupplySettings {
//     fn default() -> Self {
//         Self {
//             target_supply: 100_000_000_000.0,
//             target_future_uses: 0.0,
//             years_escrow: 10,
//             map_dd_to_slow: vec![],
//         }
//     }
// }

// impl SupplySettings {
//     // convert to the correct coin scaling
//     pub fn scale_supply(&self) -> f64 {
//         self.target_supply * 10f64.powf(ONCHAIN_DECIMAL_PRECISION.into())
//     }
// }
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
    //     // returns the ratios (split_factor, escrow_pct)
    //     pub fn set_ratios_from_settings(&mut self, settings: &SupplySettings) -> anyhow::Result<()> {
    //         // NOTE IMPORTANT: the CLI receives an unscaled integer number. And it should be scaled up to the Movevm decimal precision being used: 10^6
    //         self.split_factor = settings.scale_supply() / self.total;

    //         // get the coin amount of chosen future uses
    //         let target_future_uses = settings.target_future_uses * self.total;
    //         // excluding donor directed, how many coins are remaining to fund
    //         let remaining_to_fund = target_future_uses - self.donor_voice;
    //         // what the ratio of remaining to fund, compared to validator_slow_locked
    //         self.escrow_pct = remaining_to_fund / self.slow_validator_locked;
    //         self.epoch_reward_base_case =
    //             remaining_to_fund / (365 * 100 * settings.years_escrow) as f64; // one hundred validators over 7 years every day. Note: discussed elsewhere: if this is an over estimate, the capital gets returned to community by the daily excess burn.
    //         self.expected_user_balance = self.split_factor
    //             * (
    //                 self.normal +
    //           (self.slow_total - self.slow_validator_locked ) + // remove vals
    //           (self.slow_validator_locked * (1.0 - self.escrow_pct))
    //                 // add back vals after escrow
    //             );
    //         let total_scaled = self.total * self.split_factor;
    //         self.expected_user_ratio = self.expected_user_balance / total_scaled;

    //         self.expected_circulating = self.split_factor * (self.normal + self.slow_unlocked);
    //         self.expected_circulating_ratio = self.expected_circulating / total_scaled;

    //         Ok(())
    //     }
    // }

    fn inc_supply(&mut self, r: &LegacyRecoveryV6) -> &mut Self {
        // get balances
        let user_total: f64 = match &r.balance {
            Some(b) => b.coin as f64,
            None => 0.0,
        };
        self.total += user_total;

        // // get loose coins in make_whole
        // if let Some(mk) = &r.make_whole {
        //     let user_credits = mk.credits.iter().fold(0, |sum, e| {
        //         if !e.claimed {
        //             return sum + e.coins.value;
        //         }
        //         sum
        //     }) as f64;

        //     self.total += user_credits;
        //     self.make_whole += user_credits;
        // }

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
