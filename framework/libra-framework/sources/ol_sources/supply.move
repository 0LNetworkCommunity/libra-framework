// utils for calculating supply statistics
module ol_framework::supply {
  use ol_framework::libra_coin;
  use ol_framework::slow_wallet;
  use ol_framework::donor_voice_txs;

  #[view]
  ///
  ///@returns (total, slow_locked, donor_voice, pledged, unlocked)
  ///
  public fun get_stats(): (u64, u64, u64, u64, u64) {
    // let total = 0;
    // let slow_locked = 0;
    // let donor_voice = 0;
    let pledged = 0;
    let unlocked = 0;

    let total = libra_coin::supply();
    let slow_locked = slow_wallet::get_locked_supply();
    let donor_voice = donor_voice_txs::get_dv_supply();


    (total, slow_locked, donor_voice, pledged, unlocked)
  }
}
