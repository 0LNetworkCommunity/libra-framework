/// Separate the vouch txs module
/// founder caused a dependency cycle

module ol_framework::vouch_txs {
  use ol_framework::vouch;
  use ol_framework::founder;

  public entry fun vouch_for(grantor: &signer, friend_account: address) {
    vouch::vouch_for(grantor, friend_account);
    founder::maybe_set_friendly_founder(friend_account);
  }

  public entry fun revoke(grantor: &signer, friend_account: address) {
    vouch::revoke(grantor, friend_account);
  }
}
