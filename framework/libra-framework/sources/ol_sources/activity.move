
/// Maintains the version number for the blockchain.
module ol_framework::activity {
  use std::signer;
  use std::error;
  use ol_framework::testnet;
  use diem_std::timestamp;

  friend ol_framework::filo_migration;
  friend ol_framework::genesis;
  friend ol_framework::ol_account;
  friend ol_framework::donor_voice_migration;
  friend diem_framework::transaction_validation;

  #[test_only]
  friend ol_framework::donor_voice_reauth;
  #[test_only]
  friend ol_framework::mock;
  #[test_only]
  friend ol_framework::test_filo_migration;


  //////// ERROR CODES ///////
  /// account not initialized on v8 chain
  const EACCOUNT_NOT_MIGRATED: u64 = 0;
  /// account has malformed timestamp
  const ETIMESTAMP_MALFORMED: u64 = 1;

  struct Activity has key {
    last_touch_usecs: u64,
    onboarding_usecs: u64,
  }

  /// Initialize the activity timestamp of a user
  public(friend) fun lazy_initialize(user: &signer) {
    let timestamp = timestamp::now_seconds();
    if (!is_initialized(signer::address_of(user))) {
      move_to<Activity>(user, Activity {
        last_touch_usecs: timestamp,
        onboarding_usecs: timestamp
      })
    }
  }

  /// Increment the activity timestamp of a user
  // NOTE: this serves to gate users who have not onboarded since v8 upgrade
  public(friend) fun increment(user: &signer) acquires Activity {
    is_initialized(signer::address_of(user));
    maybe_fix_malformed(user);

    let state = borrow_global_mut<Activity>(signer::address_of(user));
    state.last_touch_usecs = timestamp::now_seconds();
  }

  #[view]
  /// get the last activity timestamp of a user
  public fun get_last_activity_usecs(user: address): u64 acquires Activity {
    if (is_initialized(user)) {
      let state = borrow_global<Activity>(user);
      return state.last_touch_usecs
    };
    0
  }

  public(friend) fun maybe_onboard(user_sig: &signer) {
    // genesis accounts should not start at 0
    let onboarding_usecs = timestamp::now_seconds();
    if (onboarding_usecs == 0) {
      onboarding_usecs = 1;
    };

    if (!is_initialized(signer::address_of(user_sig))) {
      move_to<Activity>(user_sig, Activity {
        last_touch_usecs: 0, // how we identify if a users has used the account after a peer created it.
        onboarding_usecs,
      })
    }
  }

  /// migrate or heal a pre-v8 account
  public(friend) fun migrate(user_sig: &signer) acquires Activity {
    let addr = signer::address_of(user_sig);
    if (!is_initialized(addr)) {
      move_to<Activity>(user_sig, Activity {
        last_touch_usecs: 0,
        onboarding_usecs: 0,
      })
    };

    maybe_fix_malformed(user_sig);

    if (is_pre_v8(addr)) {
      // this is a pre-v8 account that might be malformed
      let state = borrow_global_mut<Activity>(addr);
      state.last_touch_usecs = 0;
      state.onboarding_usecs = 0;
    }
  }

  fun assert_initialized(user: address) {
    // check malformed accounts with microsecs
    assert!(exists<Activity>(user), error::invalid_state(EACCOUNT_NOT_MIGRATED));
  }

  /// some accounts with transactions immediately after the v8 upgrade
  /// may have a malformed timestamp
  // NOTE: make this an entry function to allow for
  // manual migration
  public entry fun maybe_fix_malformed(user: &signer) acquires Activity {
    assert_initialized(signer::address_of(user));
    if (!is_timestamp_secs(signer::address_of(user))) {
      let state = borrow_global_mut<Activity>(signer::address_of(user));
      state.onboarding_usecs = 0;
      // last_touch_usecs should be set in the transaction_validation
      // but we'll set it here to be safe
      state.last_touch_usecs = timestamp::now_seconds();
    }
  }

  /// check for edge cases in initialization during v8 migration
  fun is_timestamp_secs(acc: address): bool acquires Activity {
    // check if this is incorrectly in microsecs
    if (
      get_last_activity_usecs(acc) > 1_000_000_000_000 ||
      get_onboarding_usecs(acc) > 1_000_000_000_000
    ) {
      return false
    };

    true
  }


  #[view]
  // check if this is an account that has activity
  public fun has_ever_been_touched(user: address): bool acquires Activity {
    // I was beat, incomplete
    // I've been had, I was sad and blue
    // But you made me feel
    // Yeah, you made me feel
    // Shiny and new
    get_last_touch_usecs(user) > 0
  }



  #[view]
  public fun get_last_touch_usecs(user: address): u64 acquires Activity {
    assert_initialized(user);

    let state = borrow_global<Activity>(user);
    state.last_touch_usecs
  }

  #[view]
  public fun get_onboarding_usecs(user: address): u64 acquires Activity {
    // check malformed accounts with microsecs
    assert_initialized(user);
    let state = borrow_global<Activity>(user);
    state.onboarding_usecs
  }

  #[view]
  // check if the account activity struct is initialized
  // accounts that have been onboarded prior to V8, would not
  // have this struct.
  public fun is_initialized(user: address): bool {
    exists<Activity>(user)
  }

  #[view]
  // check the timestamp prior to v8 launch
  public fun is_pre_v8(user: address): bool acquires Activity {
    assert_initialized(user);

    if (testnet::is_testnet()) {
      return false
    };

    // catch edge cases of malformed timestamp
    // should be considered a pre-v8 for
    // purposes of triggering a migration
    if (!is_timestamp_secs(user)) {
      return true
    };

    get_onboarding_usecs(user) < 1_747_267_200
    // Date and time (GMT): Thursday, May 15, 2025 12:00:00 AM
  }




  #[view]
  // If the account is a founder/pre-v8 account has been migrated
  // then it would have an onboarding timestamp of 0
  public fun is_prehistoric(user: address): bool acquires Activity {
    get_onboarding_usecs(user) == 0
  }

  #[test_only]
  /// testnet help for framework account to mock activity
  public(friend) fun test_set_activity(framework: &signer, user: address, timestamp: u64) acquires Activity {
    testnet::assert_testnet(framework);

    let state = borrow_global_mut<Activity>(user);
    state.last_touch_usecs = timestamp;
  }

}
