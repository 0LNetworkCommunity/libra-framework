
/// Maintains the version number for the blockchain.
module ol_framework::activity {
  use std::signer;

  friend diem_framework::transaction_validation;
  friend diem_framework::ol_account;
  #[test_only]
  friend ol_framework::donor_voice_reauth;

  struct Activity has key {
    timestamp: u64,
  }

  /// Initialize the activity timestamp of a user
  fun lazy_initialize(user: &signer, timestamp: u64) {
    if (!exists<Activity>(signer::address_of(user))) {
      move_to<Activity>(user, Activity {
        timestamp
      })
    }
  }

  /// Increment the activity timestamp of a user
  public(friend) fun increment(user: &signer, timestamp: u64) acquires Activity {
    lazy_initialize(user, timestamp);

    let state = borrow_global_mut<Activity>(signer::address_of(user));
    state.timestamp = timestamp;
  }

  #[view]
  /// get the last activity timestamp of a user
  public fun get_last_activity_usecs(user: address): u64 acquires Activity {
    if (exists<Activity>(user)) {
      let state = borrow_global<Activity>(user);
      return state.timestamp
    };
    0
  }

  #[test_only]
  public(friend) fun test_set_activity(framework: &signer, user: address, timestamp: u64) acquires Activity {
    diem_framework::system_addresses::assert_diem_framework(framework);
    let state = borrow_global_mut<Activity>(user);
    state.timestamp = timestamp;
  }
}
