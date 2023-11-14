
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
    use diem_framework::event::{Self, EventHandle};
    use diem_framework::coin::{Self, Coin};
    use ol_framework::ol_account;
    use ol_framework::libra_coin::LibraCoin;
    use ol_framework::globals;
    use ol_framework::vouch;
    use ol_framework::epoch_helper;
    use std::error;

    // use diem_std::debug::print;

    friend ol_framework::epoch_boundary;
    friend ol_framework::tower_state;

    /// You need a minimum of Vouches on your account for this task.
    /// They also must be of unrelated buddies.
    /// Meaning: they don't come from the same ancestry of accounts.
    /// Go meet more people!
    const ENOT_ENOUGH_UNRELATED_VOUCHERS: u64 = 1;
    /// You'll need to have some vouches by accounts in the miner community.
    /// We check if you had any vouchers (buddies) among the successful
    /// miners of the previous epoch.
    const ENEED_MORE_FRIENDS_IN_MINER_COMMUNITY: u64 = 2;

    /// somehow your submission is behind the blockchains's time
    const ETIME_IS_IN_PAST_WHAAAT: u64 = 3;

    /// not enough time has passed between proofs.
    const ETOO_SOON_SUBMITTED: u64 = 4;

    /// Trying to submit too many proofs in period
    const EABOVE_SUBMISSION_THRESH: u64 = 5;

    /// A list of all miners' addresses
    // reset at epoch boundary
    struct ProviderList has key {
      current_above_threshold: vector<address>,
      previous_epoch_list: vector<address>,
    }

    /// the root account's counter
    struct GlobalCounter has key {
      lifetime_proofs: u64,
      proofs_in_epoch: u64,
      proofs_in_epoch_above_thresh: u64,
    }

    /// a user's account Tower of proofs
    /// a private blockchain of the proofs submitted.
    struct Tower has key {
        last_commit_timestamp: u64,
        previous_proof_hash: vector<u8>,
        verified_tower_height: u64,
        latest_epoch_mining: u64,
        count_proofs_in_epoch: u64,
        epochs_mining: u64,
        contiguous_epochs_mining: u64,
        distribute_rewards_events: EventHandle<DistributeRewardsEvent>,
    }


    /// an event struct to register payment events
    struct DistributeRewardsEvent has drop, store {
        account: address,
        rewards_amount: u64,
    }


    public fun initialize(root: &signer) {
      move_to(root, GlobalCounter {
        lifetime_proofs: 0,
        proofs_in_epoch: 0,
        proofs_in_epoch_above_thresh: 0,
      });

      move_to(root, ProviderList {
        current_above_threshold: vector::empty(),
        previous_epoch_list: vector::empty(),

      });
    }

    // init a new provider account, if they are not migrating a tower.
    public entry fun init_provider(provider: &signer) {
      if (!exists<Tower>(signer::address_of(provider))) {
        move_to(provider, Tower {
          last_commit_timestamp: 0,
          previous_proof_hash: vector::empty(),
          verified_tower_height: 0,
          latest_epoch_mining: 0,
          count_proofs_in_epoch: 0,
          epochs_mining: 0,
          contiguous_epochs_mining: 0,
          distribute_rewards_events: account::new_event_handle<DistributeRewardsEvent>(provider)
        });
      }
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
        distribute_rewards_events: account::new_event_handle<DistributeRewardsEvent>(provider)
      })
    }

    public fun submit_proof(
      provider: &signer,
      public_key_bytes: vector<u8>,
      signature_bytes: vector<u8>,
      ) acquires GlobalCounter, Tower, ProviderList {
      let provider_addr = signer::address_of(provider);

      // Don't populate the oracle miner list with accounts that don't have vouches.
      {
        // must have 3 accounts who are unrelated vouching for you.
        let frens = vouch::true_friends(provider_addr);
        assert!(vector::length(&frens) > 2, error::invalid_state(ENOT_ENOUGH_UNRELATED_VOUCHERS));
        // in the previous epoch of successful miners, you'll need to have 3 unrelated vouchers there as well.
        let previous_epoch_list = &borrow_global<ProviderList>(@ol_framework).previous_epoch_list;
        let (_, count_buddies) = vouch::true_friends_in_list(provider_addr, previous_epoch_list);
        assert!(count_buddies > 2, error::invalid_state(ENEED_MORE_FRIENDS_IN_MINER_COMMUNITY));

      };


      // the message needs to be exactly the hash of the previous proof.
      // first check if enough time has passed.
      let time = timestamp::now_microseconds();
      let tower = borrow_global_mut<Tower>(provider_addr);
      // can't send multiple in same tx
      assert!(time > tower.last_commit_timestamp, ETIME_IS_IN_PAST_WHAAAT); // TODO: fill out error
      // the sufficient time has passed
      assert!(time > tower.last_commit_timestamp + proof_interval_seconds() , ETOO_SOON_SUBMITTED);

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
      increment_stats(provider_addr, tower, time, signature_bytes);

    }

    fun increment_stats(provider_addr: address, tower: &mut Tower, time: u64, signature_bytes: vector<u8>) acquires GlobalCounter, ProviderList {

            // update the global state
      let global = borrow_global_mut<GlobalCounter>(@ol_framework);
            // is this a proof in a new epoch?
      let current_epoch = epoch_helper::get_current_epoch();

      // if this is the first proof this epoch;

      if (current_epoch > tower.latest_epoch_mining) { // 370 , 10 = true
        // we lazily reset this counter
        tower.count_proofs_in_epoch = 0;
        // and if this first proof is exaclty in a contiguous epoch
        // it qualifies as a streak
        if (tower.latest_epoch_mining + 1 == current_epoch) { // 11, 370, false
          tower.contiguous_epochs_mining = tower.contiguous_epochs_mining + 1;
        } else if (tower.latest_epoch_mining + 1 < current_epoch) { // reset it
        // 11, 370, true
          tower.contiguous_epochs_mining = 0;
        }
      };
      // exit early if above threshold
      assert!(
        tower.count_proofs_in_epoch  < globals::get_epoch_mining_thres_upper(),
        error::invalid_state(EABOVE_SUBMISSION_THRESH)
      );
      // update providers state
      tower.last_commit_timestamp = time;
      tower.previous_proof_hash = signature_bytes;
      tower.verified_tower_height = tower.verified_tower_height + 1;
      tower.count_proofs_in_epoch = tower.count_proofs_in_epoch + 1;
      tower.epochs_mining = tower.epochs_mining + 1;
      tower.latest_epoch_mining = epoch_helper::get_current_epoch();

      // update globals
      global.lifetime_proofs = global.lifetime_proofs + 1;
      global.proofs_in_epoch = global.proofs_in_epoch + 1;
      // also check if the tower is now above the threshold
      if (tower.count_proofs_in_epoch > globals::get_epoch_mining_thres_lower()) {
        global.proofs_in_epoch_above_thresh = global.proofs_in_epoch_above_thresh + 1;
        // also add to the provider list which would be elegible for rewards
        let provider_list = borrow_global_mut<ProviderList>(@ol_framework);
        if (!vector::contains(&provider_list.current_above_threshold, &provider_addr)) {
          vector::push_back(&mut provider_list.current_above_threshold, provider_addr);
        }
      };
    }

    // while transitioning to oracle, allow vdf proofs from miners.
    // can only be called by tower
    public(friend) fun count_vdf_proof(
      provider_addr: address,
      signature_bytes: vector<u8>
      ) acquires GlobalCounter, Tower, ProviderList {
      // let provider_addr = signer::address_of(provider);
      // the message needs to be exactly the hash of the previous proof.
      // first check if enough time has passed.
      let time = timestamp::now_microseconds();
      let tower = borrow_global_mut<Tower>(provider_addr);
      // can't send multiple in same tx
      assert!(time > tower.last_commit_timestamp, ETIME_IS_IN_PAST_WHAAAT); // TODO: fill out error
      // the sufficient time has passed
      assert!(time > tower.last_commit_timestamp + proof_interval_seconds() , ETOO_SOON_SUBMITTED);

      increment_stats(provider_addr, tower, time, signature_bytes);

    }

    // how long should the oracle delay be (except for VDF proofs)
    // in testnet it should be 30 seconds.
    // in production its 1 hour.
    fun proof_interval_seconds(): u64 {
      if (testnet::is_testnet()) {
        30
      } else {
        60 * 60 // 1 hr
      }
    }

    public(friend) fun epoch_boundary(root: &signer, budget: &mut Coin<LibraCoin>): (u64, u64) acquires GlobalCounter, ProviderList, Tower {
      let (provider_count, paid_amount ) = epoch_reward(root, budget);
      reset_counters(root);
      (provider_count, paid_amount)
    }

    fun reset_counters(root: &signer) acquires ProviderList, GlobalCounter{
      system_addresses::assert_ol(root);
      let provider_state = borrow_global_mut<ProviderList>(@ol_framework);
      provider_state.previous_epoch_list = provider_state.current_above_threshold;
      provider_state.current_above_threshold = vector::empty<address>();

      let counter_state = borrow_global_mut<GlobalCounter>(@ol_framework);
      counter_state.proofs_in_epoch = 0;
      counter_state.proofs_in_epoch_above_thresh = 0;

    }

    /// from the total reward, available to the miners, divide equally among
    /// successful miners.
    /// returns: provider_list_len total_deposited
    ///
    fun epoch_reward(root: &signer, budget: &mut Coin<LibraCoin>): (u64, u64) acquires ProviderList, Tower {
      system_addresses::assert_ol(root);

      let coin_value = coin::value(budget);

      let provider_list = borrow_global_mut<ProviderList>(@ol_framework).current_above_threshold;

      let len = vector::length(&provider_list);

      if (len == 0) return (0, 0);
      let total_deposited = 0;
      let per_user = coin_value / len;
      vector::for_each_ref(&provider_list, |addr| {
        emit_distribute_reward(root, addr, per_user);
        let split = coin::extract(budget, per_user);
        let value = coin::value(&split);
        total_deposited = total_deposited + value;
        ol_account::deposit_coins(*addr, split);
      });

      (len, total_deposited)
    }

    // since rewards are handled externally to stake.move we need an api to emit the event
    public(friend) fun emit_distribute_reward(root: &signer, account: &address, rewards_amount: u64) acquires Tower {
        system_addresses::assert_ol(root);
        let oracle_tower = borrow_global_mut<Tower>(*account);
        event::emit_event(
          &mut oracle_tower.distribute_rewards_events,
          DistributeRewardsEvent {
              account: *account,
              rewards_amount,
          },
      );
    }

    //////// GETTERS ////////

    #[view]
    /// returns the number of proofs for a miner in the current epoch
    public fun get_count_in_epoch(miner_addr: address): u64 acquires Tower {
      if (exists<Tower>(miner_addr)) {
        let s = borrow_global<Tower>(miner_addr);
        if (s.latest_epoch_mining == epoch_helper::get_current_epoch()) {
          return s.count_proofs_in_epoch
        };
      };
      0
    }

    //////// TEST HELPERS ////////
    #[test_only]
    public fun set_tower(root: &signer, addr: address, count_proofs_in_epoch:
    u64, latest_epoch_mining: u64) acquires Tower {
      system_addresses::assert_ol(root);
      let state = borrow_global_mut<Tower>(addr);
      state.count_proofs_in_epoch = count_proofs_in_epoch;
      state.latest_epoch_mining = latest_epoch_mining
    }

    #[test_only]
    /// returns the number of proofs for a miner in the current epoch
    public fun get_exact_count(miner_addr: address): u64 acquires Tower {
      let s = borrow_global<Tower>(miner_addr);
      return s.count_proofs_in_epoch
    }

}
