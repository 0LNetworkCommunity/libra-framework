/// Maintains the version number for the blockchain.
module ol_framework::founder {
  use std::signer;
  use ol_framework::page_rank_lazy;

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
    2. Checking if this score exceeds the threshold (currently 50)
    3. Only setting founder status as "has_human_friends" when this threshold is met

    This works in conjunction with the anti-sybil protections in vouch.move, which prevent
    rapid vouching and revoking to create fake identities.
  */

  /// The threshold score for a user to be considered well-vouched
  /// Users need a minimum score of 50 to qualify as having human friends
  const THRESHOLD_SCORE: u64 = 50;

  struct Founder has key {
    has_human_friends: bool
  }

  public(friend) fun migrate(user_sig: &signer) {
    if (!exists<Founder>(signer::address_of(user_sig))) {
      move_to<Founder>(user_sig, Founder {
        has_human_friends: false // ooh it's lonely at the top
      });
    }
  }

  // DANGER: open to any friend function
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
  public fun is_voucher_score_valid(user: address): bool {
    page_rank_lazy::get_trust_score(user) >= THRESHOLD_SCORE
  }

  #[view]
  // If the account is a founder/pre-v8 account has been migrated
  // then it would have an onboarding timestamp of 0
  public fun is_founder(user: address): bool {
    exists<Founder>(user)
  }

  #[view]
  /// Bot or not
  public fun has_friends(user: address): bool acquires Founder {
    let f = borrow_global<Founder>(user);
    f.has_human_friends
  }

  #[test_only]
  public(friend) fun test_mock_friendly(framework: &signer, user: &signer) acquires Founder {
    testnet::assert_testnet(framework);
    let state = borrow_global_mut<Founder>(signer::address_of(user));
    state.has_human_friends = true;
  }

}
