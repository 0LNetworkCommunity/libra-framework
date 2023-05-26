
module ol_framework::rewards {
  use aptos_framework::system_addresses;
  use ol_framework::gas_coin::GasCoin;
  use aptos_framework::coin::{Self, Coin};
  use std::vector;
  // use aptos_std::debug::print;

  friend aptos_framework::reconfiguration;

  /// public api for processing rewards
  // belt and suspenders pattern: guarded by friend, authorized by root, and public function separated from authorized private function. yes, it's redundant, see ol_move_coding_conventions.md
  public(friend) fun process_validators(root: &signer, list: vector<address>, reward_per: u64, reward_budget: &mut Coin<GasCoin>) {
    // note the mutable coin will be retuned to caller for them to do what
    // is necessary, including destroying if it is zero.

    system_addresses::assert_ol(root);
    process_validators_impl(root, list, reward_per, reward_budget);
  }
  
  /// process all the validators
  // belt and suspenders
  fun process_validators_impl(root: &signer, list: vector<address>, reward_per: u64, reward_budget: &mut Coin<GasCoin>) {
    // note the mutable coin will be retuned to caller for them to do what
    // is necessary, including destroying if it is zero.
    system_addresses::assert_ol(root);
    let i = 0;
    while (i < vector::length(&list)) {
      // split off the reward amount per validator from coin
      let user_coin = coin::extract(reward_budget, reward_per);
      pay_reward(root, *vector::borrow(&list, i), user_coin);
      // TODO: emit payment event in stake.move
      i = i + 1;
    }

  }

  /// Pay one validator their reward
  /// belt and suspenders
  fun pay_reward(root: &signer, addr: address, coin: Coin<GasCoin>) {
  // draw from transaction fees account
  // transaction fees account should have a subsidy from infra escrow
  // from start of epoch.
  // before we got here we should have checked that we have sufficient
  // funds to pay all members the proof-of-fee reward.
  // if we don't have enough funds, we should exit without abort.
  system_addresses::assert_ol(root);
// print(&10101);
  coin::deposit(addr, coin);


  }


  //////// TEST HELPERS ////////
  #[test_only]
  use ol_framework::testnet;

  #[test_only]
  /// helps create a payment to a validator
  // belt and suspenders too
  public fun test_helper_pay_reward(root: &signer, addr: address, coin: Coin<GasCoin>) {
    testnet::assert_testnet(root);
    pay_reward(root, addr, coin);
  }

}