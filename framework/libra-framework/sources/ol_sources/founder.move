
/// Maintains the version number for the blockchain.
module ol_framework::founder {
  use std::signer;
  use ol_framework::vouch_score;

  friend diem_framework::vouch_txs;
  friend ol_framework::filo_migration;

  struct Founder has key {
    has_human_friends: bool
  }

  public(friend) fun migrate(user_sig: &signer) {
    if (!exists<Founder>(signer::address_of(user_sig))) {
      move_to<Founder>(user_sig, Founder {
        has_human_friends: false // ooh is's lonely at the top
      });
    }
  }

  // DANGER: open to any friend function
  public(friend) fun maybe_set_friendly_founder(user: address) acquires Founder {
    if (
      is_founder(user) &&
      vouch_score::is_voucher_score_valid(user)
    ) {
      let f = borrow_global_mut<Founder>(user);
      f.has_human_friends = true;
    }
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
