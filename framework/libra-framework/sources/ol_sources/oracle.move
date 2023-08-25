
module ol_framework::oracle {

    use std::vector;
    use std::signer;
    use std::option;
    use ol_framework::testnet;
    use diem_framework::system_addresses;
    use diem_framework::timestamp;
    use diem_framework::account;
    use diem_std::ed25519;
    use diem_std::comparator;

    /// A list of all miners' addresses
    // reset at epoch boundary
    struct ProviderList has key {
      list: vector<address>
    }

    struct GlobalCounter has key {
      lifetime_proofs: u64,
      proofs_in_epoch: u64,
      proofs_in_epoch_above_thresh: u64,
    }

    struct Tower has key {
        last_commit_timestamp: u64,
        previous_proof_hash: vector<u8>,
        verified_tower_height: u64,
        latest_epoch_mining: u64,
        count_proofs_in_epoch: u64,
        epochs_mining: u64,
        contiguous_epochs_mining: u64,
    }


    public fun initialize(root: &signer) {
      move_to(root, GlobalCounter {
        lifetime_proofs: 0,
        proofs_in_epoch: 0,
        proofs_in_epoch_above_thresh: 0,
      });

      move_to(root, ProviderList {
        list: vector::empty(),
      });
    }

    // init a new provider account, if they are not migrating a tower.
    public entry fun init_provider(provider: &signer) {
      move_to(provider, Tower {
        last_commit_timestamp: 0,
        previous_proof_hash: vector::empty(),
        verified_tower_height: 0,
        latest_epoch_mining: 0,
        count_proofs_in_epoch: 0,
        epochs_mining: 0,
        contiguous_epochs_mining: 0,
      });

    }

    /// At genesis this can be called once to migrate towers
    public fun migrate_from_vdf_tower(
      root: &signer,
      provider: &signer,
      previous_proof_hash: vector<u8>,
      verified_tower_height: u64,
      latest_epoch_mining: u64,
      count_proofs_in_epoch: u64,
      epochs_validating_and_mining: u64,
      contiguous_epochs_validating_and_mining: u64,


    ) {
      system_addresses::assert_ol(root);
      move_to(provider, Tower {
        last_commit_timestamp: 0,
        previous_proof_hash,
        verified_tower_height,
        latest_epoch_mining,
        count_proofs_in_epoch,
        epochs_mining: epochs_validating_and_mining,
        contiguous_epochs_mining: contiguous_epochs_validating_and_mining,
      })
    }

    public fun submit_proof(
      provider: &signer,
      public_key_bytes: vector<u8>,
      signature_bytes: vector<u8>,
      ) acquires GlobalCounter, Tower, ProviderList {
      let provider_addr = signer::address_of(provider);

      // the message needs to be exactly the hash of the previous proof.
      // first check if enough time has passed.
      let time = timestamp::now_microseconds();
      let tower = borrow_global_mut<Tower>(provider_addr);
      // can't send multiple in same tx
      assert!(time > tower.last_commit_timestamp, 77); // TODO: fill out error
      // the sufficient time has passed
      assert!(time > tower.last_commit_timestamp + proof_interval_seconds() , 77);

      // assert the public key used matched the auth key on account.
      let checked_pk = ed25519::new_validated_public_key_from_bytes(public_key_bytes);


      let auth = ed25519::validated_public_key_to_authentication_key(option::borrow(&checked_pk));

      let user_auth = account::get_authentication_key(provider_addr);
      assert!(auth == user_auth, 77);

      let res = comparator::compare_u8_vector(user_auth, auth);
      assert!(comparator::is_equal(&res), 88);

      // is the signed message's content the previous proof hash?
      // use Unverified type to do signature_verify_strict()

      let pk = ed25519::new_unvalidated_public_key_from_bytes(public_key_bytes);
      let sig = ed25519::new_signature_from_bytes(signature_bytes);
      assert!(ed25519::signature_verify_strict(&sig, &pk, tower.previous_proof_hash), 77);

      // the proof is valid, update the tower state.

      // update the global state
      let global = borrow_global_mut<GlobalCounter>(@ol_framework);
      global.lifetime_proofs = global.lifetime_proofs + 1;
      global.proofs_in_epoch = global.proofs_in_epoch + 1;


      // update providers state
      tower.last_commit_timestamp = time;
      tower.previous_proof_hash = signature_bytes;
      tower.verified_tower_height = tower.verified_tower_height + 1;
      tower.count_proofs_in_epoch = tower.count_proofs_in_epoch + 1;

      // also check if the tower is now above the threshold
       if (tower.count_proofs_in_epoch > threshold_of_signatures()) {
        global.proofs_in_epoch_above_thresh = global.proofs_in_epoch_above_thresh + 1;
        // also add to the provider list which would be elegible for rewards
        let provider_list = borrow_global_mut<ProviderList>(@ol_framework);
        vector::push_back(&mut provider_list.list, provider_addr);
      };


      let current_epoch = 0; // todo: get current epoch
      if (current_epoch == (tower.latest_epoch_mining - 1)) {
        tower.contiguous_epochs_mining = tower.contiguous_epochs_mining + 1;

      };
      tower.epochs_mining = tower.epochs_mining + 1;
      tower.latest_epoch_mining = 0; // todo: get current epoch;


    }

    // how long should the delay be.
    // in testnet it should be 30 seconds.
    // in production its 1 hour.
    fun proof_interval_seconds(): u64 {
      if (testnet::is_testnet()) {
        30
      } else {
        60 * 60
      }
    }

    // how many proofs needed in an epoch to be considered active
    fun threshold_of_signatures(): u64 {
      if (testnet::is_testnet()) {
        1
      } else {
        12
      }
    }

    fun has_unrelated_vouches_above_threshold() {

    }
}