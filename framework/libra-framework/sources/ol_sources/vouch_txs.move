

module ol_framework::vouch_txs {

  use ol_framework::vouch;

  public entry fun vouch_for(grantor: &signer, friend_account: address) {
    vouch::vouch_for(grantor, friend_account);
  }

  public entry fun revoke(grantor: &signer, friend_account: address) {
    vouch::revoke(grantor, friend_account);
  }
}
