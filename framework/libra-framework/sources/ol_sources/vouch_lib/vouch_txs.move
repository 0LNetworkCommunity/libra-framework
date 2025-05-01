/// Separate the vouch txs module
/// to prevent dependency cycles

module ol_framework::vouch_txs {
  use std::signer;
  use diem_framework::stake;
  use diem_framework::transaction_fee;
  use ol_framework::founder;
  use ol_framework::ol_account;
  use ol_framework::page_rank_lazy;
  use ol_framework::vouch;
  use ol_framework::vouch_limits;

  public entry fun vouch_for(grantor: &signer, friend_account: address) {
    let grantor_addr = signer::address_of(grantor);
    vouch_limits::assert_under_limit(grantor_addr, friend_account);
    page_rank_lazy::mark_as_stale(friend_account);
    vouch::vouch_for(grantor, friend_account);
    maybe_debit_validator_cost(grantor, friend_account);
    founder::maybe_set_friendly_founder(friend_account);
  }

  public entry fun revoke(grantor: &signer, friend_account: address) {
    // You've got a lotta nerve to say you are my friend
    // When I was down you just stood there grinnin'
    // You've got a lotta nerve to say you got a helping hand to lend
    // You just want to be on the side that's winnin'

    // You say I let you down, ya know its not like that
    // If you're so hurt, why then don't you show it?
    // You say you've lost your faith, but that's not where its at
    // You have no faith to lose, and ya know it
    vouch_limits::assert_revoke_limit(signer::address_of(grantor));
    vouch::revoke(grantor, friend_account);
    page_rank_lazy::mark_as_stale(friend_account);
  }

  public entry fun clean_expired(user_sig: &signer) {
    vouch::garbage_collect_expired(signer::address_of(user_sig));
  }

  /// validators vouching has a cost
  // this fee is paid to the system, cannot be reclaimed
  // TODO: refactor validator vouch into own module
  fun maybe_debit_validator_cost(grantor: &signer, friend_account: address)  {
    if (
      stake::is_valid(friend_account) &&
      stake::is_valid(signer::address_of(grantor))
    ) {
    let price = vouch::get_vouch_price();
      if (price > 0) {
        let vouch_cost = ol_account::withdraw(grantor, price);
        transaction_fee::user_pay_fee(grantor, vouch_cost);
      };
    }
  }
}
