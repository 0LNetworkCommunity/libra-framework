/// Module that maintains founder status for pre-v8 accounts and verifies their connections
/// in the trust network to prevent sybil attacks.
module ol_framework::founder {
  use ol_framework::page_rank_lazy;
  use ol_framework::vouch;
  use std::signer;
  use std::vector;

  #[test_only]
  use ol_framework::testnet;

  friend diem_framework::vouch_txs;
  friend ol_framework::filo_migration;

  #[test_only]
  friend ol_framework::test_filo_migration;

  /*
    Founder Status

    The founder status is a special designation that identifies pre-v8 accounts that have established
    a web of trust with other users. This is critical for anti-sybil security, as it helps verify
    that accounts are operated by real human users rather than bots or sock puppet accounts.

    By requiring a minimum trust score (implemented in page_rank_lazy.move), the system can ensure
    that an account has meaningful connections with other accounts in the network. This is done by:

    1. Calculating the trust score based on the account's position in the network graph
    2. Checking if this score exceeds the threshold defined by MULTIPLIER
    3. Only setting founder status as "has_human_friends" when this threshold is met

    This works in conjunction with the anti-sybil protections in vouch.move, which prevent
    rapid vouching and revoking to create fake identities.
  */

  /// The multiplier against the page_rank_lazy max_single_score for a user to be considered well-vouched.
  /// NOTE: A multiplier of 1, means the score equivalent of having one root
  /// of trust user vouch for you to qualify as having human friends.
  const MULTIPLIER: u64 = 1;

  struct Founder has key {
    has_human_friends: bool
  }

  /// Migrates a user account by creating a Founder resource if it doesn't exist.
  public(friend) fun migrate(user_sig: &signer) {
    if (!exists<Founder>(signer::address_of(user_sig))) {
      move_to<Founder>(user_sig, Founder {
        has_human_friends: false // ooh it's lonely at the top
      });
    }
  }

  /// Sets the founder as having human friends if they meet the voucher score criteria.
  /// DANGER: open to any friend function
  public(friend) fun maybe_set_friendly_founder(user: address) acquires Founder {
    if (
      is_founder(user) &&
      is_voucher_score_valid(user)
    ) {
      let f = borrow_global_mut<Founder>(user);
      f.has_human_friends = true;
    }
  }

  #[view]
  /// Checks if the user's trust score meets the required threshold.
  public fun is_voucher_score_valid(user: address): bool {
    diem_std::debug::print(&77777);
    // requires a minimum of N vouches
    let len = vector::length(&vouch::get_received_vouches_not_expired(user));
    if (len < 2) {
      return false
    };

    page_rank_lazy::get_trust_score(user) >= MULTIPLIER * page_rank_lazy::get_max_single_score()
  }

  #[view]
  /// Checks if the account is a founder (pre-v8 account that has been migrated).
  public fun is_founder(user: address): bool {
    exists<Founder>(user)
  }

  #[view]
  /// Checks if the founder has established connections with other users.
  /// Returns true if the founder has human friends (passed the trust threshold).
  public fun has_friends(user: address): bool acquires Founder {
    let f = borrow_global<Founder>(user);
    f.has_human_friends
  }

  #[test_only]
  /// Mock a founder as having friends for testing purposes.
  public(friend) fun test_mock_friendly(framework: &signer, user: &signer) acquires Founder {
    testnet::assert_testnet(framework);
    let state = borrow_global_mut<Founder>(signer::address_of(user));
    state.has_human_friends = true;
  }
}
