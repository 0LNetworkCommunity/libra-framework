module ol_framework::lockbox {
  use diem_framework::coin::Coin;
  use ol_framework::libra_coin::LibraCoin;
  use diem_std::math64;
  use std::fixed_point32::{Self, FixedPoint32};

  #[test_only]
  use diem_framework::debug::print;
  #[test_only]
  use std::vector;
  #[test_only]
  use ol_framework::mock;

  const DEFAULT_LOCKS: vector<u64> = vector[1*12, 4*12, 8*12, 16*12, 20*12, 24*12, 28*12, 32*12];

  struct Lockbox has key, store {
    locked_coins: Coin<LibraCoin>,
    duration: u64,
    lifetime_unlocked: u64,
    last_unlock_timestamp: u64,
  }

  struct SlowWalletV2 has key {
    list: vector<Lockbox>
  }

  //////// GETTERS ////////
  fun get_daily_pct(box: &Lockbox): FixedPoint32  {
    let days = math64::mul_div(box.duration, 365, 12);
    fixed_point32::create_from_rational(1, days)
  }

  // add to lockbox
  // fun add(user: &signer, coin: Coin<LibraCoin>, duration: u8) {

  // }

  // self drip

  // balance

  // unlocked balance

  #[test]
  fun test_iter() {
    let t = vector::fold(DEFAULT_LOCKS, 0,  |r, e| r + e);
    print(&t);
    vector::for_each(DEFAULT_LOCKS, |e| print(&e));

  }

  #[test(framework = @0x1, bob = @0x10002)]
  fun test_init_lockbox(framework: &signer, bob: address) {

    mock::ol_mint_to(framework, bob, 123);
  }

}
