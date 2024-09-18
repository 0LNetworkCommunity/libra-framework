/////////////////////////////////////////////////////////////////////////
// 0L Module
// Secret Bid
// Implements a commit-reveal scheme for more private bidding in PoF auction
///////////////////////////////////////////////////////////////////////////

module ol_framework::secret_bid {
  use std::error;
  use std::signer;
  use std::vector;
  use diem_std::ed25519;
  use diem_framework::account;
  use diem_framework::epoch_helper;

  /// User bidding not initialized
  const ECOMMIT_BID_NOT_INITIALIZED: u64 = 0;
  /// Specified current public key is not correct
  const EWRONG_CURRENT_PUBLIC_KEY: u64 = 1;
  /// Invalid signature on bid message
  const EINVALID_SIGNATURE: u64 = 2;
  /// Must reveal bid in same epoch as committed
  const EMISMATCH_EPOCH: u64 = 3;
  /// Must submit bid before reveal window opens, and reveal bid only after.
  const ENOT_IN_REVEAL_WINDOW: u64 = 4;


  struct CommittedBid has key {
    reveal_net_reward: u64,
    commit_epoch: u64,
    commit_signed_message: vector<u8>
  }

  struct Bid has drop {
    net_reward: u64,
    epoch: u64,
  }

  /// check if we are within the reveal window
  /// do not allow bids within the reveal window
  /// allow reveal transaction to be submitted
  fun in_reveal_window(): bool {
    // get the timestamp
    // get the epoch ending timestamp
    // get 5 mins prior
    true
  }

  fun is_init(account: address): bool {
    exists<CommittedBid>(account)
  }

  /// user transaction for setting a signed message with the bid
  public fun commit_net_reward(user: &signer, message: vector<u8>) acquires CommittedBid {
    // don't allow commiting within reveal window
    assert!(!in_reveal_window(), error::invalid_state(ENOT_IN_REVEAL_WINDOW));

    if (!is_init(signer::address_of(user))) {
      move_to<CommittedBid>(user, CommittedBid {
        reveal_net_reward: 0,
        commit_epoch: 0,
        commit_signed_message: vector::empty(),
      });
    };

    let state = borrow_global_mut<CommittedBid>(signer::address_of(user));
    state.commit_signed_message = message;
    state.commit_epoch = epoch_helper::get_current_epoch();
  }

  /// user sends transaction  which takes the committed signed message
  /// submits the public key used to sign message, which we compare to the authentication key.
  /// we use the epoch as the sequence number, so that messages are different on each submission.
  public fun reveal_net_reward(user: &signer, pk: vector<u8>, net_reward: u64) acquires CommittedBid {
    assert!(is_init(signer::address_of(user)), error::invalid_state(ECOMMIT_BID_NOT_INITIALIZED));

    assert!(in_reveal_window(), error::invalid_state(ENOT_IN_REVEAL_WINDOW));
    let state = borrow_global_mut<CommittedBid>(signer::address_of(user));

    // must reveal within the current epoch of the bid
    let epoch = epoch_helper::get_current_epoch();
    assert!(epoch == state.commit_epoch, error::invalid_state(EMISMATCH_EPOCH));

    let bid = Bid {
      net_reward,
      epoch,
    };

    check_signature(user, pk, state.commit_signed_message, bid);

    state.reveal_net_reward = net_reward;
  }

  fun check_signature(user: &signer, account_public_key_bytes: vector<u8>, signed_message_bytes: vector<u8>, bid_message: Bid) {
    let pubkey = ed25519::new_unvalidated_public_key_from_bytes(account_public_key_bytes);
    let expected_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&pubkey);
    assert!(account::get_authentication_key(signer::address_of(user)) == expected_auth_key, error::invalid_argument(EWRONG_CURRENT_PUBLIC_KEY));

    // confirm that the signature bytes belong to the message (the Bid struct)
    let sig = ed25519::new_signature_from_bytes(signed_message_bytes);
    assert!(ed25519::signature_verify_strict_t(&sig, &pubkey, bid_message), error::invalid_argument(EINVALID_SIGNATURE));
  }
}
