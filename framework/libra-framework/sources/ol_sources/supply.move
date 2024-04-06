// utils for calculating supply statistics
module ol_framework::supply {
  use ol_framework::libra_coin;
  use ol_framework::slow_wallet;
  use ol_framework::donor_voice_txs;
  use ol_framework::pledge_accounts;

  #[view]
  /// Convenience function to return supply statistics
  ///@returns (total, slow_locked, donor_voice, pledge, unlocked)
  public fun get_stats(): (u64, u64, u64, u64, u64) {
    let total = libra_coin::supply();
    let slow_locked = slow_wallet::get_locked_supply();
    let donor_voice = donor_voice_txs::get_dv_supply();
    let pledge = pledge_accounts::get_pledge_supply();
    let unlocked = total - slow_locked - donor_voice - pledge;

    (total, slow_locked, donor_voice, pledge, unlocked)
  }
}
