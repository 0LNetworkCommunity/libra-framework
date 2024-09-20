module ol_framework::lockbox {
  use std::error;
  use std::fixed_point32::{Self, FixedPoint32};
  use std::signer;
  use std::vector;

  use diem_framework::coin::Coin;
  use diem_std::math64;
  use ol_framework::libra_coin::{Self, LibraCoin};

  // use diem_framework::timestamp;

  const ENOT_INITIALIZED: u64 = 1;

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

  // user init lockbox
  fun maybe_initialize(user: &signer) {
    if (!exists<SlowWalletV2>(signer::address_of(user))) {
      move_to(user, SlowWalletV2 {
        list: vector::empty()
      })
    }
  }

  // add to lockbox
  fun add_to_or_create_box(user: &signer, coin: Coin<LibraCoin>, duration: u64) acquires SlowWalletV2 {
    maybe_initialize(user);

    let user_addr = signer::address_of(user);
    let (found, idx) = idx_by_duration(user_addr, duration);

    let list = &mut borrow_global_mut<SlowWalletV2>(user_addr).list;

    if (found) {
      let box = vector::borrow_mut(list, idx);
      libra_coin::merge(&mut box.locked_coins, coin);
    } else {
      let new_lock = Lockbox {
        locked_coins: coin,
        duration,
        lifetime_unlocked: 0,
        last_unlock_timestamp: 0,
      };

      vector::push_back(list, new_lock);

    }

    // TODO: always sort the list by duration
  }



  #[view]
  public fun idx_by_duration(user_addr: address, duration: u64): (bool, u64) acquires SlowWalletV2 {
    assert!(exists<SlowWalletV2>(user_addr), error::invalid_state(ENOT_INITIALIZED));
    let list = &borrow_global<SlowWalletV2>(user_addr).list;
    // NOTE: there should only be one box per duration. TBD if there's a different use case to have duplicate lockbox durations.

    let len = vector::length(list);
    let i = 0;
    while (i < len) {
        let el = vector::borrow(list, i);
        if (el.duration == duration) {
            return (true, i)
        };
        i = i + 1;
    };

    (false, 0)
  }

  // self drip

  // balance

  // unlocked balance

  //////// UNIT TESTS ////////

  #[test_only]
  use diem_framework::debug::print;
  #[test_only]
  use ol_framework::mock;
  #[test_only]
  use ol_framework::ol_account;

  #[test]
  fun test_iter() {
    let t = vector::fold(DEFAULT_LOCKS, 0,  |r, e| r + e);
    print(&t);
    vector::for_each(DEFAULT_LOCKS, |e| print(&e));
  }


  #[test(framework = @0x1, bob_sig = @0x10002)]
  #[expected_failure(abort_code = 196609, location = ol_framework::lockbox)]
  fun lockbox_idx_fails_when_not_init(framework: &signer, bob_sig: &signer) acquires SlowWalletV2 {
    let bob_addr = signer::address_of(bob_sig);
    mock::ol_mint_to(framework, bob_addr, 123);
    let (_,_) = idx_by_duration(bob_addr, 1*12);
  }

  #[test(framework = @0x1, bob_sig = @0x10002)]
  fun creates_lockbox(framework: &signer, bob_sig: &signer) acquires SlowWalletV2 {
    let bob_addr = signer::address_of(bob_sig);
    mock::ol_mint_to(framework, bob_addr, 123);
    let coin = ol_account::withdraw(bob_sig, 23);

    add_to_or_create_box(bob_sig, coin, 1*12);

    // see if it exists
    let (found, idx) = idx_by_duration(bob_addr, 1*12);
    assert!(found, 7357002);
    assert!(idx == 0, 7357003);
  }

  #[test(framework = @0x1, bob_sig = @0x10002)]
  fun adds_to_lockbox(framework: &signer, bob_sig: &signer) acquires SlowWalletV2 {
    let bob_addr = signer::address_of(bob_sig);
    mock::ol_mint_to(framework, bob_addr, 123);
    let coin = ol_account::withdraw(bob_sig, 23);

    add_to_or_create_box(bob_sig, coin, 1*12);

    // see if it exists
    let (found, idx) = idx_by_duration(bob_addr, 1*12);
    assert!(found, 7357002);
    assert!(idx == 0, 7357003);

    let coin2 = ol_account::withdraw(bob_sig, 100);

    add_to_or_create_box(bob_sig, coin2, 1*12);

    // see if it exists
    let (found, idx) = idx_by_duration(bob_addr, 1*12);
    assert!(found, 7357002);
    assert!(idx == 0, 7357003);
  }

}
