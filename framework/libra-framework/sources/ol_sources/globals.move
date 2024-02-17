///////////////////////////////////////////////////////////////////
// 0L Module
// Globals
///////////////////////////////////////////////////////////////////

/// # Summary
/// This module provides global constants
module ol_framework::globals {
    use ol_framework::testnet;
    use diem_std::math64;

    /// Global constants determining validator settings & requirements
    /// Some constants need to be changed based on environment; dev, testing, prod.
    /// epoch by a miner to remain compliant
    struct GlobalConstants has drop {
      // For validator set.
      epoch_length: u64,
      val_set_at_genesis: u64,
      subsidy_ceiling_gas: u64,
      vdf_difficulty_baseline: u64,
      vdf_security_baseline: u64,
      epoch_mining_thres_lower: u64,
      epoch_mining_thres_upper: u64,
      epoch_slow_wallet_unlock: u64,
      min_blocks_per_epoch: u64,
      validator_vouch_threshold: u64,
      signing_threshold_pct: u64,
    }

    const COIN_DECIMAL_PLACES: u8 = 6; // Or 10^6, 1 coin is 1_000_000 units in the database. Any human display needs to consider this scaling factor.


    /// Get the epoch length
    public fun get_epoch_length(): u64 {
       get_constants().epoch_length
    }

    /// Get max validator per epoch
    public fun get_val_set_at_genesis(): u64 {
       get_constants().val_set_at_genesis
    }

    #[view]
    /// Get the epoch length
    public fun get_coin_scaling_factor(): u64 {
       math64::pow(10, (COIN_DECIMAL_PLACES as u64))
    }

    public fun get_coin_decimal_places(): u8 {
      COIN_DECIMAL_PLACES
    }

    /// Get max validator per epoch
    public fun get_subsidy_ceiling_gas(): u64 {
       get_constants().subsidy_ceiling_gas
    }

    /// Get the current vdf_difficulty
    public fun get_vdf_difficulty_baseline(): u64 {
      get_constants().vdf_difficulty_baseline
    }

    /// Get the current vdf_difficulty
    public fun get_vdf_security_baseline(): u64 {
      get_constants().vdf_security_baseline
    }

    /// Get the mining threshold
    public fun get_epoch_mining_thres_lower(): u64 {
      get_constants().epoch_mining_thres_lower
    }

    /// Get the mining threshold
    public fun get_epoch_mining_thres_upper(): u64 {
      get_constants().epoch_mining_thres_upper
    }

    /// Get the mining threshold
    public fun get_unlock(): u64 {
      get_constants().epoch_slow_wallet_unlock
    }

    /// Get the mining threshold
    public fun get_min_blocks_epoch(): u64 {
      get_constants().min_blocks_per_epoch
    }

    /// Get the threshold for unrelated vouchers per validator
    public fun get_validator_vouch_threshold(): u64 {
      get_constants().validator_vouch_threshold
    }

    /// Get the threshold of number of signed blocks in an epoch per validator
    public fun get_signing_threshold(): u64 {
      get_constants().signing_threshold_pct
    }

    /// Get the constants for the current network
    fun get_constants(): GlobalConstants {

      if (testnet::is_testnet()) {
        return GlobalConstants {
          epoch_length: 60, // seconds
          val_set_at_genesis: 10,
          subsidy_ceiling_gas: 296 * get_coin_scaling_factor(),
          vdf_difficulty_baseline: 100,
          vdf_security_baseline: 512,
          epoch_mining_thres_lower: 2, // many tests depend on two proofs because
                                       // the test harness already gives one at
                                       // genesis to validators
          epoch_mining_thres_upper: 1000, // upper bound unlimited
          epoch_slow_wallet_unlock: 10,
          min_blocks_per_epoch: 0,
          validator_vouch_threshold: 0,
          signing_threshold_pct: 3,
        }
      };

      if (testnet::is_staging_net()) {
        // All numbers like MAINNET except shorter epochs of 30 mins
        // and minimum mining of 1 proof
        return GlobalConstants {
          epoch_length: 60 * 30, // 30 mins, enough for a hard miner proof.
          val_set_at_genesis: 100, // max expected for BFT limits.
          // See DiemVMConfig for gas constants:
          // Target max gas units per transaction 100000000
          // target max block time: 2 secs
          // target transaction per sec max gas: 20
          // uses "scaled representation", since there are no decimals.
          subsidy_ceiling_gas: 8640000 * get_coin_scaling_factor(), // subsidy amount assumes 24 hour epoch lengths. Also needs to be adjusted for coin_scale the onchain representation of human readable value.
          vdf_difficulty_baseline: 120000000, // wesolowski proof, new parameters. Benchmark available in docs/delay_tower/benchmarking
          vdf_security_baseline: 512,
          // NOTE Reviewers: this goes back to v5 params since the VDF cryptograpy will actually not be changed
          epoch_mining_thres_lower: 1, // lower bound, allows for some operator error
          epoch_mining_thres_upper: 72, // upper bound enforced at 20 mins per  proof
          epoch_slow_wallet_unlock: 1000 * get_coin_scaling_factor(), // approx 10 years for largest accounts in genesis.
          min_blocks_per_epoch: 10000,
          validator_vouch_threshold: 2, // Production must be more than 1 vouch validator (at least 2)
          signing_threshold_pct: 3,
        }
      } else {
        return GlobalConstants {
          epoch_length: 60 * 60 * 24, // approx 24 hours at 1.4 vdf_proofs/sec
          val_set_at_genesis: 100, // max expected for BFT limits.
          // See DiemVMConfig for gas constants:
          // Target max gas units per transaction 100000000
          // target max block time: 2 secs
          // target transaction per sec max gas: 20
          // uses "scaled representation", since there are no decimals.
          subsidy_ceiling_gas: 8640000 * get_coin_scaling_factor(), // subsidy amount assumes 24 hour epoch lengths. Also needs to be adjusted for coin_scale the onchain representation of human readable value.
          vdf_difficulty_baseline: 120000000, // wesolowski proof, new parameters. Benchmark available in docs/delay_tower/benchmarking
          vdf_security_baseline: 512,
          // NOTE Reviewers: this goes back to v5 params since the VDF cryptograpy will actually not be changed
          epoch_mining_thres_lower: 7, // lower bound, allows for some operator error
          epoch_mining_thres_upper: 72, // upper bound enforced at 20 mins per  proof
          epoch_slow_wallet_unlock: 1000 * get_coin_scaling_factor(), // approx 10 years for largest accounts in genesis.
          min_blocks_per_epoch: 10000,
          validator_vouch_threshold: 2, // Production must be more than 1 vouch validator (at least 2)
          signing_threshold_pct: 3,
        }
      }
    }
}