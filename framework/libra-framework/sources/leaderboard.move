/// Validator track record
/// NOTE: other reputational metrics exist on jail.move
module diem_framework::leaderboard {

  use std::vector;
  use std::signer;
  use diem_framework::system_addresses;

  // NOTE: reputation will be a work in progress. As a dev strategy, let's break
  // up the structs into small atomic pieces so we can increment without
  // needing to migrate state when we imlpement new metrics.


   friend ol_framework::epoch_boundary;
   friend ol_framework::genesis;

  // Who is in the Top 10 net proposals at end of last epoch.
  // Note: in 0L validators all have equal weight, so the net proposals are a
  // result of validator performance, and network topology
  // global state on 0x1
  struct TopTen has key {
    list: vector<address>
  }

  /// Count total epochs participating as validator
  /// count both successes and fails
  struct TotalScore has key {
    success: u64,
    fail: u64
  }

  // How many epochs has this validator been in the set successfully without
  // interruption
  // resets on the first jail
  struct WinStreak has key {
    value: u64
  }

  // The count of epochs the validator has been in the Top Ten by net proposal count
  struct TopTenStreak has key {
    value: u64
  }

  public entry fun init_player(sig: &signer) {
    let addr = signer::address_of(sig);
    if (!exists<TotalScore>(addr)) {
      move_to(sig, TotalScore {
        success: 0,
        fail: 0
      })
    };

    if (!exists<WinStreak>(addr)) {
      move_to(sig, WinStreak {
        value: 0
      })
    };

    if (!exists<TopTenStreak>(addr)) {
      move_to(sig, TopTenStreak {
        value: 0
      })
    }
  }

  /// genesis and migration initializer
  public(friend) fun initialize(framework: &signer) {

    if (!exists<TopTen>(@diem_framework)) {
      move_to(framework, TopTen {
        list: vector::empty()
      })
    }
  }

  /// sets the top 10
  public(friend) fun set_top_ten(framework: &signer, list: vector<address>) acquires TopTen, TopTenStreak {
    system_addresses::assert_diem_framework(framework);
    let state = borrow_global_mut<TopTen>(@diem_framework);

    // increament winning streak if again in the Top 10
    vector::for_each(state.list, |acc| {
      let state = borrow_global_mut<TopTenStreak>(acc);

      if (vector::contains(&list, &acc)) {
        state.value = state.value + 1;
      } else {
        // reset streak
        state.value = 0;
      }
    });

    state.list = list
  }


  /// The framework signer can increment the total stats for the validator at
  /// epoch boundaries.
  public(friend) fun increment_total(framework: &signer, acc: address, success: bool) acquires TotalScore {
    system_addresses::assert_diem_framework(framework);
    let state = borrow_global_mut<TotalScore>(acc);
    if (success) {
      state.success = state.success + 1;
    } else {
      state.fail = state.fail + 1;
    }
  }

  /// The framework signer can increment the total stats for the validator at
  /// epoch boundaries.
  public(friend) fun increment_streak(framework: &signer, acc: address) acquires WinStreak {
    system_addresses::assert_diem_framework(framework);
    let state = borrow_global_mut<WinStreak>(acc);
    state.value = state.value + 1;
  }

  /// The framework signer can increment the total stats for the validator at
  /// epoch boundaries.
  public(friend) fun reset_streak(framework: &signer, acc: address) acquires WinStreak {
    system_addresses::assert_diem_framework(framework);
    let state = borrow_global_mut<WinStreak>(acc);
    state.value = 0;
  }

  //////// GETTERS ////////
  #[view]
  /// Get totals (success epochs, fail epochs)
  public fun get_total(acc: address): (u64, u64) acquires TotalScore {
    let state = borrow_global<TotalScore>(acc);
    (state.success, state.fail)
  }

  #[view]
  /// Get win streak (success epochs, fail epochs)
  public fun get_streak(acc: address): u64 acquires WinStreak {
    let state = borrow_global<WinStreak>(acc);
    state.value
  }

  #[view]
  /// Get win streak (success epochs, fail epochs)
  public fun get_topten_streak(acc: address): u64 acquires TopTenStreak {
    let state = borrow_global<TopTenStreak>(acc);
    state.value
  }
}
