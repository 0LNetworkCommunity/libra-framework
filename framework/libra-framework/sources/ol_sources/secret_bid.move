/////////////////////////////////////////////////////////////////////////
// 0L Module
// Secret Bid
// Implements a commit-reveal scheme for more private bidding in PoF auction
///////////////////////////////////////////////////////////////////////////

module ol_framework::secret_bid {
  use std::bcs;
  use std::error;
  use std::hash;
  use std::signer;
  use std::vector;

  use diem_std::ed25519;
  use diem_std::comparator;
  use diem_framework::epoch_helper;
  use diem_framework::block;

  #[test_only]
  use diem_framework::account;
  // use diem_framework::debug::print;

  /// User bidding not initialized
  const ECOMMIT_BID_NOT_INITIALIZED: u64 = 1;
  /// Invalid signature on bid message
  const EINVALID_SIGNATURE: u64 = 2;
  /// Must reveal bid in same epoch as committed
  const EMISMATCH_EPOCH: u64 = 3;
  /// Must submit bid before reveal window opens, and reveal bid only after.
  const ENOT_IN_REVEAL_WINDOW: u64 = 4;
  /// Bad Alice, the reveal does not match the commit
  const ECOMMIT_DIGEST_NOT_EQUAL: u64 = 5;


  struct CommittedBid has key {
    reveal_net_reward: u64,
    commit_digest: vector<u8>,
    commit_epoch: u64,
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

  public entry fun commit(user: &signer, digest: vector<u8>) acquires CommittedBid {
    // don't allow commiting within reveal window
    assert!(!in_reveal_window(), error::invalid_state(ENOT_IN_REVEAL_WINDOW));
    commit_net_reward_impl(user, digest);
  }

  // TODO: the public key could be the consensus_pubkey that is already registered for the validator. That way the operator does not need the root key for bidding strategy. Note it's a BLS key.
  public entry fun reveal(user: &signer, pk: vector<u8>, net_reward: u64, signed_msg: vector<u8>) acquires CommittedBid {
    // don't allow commiting within reveal window
    assert!(in_reveal_window(), error::invalid_state(ENOT_IN_REVEAL_WINDOW));
    reveal_net_reward_impl(user, pk, net_reward, signed_msg);
  }

  /// user transaction for setting a signed message with the bid
  fun commit_net_reward_impl(user: &signer, digest: vector<u8>) acquires CommittedBid {

    if (!is_init(signer::address_of(user))) {
      move_to<CommittedBid>(user, CommittedBid {
        reveal_net_reward: 0,
        commit_digest: vector::empty(),
        commit_epoch: 0,
      });
    };

    let state = borrow_global_mut<CommittedBid>(signer::address_of(user));
    state.commit_digest = digest;
    state.commit_epoch = epoch_helper::get_current_epoch();
  }

  /// The hashing protocol which the client will be submitting commitments
  /// instead of a sequence number, we are using the signed message for hashing nonce, which would be private to the validator before the reveal.
  // TODO: should we also use a salt or overkill for these purposes?
  fun make_hash(net_reward: u64, epoch: u64, signed_message: vector<u8>): vector<u8>{
    let bid = Bid {
      net_reward,
      epoch,
    };

    let bid_bytes = bcs::to_bytes(&bid);
    vector::append(&mut bid_bytes, copy signed_message);
    let commitment = hash::sha3_256(bid_bytes);
    commitment
  }

  /// user sends transaction  which takes the committed signed message
  /// submits the public key used to sign message, which we compare to the authentication key.
  /// we use the epoch as the sequence number, so that messages are different on each submission.
  public fun reveal_net_reward_impl(user: &signer, pk: vector<u8>, net_reward: u64, signed_msg: vector<u8>) acquires CommittedBid {
    assert!(is_init(signer::address_of(user)), error::invalid_state(ECOMMIT_BID_NOT_INITIALIZED));

    let state = borrow_global_mut<CommittedBid>(signer::address_of(user));


    // must reveal within the current epoch of the bid
    let epoch = epoch_helper::get_current_epoch();
    assert!(epoch == state.commit_epoch, error::invalid_state(EMISMATCH_EPOCH));


    let commitment = make_hash(net_reward, epoch, signed_msg);

    let bid = Bid {
      net_reward,
      epoch,
    };

    check_signature(pk, signed_msg, bid);

    assert!(comparator::is_equal(&comparator::compare(&commitment, &state.commit_digest)), error::invalid_argument(ECOMMIT_DIGEST_NOT_EQUAL));


    state.reveal_net_reward = net_reward;
  }

  fun check_signature(account_public_key_bytes: vector<u8>, signed_message_bytes: vector<u8>, bid_message: Bid) {
    let pubkey = ed25519::new_unvalidated_public_key_from_bytes(account_public_key_bytes);

    // confirm that the signature bytes belong to the message (the Bid struct)
    let sig = ed25519::new_signature_from_bytes(signed_message_bytes);
    assert!(ed25519::signature_verify_strict_t(&sig, &pubkey, bid_message), error::invalid_argument(EINVALID_SIGNATURE));

    /////////
    // NOTE: previously we would check if the public key for the signed message
    // belonged to this account. However that is not necessary for prevention of the replay attack. Any ed25519 key pair would be acceptable for producing the nonce, and the user may prefer to use one that is different than their main account's key.
    // let expected_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&pubkey);
    // assert!(account::get_authentication_key(signer::address_of(user)) == expected_auth_key, error::invalid_argument(EWRONG_CURRENT_PUBLIC_KEY));
    ////////
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

      assert!(ed25519::signature_verify_strict_t<Bid>(&sig_again, &new_pk_unvalidated, copy message), error::invalid_argument(EINVALID_SIGNATURE));

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      check_signature(pk_bytes, sig_bytes, message);

  }

  #[test]
  #[expected_failure(abort_code = 65538, location = Self)]
  fun test_check_signature_sad() {
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

      assert!(ed25519::signature_verify_strict_t<Bid>(&sig_again, &new_pk_unvalidated, copy message), error::invalid_argument(EINVALID_SIGNATURE));

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      let message = Bid {
        net_reward: 2, // incorrect
        epoch: 0,
      };

      check_signature(pk_bytes, sig_bytes, message);

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
      let net_reward = 5;
      let epoch = 0;

      let message = Bid {
        net_reward,
        epoch,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      let digest = make_hash(net_reward, epoch, sig_bytes);
      // end set-up

      commit_net_reward_impl(&alice, digest);
  }

  #[test(framework = @0x1)]
  fun test_reveal(framework: &signer) acquires CommittedBid {
      use diem_std::from_bcs;

      let epoch = 1;
      epoch_helper::test_set_epoch(framework, epoch);

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);
      let net_reward = 5;
      let message = Bid {
        net_reward,
        epoch,
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      let digest = make_hash(net_reward, epoch, sig_bytes);
      // end set-up

      commit_net_reward_impl(&alice, digest);

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      check_signature(pk_bytes, sig_bytes, message);

      reveal_net_reward_impl(&alice, pk_bytes, 5, sig_bytes);
  }

  #[test(framework = @0x1)]
  #[expected_failure(abort_code = 65538, location = Self)]
  fun test_reveal_sad_wrong_epoch(framework: &signer) acquires CommittedBid {
      use diem_std::from_bcs;
      let epoch = 1;
      epoch_helper::test_set_epoch(framework, epoch);

      let (new_sk, new_pk) = ed25519::generate_keys();
      let new_pk_unvalidated = ed25519::public_key_to_unvalidated(&new_pk);
      let new_auth_key = ed25519::unvalidated_public_key_to_authentication_key(&new_pk_unvalidated);
      let new_addr = from_bcs::to_address(new_auth_key);
      let alice = account::create_account_for_test(new_addr);
      let net_reward = 5;
      let wrong_epoch = 100;

      let message = Bid {
        net_reward,
        epoch: wrong_epoch, // wrong epoch, we are at 1
      };

      let to_sig = ed25519::sign_struct(&new_sk, copy message);
      let sig_bytes = ed25519::signature_to_bytes(&to_sig);
      let digest = make_hash(net_reward, epoch, sig_bytes);
      // end set-up

      commit_net_reward_impl(&alice, digest);

      let pk_bytes = ed25519::unvalidated_public_key_to_bytes(&new_pk_unvalidated);

      check_signature(pk_bytes, sig_bytes, message);

      reveal_net_reward_impl(&alice, pk_bytes, 5, sig_bytes);
  }
}
