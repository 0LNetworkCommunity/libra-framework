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


    let all_unlocked = get_all_unlocked();
    // v7 accounts which have not migrated to slow wallets,
    // will not be counted in the slow_locked or unlocked
    let assumed_locked = total - community_endowments - future_pledges - all_unlocked;

    // TODO: add unlocked coins that CW have from advances

    (total, assumed_locked, community_endowments, future_pledges, all_unlocked)
  }

  #[view]
  // Unlocks come from two sources:
  // 1. Slow wallets, which have been unlocked by the slow wallet system
  // 2. Community wallets, which can borrow advances from their balance
  public fun get_all_unlocked(): u64 {
    // Note, since v8 we assume everything is locked.
    // So we calculated what has be transferred out of
    // slow wallets, and from community wallets
    let slow_unlocked = slow_wallet::get_lifetime_unlocked_supply();
    let cw_unlocked = get_cw_advanced();
    slow_unlocked + cw_unlocked
  }

  #[view]
  // What the market calls circulating supply would be all the supply that is immediately transferable. In OL this would translate to:
  // 1. all the coins in ordinary accounts
  // 2. the unlocked portion of slow wallets
  // 3. the credit available in community wallets
  public fun get_cw_advanced(): u64 {
    // get list of donor voice accounts
    // iterate over the list, and get the community_wallet_advances::Advances
    // lifetime withdrawals

    // returns the sum of all the lifetime withdrawals of all CWs
    0

  }

  public fun get_cw_remaining_credit(): u64 {
    // get list of donor voice accounts
    // iterate over the list, and get the community_wallet_advances::Advances
    // balance outstanding

    // returns the sum of all the balance outstanding of all CWs
    0
  }


  #[view]
  // What the market calls circulating supply would be all the supply that is immediately transferable. In OL this would translate to:
  // 1. all the coins in ordinary accounts
  // 2. the unlocked portion of slow wallets
  // 3. the credit available in community wallets
  public fun get_circulating(): u64 {
    let unlocked = get_all_unlocked();
    let available_credit = get_cw_remaining_credit();
    unlocked + available_credit
  }
}
