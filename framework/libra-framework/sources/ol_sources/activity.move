
/// Maintains the version number for the blockchain.
module ol_framework::activity {
  use std::signer;

  friend diem_framework::transaction_validation;

  struct Activity has key {
    timestamp: u64,
  }

  fun lazy_initialize(user: &signer, timestamp: u64) {
    if (!exists<Activity>(signer::address_of(user))) {
      move_to<Activity>(user, Activity {
        timestamp
      })
    }
  }
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
}
