module ol_framework::lockbox {
  use std::error;
  use std::fixed_point32::{Self, FixedPoint32};
  use std::option::{Self, Option};
  use std::signer;
  use std::vector;
  use diem_framework::coin::Coin;
  use diem_std::math64;
  use ol_framework::libra_coin::{Self, LibraCoin};
  use ol_framework::date;

  // use diem_framework::debug::print;

  friend ol_framework::ol_account;

  /// SlowWalletV2 not initialized
  const ENOT_INITIALIZED: u64 = 1;

  /// No lockbox of this duration found
  const ENO_DURATION_FOUND: u64 = 2;

  /// List of durations incomplete
  const ELIST_INCOMPLETE: u64 = 3;

  /// Tried to drip twice in one day, silly rabbit.
  const ENO_DOUBLE_DIPPING: u64 = 4;

  const DEFAULT_LOCKS: vector<u64> = vector[1*12, 4*12, 8*12, 16*12, 20*12, 24*12, 28*12, 32*12];

  struct Lockbox has key, store {
    locked_coins: Coin<LibraCoin>,
    duration_type: u64,
    lifetime_deposits: u64,
    lifetime_unlocked: u64,
    last_unlock_timestamp: u64,
  }

  struct SlowWalletV2 has key {
    list: vector<Lockbox>
  }

  //////// GETTERS ////////
  fun get_daily_pct(box: &Lockbox): FixedPoint32  {
    let days = math64::mul_div(box.duration_type, 365, 12);
    fixed_point32::create_from_rational(1, days)
  }

  fun calc_daily_drip(box: &Lockbox): u64  {
    let daily_pct = get_daily_pct(box);
    let value = libra_coin::value(&box.locked_coins);
    fixed_point32::multiply_u64(value, daily_pct)
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
  public(friend) fun add_to_or_create_box(user: &signer, coin: Coin<LibraCoin>, duration_type: u64) acquires SlowWalletV2 {
    maybe_initialize(user);

    let user_addr = signer::address_of(user);
    let (found, idx) = idx_by_duration(user_addr, duration_type);

    let list = &mut borrow_global_mut<SlowWalletV2>(user_addr).list;

    if (found) {
      let box = vector::borrow_mut(list, idx);
      libra_coin::merge(&mut box.locked_coins, coin);
    } else {
      let new_lock = Lockbox {
        locked_coins: coin,
        duration_type,
        lifetime_deposits: 0,
        lifetime_unlocked: 0,
        last_unlock_timestamp: 0,
      };

      vector::push_back(list, new_lock);

    }

    // TODO: always sort the list by duration_type
  }



  #[view]
  public fun idx_by_duration(user_addr: address, duration_type: u64): (bool, u64) acquires SlowWalletV2 {
    assert!(exists<SlowWalletV2>(user_addr), error::invalid_state(ENOT_INITIALIZED));
    let list = &borrow_global<SlowWalletV2>(user_addr).list;
    // NOTE: there should only be one box per duration_type. TBD if there's a different use case to have duplicate lockbox durations.

    let len = vector::length(list);
    let i = 0;
    while (i < len) {
        let el = vector::borrow(list, i);
        if (el.duration_type == duration_type) {
            return (true, i)
        };
        i = i + 1;
    };

    (false, 0)
  }

  fun get_list_durations_holding(user_addr: address): vector<u64> acquires SlowWalletV2 {
    assert!(exists<SlowWalletV2>(user_addr), error::invalid_state(ENOT_INITIALIZED));
    let list = &borrow_global<SlowWalletV2>(user_addr).list;
    let all_durations = vector[];
    let len = vector::length(list);
    let i = 0;
    while (i < len) {
        let el = vector::borrow(list, i);
        vector::push_back(&mut all_durations, el.duration_type);
        i = i + 1;
    };
    assert!(vector::length(&all_durations) == len, error::invalid_state(ELIST_INCOMPLETE));

    all_durations
  }

  // drip one duration
  fun withdraw_drip_impl(user: &signer, idx: u64): Option<Coin<LibraCoin>> acquires SlowWalletV2 {
    let user_addr = signer::address_of(user);

    let list = &mut borrow_global_mut<SlowWalletV2>(user_addr).list;
    let box = vector::borrow_mut(list, idx);
    let (start_today, _) = date::start_of_day_seconds();
    // abort if tried to drip on same unix day
    assert!(start_today > box.last_unlock_timestamp, error::invalid_state(ENO_DOUBLE_DIPPING));

    let drip_value = calc_daily_drip(box);

    if (drip_value == 0) return option::none();

    box.last_unlock_timestamp = start_today;

    let dripped_coins = libra_coin::extract(&mut box.locked_coins, drip_value);

    option::some(dripped_coins)
  }

  /// drips all lockboxes, callable by ol_account
  public(friend) fun withdraw_drip_all(user: &signer): Option<Coin<LibraCoin>> acquires SlowWalletV2 {
    let list = &mut borrow_global_mut<SlowWalletV2>(signer::address_of(user)).list;
    assert!(!vector::is_empty(list), ENO_DURATION_FOUND);

    let coin_opt = option::none<Coin<LibraCoin>>();

    let len = vector::length(list);
    let i = 0;
    while (i < len) {
        let box_coins_opt = withdraw_drip_impl(user, i);
        if (option::is_none(&box_coins_opt)) {
          option::destroy_none(box_coins_opt);
          continue
        };

        let c = option::extract(&mut box_coins_opt);
        option::destroy_none(box_coins_opt);

        if (option::is_none(&coin_opt) ){
          option::fill(&mut coin_opt, c);
        } else {
          let all_coins = option::borrow_mut(&mut coin_opt);
          libra_coin::merge(all_coins, c);
        };

        i = i + 1;
    };
    coin_opt
  }




  #[view]
  /// balance for one duration_type's box
  public fun balance_duration(user_addr: address, duration_type: u64): u64 acquires SlowWalletV2 {
    if (!exists<SlowWalletV2>(user_addr)) return 0;
    let (found, idx) = idx_by_duration(user_addr, duration_type);
    if (!found) {
      return 0
    };

    let list = &borrow_global<SlowWalletV2>(user_addr).list;
    let box = vector::borrow(list, idx);
    libra_coin::value(&box.locked_coins)
  }

  #[view]
  /// balance of all lockboxes
  public fun balance_all(user_addr: address): u64 acquires SlowWalletV2 {
    let sum = 0;
    if (!exists<SlowWalletV2>(user_addr)) return sum;
    let list = &borrow_global<SlowWalletV2>(user_addr).list;
    let len = vector::length(list);
    let i = 0;
    while (i < len) {
        let el = vector::borrow(list, i);
        sum = sum + libra_coin::value(&el.locked_coins);
        i = i + 1;
    };

    sum
  }


  //////// UNIT TESTS ////////
  #[test_only]
  use diem_framework::coin;

  #[test_only]
  fun test_setup(framework: &signer, amount: u64): Coin<LibraCoin> {
    let (burn_cap, mint_cap) = libra_coin::initialize_for_test(framework);
    let c = coin::test_mint(amount, &mint_cap);
    coin::destroy_mint_cap(mint_cap);
    coin::destroy_burn_cap(burn_cap);
    c
  }


  #[test(bob_sig = @0x10002)]
  #[expected_failure(abort_code = 196609, location = ol_framework::lockbox)]
  fun lockbox_idx_aborts_when_not_init(bob_sig: &signer) acquires SlowWalletV2 {
    let bob_addr = signer::address_of(bob_sig);
    let (_,_) = idx_by_duration(bob_addr, 1*12);
  }

  #[test(framework = @0x1, bob_sig = @0x10002)]
  fun creates_lockbox(framework: &signer, bob_sig: &signer) acquires SlowWalletV2 {
    let bob_addr = signer::address_of(bob_sig);

    let coin = test_setup(framework, 23);

    add_to_or_create_box(bob_sig, coin, 1*12);

    // see if it exists
    let (found, idx) = idx_by_duration(bob_addr, 1*12);
    assert!(found, 7357002);
    assert!(idx == 0, 7357003);

    let balance_one = balance_duration(bob_addr, 1*12);
    assert!(balance_one == 23, 7357004);
    let balanace_all = balance_all(bob_addr);
    assert!(balanace_all == 23, 7357005);
  }

  #[test(framework = @0x1, bob_sig = @0x10002)]
  fun adds_to_lockbox(framework: &signer, bob_sig: &signer) acquires SlowWalletV2 {
    let bob_addr = signer::address_of(bob_sig);
    let coin = test_setup(framework, 123);
    let split_coin = libra_coin::extract(&mut coin, 23);

    add_to_or_create_box(bob_sig, split_coin, 1*12);

    // see if it exists
    let (found, idx) = idx_by_duration(bob_addr, 1*12);
    assert!(found, 7357002);
    assert!(idx == 0, 7357003);

    let bal = balance_all(bob_addr);
    assert!(bal == 23, 7357004);

    // remainder of coin should be 100
    add_to_or_create_box(bob_sig, coin, 1*12);

    // see if it exists
    let (found, idx) = idx_by_duration(bob_addr, 1*12);
    assert!(found, 7357005);
    assert!(idx == 0, 7357006);

    let bal2 = balance_all(bob_addr);
    assert!(bal2 == 123, 7357007);
  }

}
