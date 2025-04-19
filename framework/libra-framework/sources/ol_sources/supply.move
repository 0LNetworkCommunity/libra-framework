// utils for calculating supply statistics
module ol_framework::supply {
  use ol_framework::libra_coin;
  use ol_framework::slow_wallet;
  use ol_framework::donor_voice_txs;
  use ol_framework::pledge_accounts;

  #[view]

  /// Retrieves various supply statistics for Libra Coin.
  ///
  /// This function provides a consolidated view of different Libra Coin supply metrics,
  /// including the total supply, slow wallet locked amounts, donor voice allocations,
  /// pledged coins, and unlocked circulation.
  ///
  /// # Returns
  ///
  /// A tuple containing five values:
  /// * `total`: The total supply of Libra Coin in existence
  /// * `slow_locked`: Amount of coins locked in slow wallets
  /// * `donor_voice`: Coins allocated to community endowments via donor voice
  /// * `pledge`: Coins committed to future pledges
  /// * `unlocked`: Total coins that have been unlocked from slow wallets and are in circulation
  ///
  /// Note: Since v8, the system assumes all coins are initially locked, and this
  /// function calculates the unlocked supply based on transfers out of slow wallets
  /// and community wallets.

  public fun get_stats(): (u64, u64, u64, u64, u64) {
    let total = libra_coin::supply();
    let community_endowments = donor_voice_txs::get_dv_supply();
    let future_pledges = pledge_accounts::get_pledge_supply();
    // Note, since v8 we assume everything is locked.
    // So we calculated what has be transferred out of
    // slow wallets, and from community wallets
    let slow_unlocked = slow_wallet::get_lifetime_unlocked_supply();
    let slow_locked = slow_wallet::get_locked_supply();

    // TODO: add unlocked coins that CW have from advances

    (total, slow_locked, community_endowments, future_pledges, slow_unlocked)
  }
}
