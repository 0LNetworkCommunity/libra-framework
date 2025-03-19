
/// Maintains the version number for the blockchain.
module ol_framework::activity {
  use std::signer;
  use diem_std::timestamp;

  friend diem_framework::transaction_validation;
  friend ol_framework::ol_account;

  struct Activity has key {
    last_touch_usecs: u64,
    onboarding_usecs: u64,
  }

  public(friend) fun increment(user_sig: &signer, timestamp: u64) acquires Activity {
    // migrate old accounts
    // catch the case of existing "founder" accounts from prior to V8
    if (!exists<Activity>(signer::address_of(user_sig))) {
      migrate(user_sig, timestamp);
    } else {
      let state = borrow_global_mut<Activity>(signer::address_of(user_sig));
      state.last_touch_usecs = timestamp;
    }
  }

  fun migrate(user_sig: &signer, timestamp: u64) {
      move_to<Activity>(user_sig, Activity {
        last_touch_usecs: timestamp,
        onboarding_usecs: 0, // also how we identify pre-V8 "founder account",
      });

  }

  public(friend) fun maybe_onboard(user_sig: &signer){

    if (!exists<Activity>(signer::address_of(user_sig))) {
      move_to<Activity>(user_sig, Activity {
        last_touch_usecs: 0, // how we identify if a users has used the account after a peer created it.
        onboarding_usecs: timestamp::now_seconds(),
      })
    }
  }


  #[view]
  // if there user has been onboarded (since v8) but never transacted
  // they should have a last touch timestamp of 0.
  public fun has_ever_been_touched(user: address): bool acquires Activity {
    if (exists<Activity>(user)){
      let state = borrow_global<Activity>(user);
      return state.last_touch_usecs > 0
    };
    // I was beat, incomplete
    // I've been had, I was sad and blue
    // But you made me feel
    // Yeah, you made me feel
    // Shiny and new

    false
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
    let state = borrow_global<Activity>(user);
    state.onboarding_usecs == 0
  }

  #[test_only]
  public(friend) fun test_set_activity(framework: &signer, user: address, timestamp: u64) acquires Activity {
    diem_framework::system_addresses::assert_diem_framework(framework);
    let state = borrow_global_mut<Activity>(user);
    state.last_touch_usecs = timestamp;
  }
}
