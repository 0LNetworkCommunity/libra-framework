
/// Maintains the version number for the blockchain.
module ol_framework::activity {
  use std::signer;
  use diem_std::timestamp;

  friend ol_framework::filo_migration;
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
  const EACCOUNT_MALFORMED: u64 = 1;

  struct Activity has key {
    last_touch_usecs: u64,
    onboarding_usecs: u64,
  }

  /// Initialize the activity timestamp of a user
  public(friend) fun lazy_initialize(user: &signer, timestamp: u64) {
    if (!exists<Activity>(signer::address_of(user))) {
      move_to<Activity>(user, Activity {
        last_touch_usecs: timestamp,
        onboarding_usecs: timestamp
      })
    }
  }

  /// Increment the activity timestamp of a user
  public(friend) fun increment(user: &signer, timestamp: u64) acquires Activity {
    lazy_initialize(user, timestamp);

    let state = borrow_global_mut<Activity>(signer::address_of(user));
    state.last_touch_usecs = timestamp;
  }

  #[view]
  /// get the last activity timestamp of a user
  public fun get_last_activity_usecs(user: address): u64 acquires Activity {
    if (exists<Activity>(user)) {
      let state = borrow_global<Activity>(user);
      return state.last_touch_usecs
    };
    0
  }

  fun migrate(user_sig: &signer, timestamp: u64) {
      move_to<Activity>(user_sig, Activity {
        last_touch_usecs: timestamp,
        onboarding_usecs: 0, // also how we identify pre-V8 "founder account",
      });
  }

  public(friend) fun maybe_onboard(user_sig: &signer){

    // genesis accounts should not start at 0
    let onboarding_usecs = timestamp::now_seconds();
    if (onboarding_usecs == 0) {
      onboarding_usecs = 1;
    };

    if (!exists<Activity>(signer::address_of(user_sig))) {
      move_to<Activity>(user_sig, Activity {
        last_touch_usecs: 0, // how we identify if a users has used the account after a peer created it.
        onboarding_usecs,
      })
    }
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
    let state = borrow_global<Activity>(user);
    state.last_touch_usecs
  }

  #[view]
  public fun get_onboarding_usecs(user: address): u64 acquires Activity {
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
  // If the account is a founder/pre-v8 account has been migrated
  // then it would have an onboarding timestamp of 0
  public fun is_prehistoric(user: address): bool acquires Activity {
    get_onboarding_usecs(user) == 0
  }

  #[test_only]
  /// testnet help for framework account to mock activity
  public(friend) fun test_set_activity(framework: &signer, user: address, timestamp: u64) acquires Activity {
    ol_framework::testnet::assert_testnet(framework);

    let state = borrow_global_mut<Activity>(user);
    state.last_touch_usecs = timestamp;
  }

}
