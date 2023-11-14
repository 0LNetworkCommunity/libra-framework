
module ol_framework::rewards {
  use std::vector;
  use diem_framework::coin::{Self, Coin};
  use diem_framework::stake;
  use diem_framework::system_addresses;
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::ol_account;

  // use diem_std::debug::print;

  friend diem_framework::epoch_boundary;

  const REWARD_VALIDATOR: u8 = 1;
  const REWARD_ORACLE: u8 = 2;
  const REWARD_MISC: u8 = 255;

  /// process a single reward
  /// root needs to have an owned coin already extracted. Not a mutable borrow.
  public(friend) fun process_single(root: &signer, addr: address, coin: Coin<LibraCoin>, reward_type: u8) {
    system_addresses::assert_ol(root);
    pay_reward(root, addr, coin, reward_type);
  }

  /// public api for processing bulk rewards
  /// convenience function to process payment for multiple recipients
  /// when the reward is the same for all recipients.
  // belt and suspenders pattern: guarded by friend, authorized by root, and public function separated from authorized private function. yes, it's redundant, see ol_move_coding_conventions.md
  public(friend) fun process_multiple(root: &signer, list: vector<address>, reward_per: u64, reward_budget: &mut Coin<LibraCoin>, reward_type: u8) {
    // note the mutable coin will be retuned to caller for them to do what
    // is necessary, including destroying if it is zero.

    system_addresses::assert_ol(root);
    process_recipients_impl(root, list, reward_per, reward_budget, reward_type);
  }



  /// process all the validators
  // belt and suspenders
  fun process_recipients_impl(root: &signer, list: vector<address>, reward_per: u64, reward_budget: &mut Coin<LibraCoin>, reward_type: u8) {
    // note the mutable coin will be retuned to caller for them to do what
    // is necessary, including destroying if it is zero.
    system_addresses::assert_ol(root);
    let i = 0;
    while (i < vector::length(&list)) {
      // split off the reward amount per validator from coin
      let user_coin = coin::extract(reward_budget, reward_per);
      pay_reward(root, *vector::borrow(&list, i), user_coin, reward_type);
      // TODO: emit payment event in stake.move
      i = i + 1;
    }

  }

  /// Pay one validator their reward
  /// belt and suspenders
  fun pay_reward(root: &signer, addr: address, coin: Coin<LibraCoin>, reward_type: u8) {
    // draw from transaction fees account
    // transaction fees account should have a subsidy from infra escrow
    // from start of epoch.
    // before we got here we should have checked that we have sufficient
    // funds to pay all members the proof-of-fee reward.
    // if we don't have enough funds, we should exit without abort.
    system_addresses::assert_ol(root);
    let amount = coin::value(&coin);
    ol_account::vm_deposit_coins_locked(root, addr, coin);

    if (reward_type == REWARD_VALIDATOR) {
      stake::emit_distribute_reward(root, addr, amount);
    } else if (reward_type == REWARD_ORACLE) {
      // oracle::emit_reward_event(addr, coin);
    } else if (reward_type == REWARD_MISC) {
      // misc::emit_reward_event(addr, coin);
    };
  }


  //////// TEST HELPERS ////////

  #[test_only]
  /// helps create a payment to a validator
  // belt and suspenders too
  public fun test_helper_pay_reward(root: &signer, addr: address, coin: Coin<LibraCoin>, reward_type: u8) {
    use ol_framework::testnet;
    testnet::assert_testnet(root);
    pay_reward(root, addr, coin, reward_type);
  }

  #[test_only]
  public fun test_process_recipients(root: &signer, list: vector<address>, reward_per: u64, coin: &mut Coin<LibraCoin>, reward_type: u8) {

    system_addresses::assert_ol(root);
    process_multiple(root, list, reward_per, coin, reward_type);

  }

}
