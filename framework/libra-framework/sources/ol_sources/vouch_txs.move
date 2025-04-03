/// Separate the vouch txs module
/// founder caused a dependency cycle

module ol_framework::vouch_txs {
  use std::signer;
  use diem_framework::stake;
  use diem_framework::transaction_fee;
  use ol_framework::founder;
  use ol_framework::ol_account;
  use ol_framework::vouch;


  public entry fun vouch_for(grantor: &signer, friend_account: address) {
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
    vouch::revoke(grantor, friend_account);
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
