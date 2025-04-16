
/// Maintains the version number for the blockchain.
module ol_framework::founder {
  use std::signer;

  // friend diem_framework::vouch;
  friend ol_framework::filo_migration;

  struct Founder has key {
    has_human_friends: bool
  }
  public entry fun migrate(user_sig: &signer) {
    if (!exists<Founder>(signer::address_of(user_sig))) {
      move_to<Founder>(user_sig, Founder {
        has_human_friends: false // ooh is's lonely at the top
      });
    }
  }

  public(friend) fun self_set_friendly(user_sig: &signer) acquires Founder {
    let f = borrow_global_mut<Founder>(signer::address_of(user_sig));
    f.has_human_friends = true;
  }

  #[view]
  // If the account is a founder/pre-v8 account has been migrated
  // then it would have an onboarding timestamp of 0
  public fun is_founder(user: address): bool {
    exists<Founder>(user)
  }

  #[view]
  // Bot or not
  public fun has_friends(user: address): bool acquires Founder {
    let f = borrow_global<Founder>(user);
    f.has_human_friends
  }

}
