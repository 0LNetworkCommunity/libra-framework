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
  use diem_framework::block;


  // use diem_framework::debug::print;

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

  struct Bid has drop, copy {
    net_reward: u64,
    epoch: u64,
  }

  /// check if we are within the reveal window
  /// do not allow bids within the reveal window
  /// allow reveal transaction to be submitted
  fun in_reveal_window(): bool {
    // get the timestamp
    // NOTE: this might cause a dependency cycle issue in the future
    let remaining_secs = block::get_remaining_epoch_secs();
    let five_mins = 60*5;
    if (remaining_secs > five_mins) {
      return false
    };
    true
  }

  fun is_init(account: address): bool {
    exists<CommittedBid>(account)
  }

  public entry fun commit(user: &signer, message: vector<u8>) acquires CommittedBid {
    // don't allow commiting within reveal window
    assert!(!in_reveal_window(), error::invalid_state(ENOT_IN_REVEAL_WINDOW));
    commit_net_reward_impl(user, message);
  }

  public entry fun reveal(user: &signer, pk: vector<u8>, net_reward: u64) acquires CommittedBid {
    // don't allow commiting within reveal window
    assert!(in_reveal_window(), error::invalid_state(ENOT_IN_REVEAL_WINDOW));
    reveal_net_reward_impl(user, pk, net_reward);
  }

  /// user transaction for setting a signed message with the bid
  fun commit_net_reward_impl(user: &signer, message: vector<u8>) acquires CommittedBid {

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
  public fun reveal_net_reward_impl(user: &signer, pk: vector<u8>, net_reward: u64) acquires CommittedBid {
    assert!(is_init(signer::address_of(user)), error::invalid_state(ECOMMIT_BID_NOT_INITIALIZED));

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

  //////// TESTS ////////
  #[test]
  fun test_sign_message() {
      use diem_std::from_bcs;

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let _alice = account::create_account_for_test(new_addr);

      let message = Bid {
        net_reward: 0,
        epoch: 0,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      // end set-up

      // yes repetitive, but following the same workflow
      let sig_again = ed25519::new_signature_from_bytes(sig_bytes);

      assert!(ed25519::signature_verify_strict_t<Bid>(&sig_again, &new_pk_unvalidated, message), error::invalid_argument(EINVALID_SIGNATURE));


  }

  #[test]
  fun test_check_signature() {
      use diem_std::from_bcs;

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);

      let message = Bid {
        net_reward: 0,
        epoch: 0,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      // end set-up

      // yes repetitive, but following the same workflow
      let sig_again = ed25519::new_signature_from_bytes(sig_bytes);

      assert!(ed25519::signature_verify_strict_t<Bid>(&sig_again, &new_pk_unvalidated, copy message), error::invalid_argument(EINVALID_SIGNATURE));

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      check_signature(&alice, pk_bytes, sig_bytes, message);

  }

  #[test]
  #[expected_failure(abort_code = 65538, location = Self)]
  fun test_check_signature_sad() {
      use diem_std::from_bcs;

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);

      let message = Bid {
        net_reward: 0,
        epoch: 0,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      // end set-up

      // yes repetitive, but following the same workflow
      let sig_again = ed25519::new_signature_from_bytes(sig_bytes);

      assert!(ed25519::signature_verify_strict_t<Bid>(&sig_again, &new_pk_unvalidated, copy message), error::invalid_argument(EINVALID_SIGNATURE));

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      let message = Bid {
        net_reward: 2, // incorrect
        epoch: 0,
      };

      check_signature(&alice, pk_bytes, sig_bytes, message);

  }


  #[test(framework = @0x1)]
  fun test_commit_message(framework: &signer) acquires CommittedBid {
      use diem_std::from_bcs;

      let this_epoch = 1;
      epoch_helper::test_set_epoch(framework, this_epoch);

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);

      let message = Bid {
        net_reward: 5,
        epoch: 0,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      // end set-up

      commit_net_reward_impl(&alice, sig_bytes);
  }

  #[test(framework = @0x1)]
  fun test_reveal(framework: &signer) acquires CommittedBid {
      use diem_std::from_bcs;

      let this_epoch = 1;
      epoch_helper::test_set_epoch(framework, this_epoch);

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);

      let message = Bid {
        net_reward: 5,
        epoch: this_epoch,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      // end set-up

      commit_net_reward_impl(&alice, sig_bytes);

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      check_signature(&alice, pk_bytes, sig_bytes, message);

      reveal_net_reward_impl(&alice, pk_bytes, 5);
  }

  #[test(framework = @0x1)]
  #[expected_failure(abort_code = 65538, location = Self)]
  fun test_reveal_sad_wrong_epoch(framework: &signer) acquires CommittedBid {
      use diem_std::from_bcs;
      let this_epoch = 1;
      epoch_helper::test_set_epoch(framework, this_epoch);

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);

      let message = Bid {
        net_reward: 5,
        epoch: 100, // wrong epoch, we are at 1
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      // end set-up

      commit_net_reward_impl(&alice, sig_bytes);

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      check_signature(&alice, pk_bytes, sig_bytes, message);

      reveal_net_reward_impl(&alice, pk_bytes, 5);
  }
}
