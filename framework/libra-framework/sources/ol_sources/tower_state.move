///////////////////////////////////////////////////////////////////
module ol_framework::tower_state {
    use std::error;
    use std::signer;
    use std::vector;
    use std::hash;
    use ol_framework::globals;
    use aptos_framework::system_addresses;
    use aptos_framework::reconfiguration;
    use aptos_framework::testnet;
    use aptos_framework::ol_native_vdf;

    use aptos_std::debug::print;

    /// The current solution does not solve to previous hash.
    /// The delay proofs are not chained
    const EDELAY_NOT_CHAINED: u64 = 1;
    /// difficulty of proof does not match requirement.
    const EWRONG_DIFFICULTY: u64 = 2;
    /// security param of proof does not match requirement.
    const EWRONG_SECURITY: u64 = 3;
    /// security param of proof does not match requirement.
    const EADDRESS_NOT_IN_CHALLENGE: u64 = 4;
    /// Proof is not valid. The verification failed for solution at the expected difficulty and security parameter.
    const EPROOF_NOT_VALID: u64 = 5;

    /// A list of all miners' addresses 
    // reset at epoch boundary
    struct TowerList has key {
      list: vector<address>
    }

    /// To use in migration, and in future upgrade to deprecate.
    struct TowerStats has key {
      proofs_in_epoch: u64,
      validator_proofs: u64,
      fullnode_proofs: u64,
    }

    /// The struct to store the global count of proofs in 0x0
    struct TowerCounter has key {
      lifetime_proofs: u64,
      lifetime_validator_proofs: u64,
      lifetime_fullnode_proofs: u64,
      proofs_in_epoch: u64,
      validator_proofs_in_epoch: u64,
      fullnode_proofs_in_epoch: u64,
      validator_proofs_in_epoch_above_thresh: u64,
      fullnode_proofs_in_epoch_above_thresh: u64,
    }


    /// Struct to store information about a VDF proof submitted
    /// `challenge`: the seed for the proof 
    /// `difficulty`: the difficulty for the proof 
    ///               (higher difficulty -> longer proof time)
    /// `solution`: the solution for the proof (the result)
    struct Proof has drop {
        challenge: vector<u8>,
        difficulty: u64,
        solution: vector<u8>,
        security: u64,
    }

    /// Struct to encapsulate information about the state of a miner
    /// `previous_proof_hash`: the hash of their latest proof 
    ///     (used as seed for next proof)
    /// `verified_tower_height`: the height of the miner's tower 
    ///     (more proofs -> higher tower)
    /// `latest_epoch_mining`: the latest epoch the miner submitted sufficient
    ///     proofs (see GlobalConstants.epoch_mining_thres_lower)
    /// `count_proofs_in_epoch`: the number of proofs the miner has submitted 
    ///     in the current epoch 
    /// `epochs_mining`: the cumulative number of epochs 
    ///     the miner has been mining above threshold 
    ///     TODO does this actually only apply to validators? 
    /// `contiguous_epochs_mining`: the number of contiguous 
    ///     epochs the miner has been mining above threshold 
    ///     TODO does this actually only apply to validators?
    /// `epochs_since_last_account_creation`: the number of epochs since 
    ///     the miner last created a new account
    struct TowerProofHistory has key {
        previous_proof_hash: vector<u8>,
        verified_tower_height: u64, 
        latest_epoch_mining: u64,
        count_proofs_in_epoch: u64,
        epochs_mining: u64,
        contiguous_epochs_mining: u64,
        // epochs_since_last_account_creation: u64
    }

    struct VDFDifficulty has key {
      // current epoch number of iterations (difficulty) of vdf
      difficulty: u64,
      // current epoch security parameter for VDF
      security: u64,
      // previous epoch's difficulty. Allowance for all validtors on first proof of epoch
      prev_diff: u64,
      // allowance for first proof.
      prev_sec: u64,
    }

    public fun initialize(root: &signer) {
      init_difficulty(root);
      init_miner_list(root);
      init_tower_counter(root, 0, 0, 0);
    }
    // Create the difficulty struct
    public fun init_difficulty(vm: &signer) {
      system_addresses::assert_ol(vm);
      if (!exists<VDFDifficulty>(@ol_framework )) {
        if (testnet::is_testnet()) {
          move_to<VDFDifficulty>(vm, VDFDifficulty {
            difficulty: 100,
            security: 512,
            prev_diff: 100,
            prev_sec: 512,
          });
        } else {
          move_to<VDFDifficulty>(vm, VDFDifficulty {
            difficulty: 120000000,
            security: 512,
            prev_diff: 120000000,
            prev_sec: 512,
          });
        }

      }
    }

    /// Create an empty list of miners 
    fun init_miner_list(vm: &signer) {
      system_addresses::assert_ol(vm);
      move_to<TowerList>(vm, TowerList {
        list: vector::empty<address>()
      }); 
    }

    /// Create an empty miner stats 
    public fun init_tower_counter(
      vm: &signer,
      lifetime_proofs: u64,
      lifetime_validator_proofs: u64,
      lifetime_fullnode_proofs: u64,
    ) {
      if (!exists<TowerCounter>(@ol_framework)) {
        move_to<TowerCounter>(vm, TowerCounter {
          lifetime_proofs,
          lifetime_validator_proofs,
          lifetime_fullnode_proofs,
          proofs_in_epoch: 0,
          validator_proofs_in_epoch: 0,
          fullnode_proofs_in_epoch: 0,
          validator_proofs_in_epoch_above_thresh: 0,
          fullnode_proofs_in_epoch_above_thresh: 0,
        });
      }

    }

    // /// Create empty miners list and stats
    // public fun init_miner_list_and_stats(vm: &signer) {
    //   init_miner_list(vm);

    //   // Note: for testing migration we need to destroy this struct, see test_danger_destroy_tower_counter
    //   init_tower_counter(vm, 0, 0, 0); 
    // }


    // for hard fork migration
    public fun vm_migrate_tower_counter(
      vm: &signer,
      lifetime_proofs: u64,
      lifetime_validator_proofs: u64,
      lifetime_fullnode_proofs: u64,
    ) acquires TowerCounter {
      system_addresses::assert_ol(vm);
      let counter = borrow_global_mut<TowerCounter>(@ol_framework);
      counter.lifetime_proofs = lifetime_proofs;
      counter.lifetime_validator_proofs = lifetime_validator_proofs;
      counter.lifetime_fullnode_proofs = lifetime_fullnode_proofs;
    }

    /// returns true if miner at `addr` has been initialized 
    public fun is_init(addr: address):bool {
      exists<TowerProofHistory>(addr)
    }

    // /// is onboarding
    // public fun is_onboarding(addr: address): bool acquires TowerProofHistory {
    //   let count = get_count_in_epoch(addr);
    //   let state = borrow_global<TowerProofHistory>(addr);

    //   count < 2 &&
    //   state.epochs_since_last_account_creation < 2
    // }

    // Creates proof blob object from input parameters
    // Permissions: PUBLIC, ANYONE can call this function.
    public fun create_proof_blob(
      challenge: vector<u8>,
      solution: vector<u8>,
      difficulty: u64,
      security: u64,
    ): Proof {
       Proof {
         challenge,
         difficulty,
         solution,
         security,
      }
    }

    /// Private, can only be called within module
    /// adds `tower` to list of towers 
    fun increment_miners_list(miner: address) acquires TowerList {
      if (exists<TowerList>(@0x0)) {
        let state = borrow_global_mut<TowerList>(@0x0);
        if (!vector::contains<address>(&mut state.list, &miner)) {
          vector::push_back<address>(&mut state.list, miner);
        }
      }
    }

    // Helper function for genesis to process genesis proofs.
    // Permissions: PUBLIC, ONLY VM, AT GENESIS.
    public fun genesis_helper(
      vm_sig: &signer,
      miner_sig: &signer,
      challenge: vector<u8>,
      solution: vector<u8>,
      difficulty: u64,
      security: u64,
    ) acquires TowerProofHistory, TowerList, TowerCounter {
      
      system_addresses::assert_ol(vm_sig);
      init_miner_state(miner_sig, &challenge, &solution, difficulty, security);
    }

    /// This function is called to submit proofs to the chain 
    /// Function index: 01
    /// Permissions: PUBLIC, ANYONE
    public fun commit_state(
      miner_sign: &signer,
      proof: Proof
    ) acquires TowerProofHistory, TowerList, TowerCounter {
      print(&7777777777777);
      // Get address, assumes the sender is the signer.
      let miner_addr = signer::address_of(miner_sign);
      // let diff = borrow_global<VDFDifficulty>(@ol_framework );

      // This may be the 0th proof of an end user that hasn't had tower state initialized
      if (!is_init(miner_addr)) {
        assert!(&proof.difficulty 
          == &globals::get_vdf_difficulty_baseline(), error::invalid_argument(EWRONG_DIFFICULTY));
        assert!(&proof.security
          == &globals::get_vdf_security_baseline(), error::invalid_argument(EWRONG_SECURITY));
        // check proof belongs to user.
        let (addr_in_proof, _) = ol_native_vdf::extract_address_from_challenge(&proof.challenge);
        assert!(addr_in_proof == signer::address_of(miner_sign), error::permission_denied(EADDRESS_NOT_IN_CHALLENGE));

        init_miner_state(
          miner_sign, &proof.challenge, &proof.solution, proof.difficulty, proof.security
        );
        return
      };

      

      // // Skip this check on local tests, we need tests to send different difficulties.
      // if (!testnet::is_testnet()){
      //   // Get vdf difficulty constant. Will be different in tests than in production.
        
      //   // need to also give allowance for user's first proof in epoch to be in the last proof.
      //   if (get_count_in_epoch(miner_addr) == 0) { 
      //     // first proof in this epoch, can be either the previous difficulty or the current one
      //     let is_diff = &proof.difficulty == &diff.difficulty ||
      //                   &proof.difficulty == &diff.prev_diff;

      //     let is_sec = &proof.security == &diff.security ||
      //                  &proof.security == &diff.prev_sec;

      //     assert!(is_diff, error::invalid_argument(130102));
      //     assert!(is_sec, error::invalid_argument(13010202));
      //   } else {
      //     assert!(&proof.difficulty == &diff.difficulty, error::invalid_argument(130102));
      //     assert!(&proof.security == &diff.security, error::invalid_argument(13010202));
      //   };
      // };

      // Process the proof
      verify_and_update_state(miner_addr, proof, true);
    }

    /// The entry point to commit miner state.
    public entry fun minerstate_commit(
        sender: signer,
        challenge: vector<u8>, 
        solution: vector<u8>,
        difficulty: u64,
        security: u64,
    ) acquires TowerCounter, TowerList, TowerProofHistory {
        let proof = create_proof_blob(
            challenge,
            solution,
            difficulty,
            security,
        );

        commit_state(&sender, proof);
    }


    // // This function is called by the OPERATOR associated with node,
    // // it verifies the proof and commits to chain.
    // // Function index: 02
    // // Permissions: PUBLIC, ANYONE
    // public fun commit_state_by_operator(
    //   operator_sig: &signer,
    //   miner_addr: address,
    //   proof: Proof
    // ) acquires TowerProofHistory, TowerList, TowerCounter, VDFDifficulty {

    //   // Check the signer is in fact an operator delegated by the owner.
      
    //   // Get address, assumes the sender is the signer.
    //   assert!(ValidatorConfig::get_operator(miner_addr) == signer::address_of(operator_sig), error::requires_role(130103));
    //   // Abort if not initialized. Assumes the validator Owner account
    //   // already has submitted the 0th miner proof in onboarding.
    //   assert!(exists<TowerProofHistory>(miner_addr), error::not_published(130104));

    //   // Return early if difficulty and security are not correct.
    //   // Check vdf difficulty constant. Will be different in tests than in production.
    //   // Skip this check on local tests, we need tests to send differentdifficulties.
    //   check_difficulty(miner_addr, &proof);

    //   // Process the proof
    //   verify_and_update_state(miner_addr, proof, true);
    // }

    fun check_difficulty(miner_addr: address, proof: &Proof) acquires TowerProofHistory, VDFDifficulty {
      if (!testnet::is_testnet()){
        // Get vdf difficulty constant. Will be different in tests than in production.ex
        let diff = borrow_global<VDFDifficulty>(@ol_framework );

        // need to also give allowance for user's first proof in epoch to be in the last proof.
        if (get_count_in_epoch(miner_addr) == 0) { 
          // first proof in this epoch, can be either the previous difficulty or the current one
          let is_diff = &proof.difficulty == &diff.difficulty ||
          &proof.difficulty == &diff.prev_diff;

          let is_sec = &proof.security == &diff.security ||
          &proof.security == &diff.prev_sec;

          assert!(is_diff, error::invalid_argument(130102));
          assert!(is_sec, error::invalid_argument(13010202));
        } else {
          assert!(&proof.difficulty == &diff.difficulty, error::invalid_argument(130102));
          assert!(&proof.security == &diff.security, error::invalid_argument(13010202));
        };
      };
    }    

    // Function to verify a proof blob and update a TowerProofHistory
    // Permissions: private function.
    // Function index: 03
    fun verify_and_update_state(
      miner_addr: address,
      proof: Proof,
      steady_state: bool
    ) acquires TowerProofHistory, TowerList, TowerCounter {
      // instead of looping through all miners at end of epcoh the stats are
      // only reset when the miner submits a new proof.
      lazy_reset_count_in_epoch(miner_addr);

      assert!(
        get_count_in_epoch(miner_addr) < globals::get_epoch_mining_thres_upper(), 
        error::invalid_state(130108)
      );

      let miner_history = borrow_global<TowerProofHistory>(miner_addr);

      // If not genesis proof, check hash to ensure the proof continues the chain
      if (steady_state) {
        //If not genesis proof, check hash 
        assert!(&proof.challenge == &miner_history.previous_proof_hash,
        error::invalid_state(EDELAY_NOT_CHAINED));      
      };

      let wesolowski_algo = false;
      let valid = ol_native_vdf::verify(&proof.challenge, &proof.solution, proof.difficulty, proof.security, wesolowski_algo);
      assert!(valid, error::out_of_range(EPROOF_NOT_VALID));

      // add the miner to the miner list if not present
      increment_miners_list(miner_addr);

      // Get a mutable ref to the current state
      let miner_history = borrow_global_mut<TowerProofHistory>(miner_addr);

      // update the miner proof history (result is used as seed for next proof)
      miner_history.previous_proof_hash = hash::sha3_256(*&proof.solution);
    
      // Increment the verified_tower_height
      if (steady_state) {
        miner_history.verified_tower_height = miner_history.verified_tower_height + 1;
        miner_history.count_proofs_in_epoch = miner_history.count_proofs_in_epoch + 1;
      } else {
        miner_history.verified_tower_height = 0;
        miner_history.count_proofs_in_epoch = 1
      };

      miner_history.latest_epoch_mining = reconfiguration::get_current_epoch();

      increment_stats(miner_addr);
    }

    // Checks that the validator has been mining above the count threshold
    // Note: this is only called on a validator successfully meeting 
    // the validation thresholds (different than mining threshold). 
    // So the function presumes the validator is in good standing for that epoch.
    // Permissions: private function
    // Function index: 04
    fun update_epoch_metrics_vals(account: &signer, miner_addr: address) acquires TowerProofHistory {
      // The goal of update_metrics is to confirm that a miner participated in consensus during
      // an epoch, but also that there were mining proofs submitted in that epoch.
      system_addresses::assert_ol(account);

      // Tower may not have been initialized. 
      // Simply return in this case (don't abort)
      if(!is_init(miner_addr)) { return };

      // Check that there was mining and validating in period.
      // Account may not have any proofs submitted in epoch, since 
      // the resource was last emptied.
      let passed = node_above_thresh(miner_addr);
      let miner_history = borrow_global_mut<TowerProofHistory>(miner_addr);
      // Update statistics.
      if (passed) {
          // let this_epoch = reconfiguration::get_current_epoch();
          // miner_history.latest_epoch_mining = this_epoch; // TODO: Don't need this
          miner_history.epochs_mining 
            = miner_history.epochs_mining + 1u64;
          miner_history.contiguous_epochs_mining 
            = miner_history.contiguous_epochs_mining + 1u64;
          // miner_history.epochs_since_last_account_creation 
          //   = miner_history.epochs_since_last_account_creation + 1u64;
      } else {
        // didn't meet the threshold, reset this count
        miner_history.contiguous_epochs_mining = 0;
      };

      // This is the end of the epoch, reset the count of proofs
      miner_history.count_proofs_in_epoch = 0u64;
    }

    /// Checks to see if miner submitted enough proofs to be considered compliant
    public fun node_above_thresh(miner_addr: address): bool acquires TowerProofHistory {
      get_count_in_epoch(miner_addr) >= globals::get_epoch_mining_thres_lower()
    }

    fun epoch_param_reset(vm: &signer) acquires VDFDifficulty, TowerList, TowerProofHistory  {
      system_addresses::assert_ol(vm);

      let diff = borrow_global_mut<VDFDifficulty>(@ol_framework );

      diff.prev_diff = diff.difficulty;
      diff.prev_sec = diff.security;
      
      // NOTE: For now we are not changing the vdf security params.
      if (testnet::is_testnet()) {
        // VDF proofs must be even numbers.
        let rng =  toy_rng(3, 2);
        if (rng > 0) {
          rng = rng * 2;
        };
        diff.difficulty = globals::get_vdf_difficulty_baseline() + rng;
       
      }
    }    

    // Used at epoch boundary by vm to reset all validator's statistics.
    // Permissions: PUBLIC, ONLY VM.
    public fun reconfig(
      vm: &signer, outgoing_validators: &vector<address>
    ) acquires TowerProofHistory, TowerList, TowerCounter, VDFDifficulty {
      // Check permissions
      system_addresses::assert_ol(vm);

      // update the vdf parameters
      epoch_param_reset(vm);      

      // Iterate through validators and call update_metrics for each validator
      // that had proofs this epoch
      let vals_len = vector::length<address>(outgoing_validators); // TODO: These references are weird
      let i = 0;
      while (i < vals_len) {
          let val = vector::borrow(outgoing_validators, i); 

          // For testing: don't call update_metrics unless there is account state for the address.
          if (exists<TowerProofHistory>(*val)){
              update_epoch_metrics_vals(vm, *val);
          };
          i = i + 1;
      };

      epoch_reset(vm);
      // safety
      if (exists<TowerList>(@0x0)) {
        //reset miner list
        let towerlist_state = borrow_global_mut<TowerList>(@0x0);
        towerlist_state.list = vector::empty<address>();
      };
    }

    // Function to initialize miner state
    // Permissions: PUBLIC, signer, Validator only
    // Function code: 07
    public fun init_miner_state(
      miner_sig: &signer,
      challenge: &vector<u8>,
      solution: &vector<u8>,
      difficulty: u64,
      security: u64
    ) acquires TowerProofHistory, TowerList, TowerCounter {
      
      // NOTE Only signer can update own state.
      // Should only happen once.
      assert!(!exists<TowerProofHistory>(signer::address_of(miner_sig)), error::permission_denied(130111));
      // DiemAccount calls this.
      // Exception is DiemAccount which can simulate a signer.
      // Initialize TowerProofHistory object and give to miner account
      move_to<TowerProofHistory>(miner_sig, TowerProofHistory{
        previous_proof_hash: vector::empty(),
        verified_tower_height: 0u64,
        latest_epoch_mining: 0u64,
        count_proofs_in_epoch: 1u64,
        epochs_mining: 0u64,
        contiguous_epochs_mining: 0u64,
        // epochs_since_last_account_creation: 0u64,
      });
      // create the initial proof submission
      let proof = Proof {
        challenge: *challenge,
        difficulty,  
        solution: *solution,
        security,
      };

      //submit the proof
      verify_and_update_state(signer::address_of(miner_sig), proof, false);
    }

    /// fork tools. Migrate user state
    public fun fork_migrate_user_tower_history(
      vm: &signer,
      miner_sig: &signer,
      previous_proof_hash: vector<u8>,
      verified_tower_height: u64,
      latest_epoch_mining: u64,
      count_proofs_in_epoch: u64,
      epochs_mining: u64,
      contiguous_epochs_mining: u64,
      // epochs_since_last_account_creation: u64,
    ) {
      system_addresses::assert_ol(vm);
      move_to<TowerProofHistory>(miner_sig, TowerProofHistory{
        previous_proof_hash,
        verified_tower_height,
        latest_epoch_mining,
        count_proofs_in_epoch,
        epochs_mining,
        contiguous_epochs_mining,
        // epochs_since_last_account_creation,
      });
    }


    // Process and check the first proof blob submitted for validity (includes correct address)
    // Permissions: PUBLIC, ANYONE. (used in onboarding transaction).
    // Function code: 08
    public fun first_challenge_includes_address(new_account_address: address, challenge: &vector<u8>) {
      // Checks that the preimage/challenge of the FIRST VDF proof blob contains a given address.
      // This is to ensure that the same proof is not sent repeatedly, since all the minerstate is on a
      // the address of a miner.
      // Note: The bytes of the miner challenge is as follows:
      //         32 // 0L Key
      //         +64 // chain_id
      //         +8 // iterations/difficulty
      //         +1024; // statement

      // Calling native function to do this parsing in rust
      // The auth_key must be at least 32 bytes long
      assert!(vector::length(challenge) >= 32, error::invalid_argument(130113));
      let (parsed_address, _auth_key) = ol_native_vdf::extract_address_from_challenge(challenge);

      print(&parsed_address);
      print(&new_account_address);
      // Confirm the address is corect and included in challenge
      assert!(new_account_address == parsed_address, error::permission_denied(130114));
    }

    // Get latest epoch mined by node on given address
    // Permissions: public ony VM can call this function.
    // Function code: 09
    public fun get_miner_latest_epoch(addr: address): u64 acquires TowerProofHistory {
      let addr_state = borrow_global<TowerProofHistory>(addr);
      *&addr_state.latest_epoch_mining
    }

    // // Function to reset the timer for when an account can be created 
    // // must be signed by the account being reset 
    // // done as a part of the creation of new accounts. 
    // public fun reset_rate_limit(miner: &signer) acquires TowerProofHistory {
    //   let state = borrow_global_mut<TowerProofHistory>(signer::address_of(miner));
    //   state.epochs_since_last_account_creation = 0;
    // }

    fun increment_stats(miner_addr: address) acquires TowerProofHistory, TowerCounter {
      // safety. Don't cause VM to halt
      if (!exists<TowerCounter>(@ol_framework)) return;

      let above = node_above_thresh(miner_addr);

      let state = borrow_global_mut<TowerCounter>(@ol_framework);

      // if (ValidatorConfig::is_valid(miner_addr)) {
      //   state.validator_proofs_in_epoch = state.validator_proofs_in_epoch + 1;
      //   state.lifetime_validator_proofs = state.lifetime_validator_proofs + 1;
      //   // only proofs above threshold are counted here. The preceding proofs are not counted;
      if (above) { state.validator_proofs_in_epoch_above_thresh = state.validator_proofs_in_epoch_above_thresh + 1; };

      state.proofs_in_epoch = state.proofs_in_epoch + 1;
      state.lifetime_proofs = state.lifetime_proofs + 1;
    }

    /// Reset the tower counter at the end of epoch.
    public fun epoch_reset(vm: &signer) acquires TowerCounter {
      system_addresses::assert_vm(vm);
      if (!exists<TowerCounter>(@ol_framework)) return;

      let state = borrow_global_mut<TowerCounter>(@ol_framework);
      state.proofs_in_epoch = 0;
      state.validator_proofs_in_epoch = 0;
      state.fullnode_proofs_in_epoch = 0;
      state.validator_proofs_in_epoch_above_thresh = 0;
      state.fullnode_proofs_in_epoch_above_thresh = 0;
    }

    //////////////////////
    //  Experimental  //
    /////////////////////


    // EXPERIMENTAL: this is a test to see if we can get a plausible RNG from the VDF proofs, to use in low-stakes scenarios.
    // THIS IS AN EXPERIMENT, IT WILL BREAK. DON'T USE FOR ANYTHING SERIOUS.
    // We want to see where it breaks.
    // the first use case is to change the VDF difficulty parameter by tiny margins, in order to make it difficult to stockpile VDFs in a previous epoch, but not change the security properties.
    // the goal is to push all the RNG work to all the tower miners in the network, and minimize compute on the Move side
    // use aptos_framework::Debug::print;

    public fun toy_rng(seed: u64, iters: u64): u64 acquires TowerList, TowerProofHistory {
      // Get the list of all miners L
      // Pick a tower miner  (M) from the seed position 1/(N) of the list of miners.

      let l = get_miner_list();
      // the length will keep incrementing through the epoch. The last miner can know what the starting position will be. There could be a race to be the last validator to augment the set and bias the initial shuffle.
      let len = vector::length(&l);
      if (len == 0) return 0;

      // start n with the seed index
      let n = seed;

      let i = 0;
      while (i < iters) {
        // make sure we get an n smaller than list of validators
        // abort if loops too much
        let k = 0;
        while (n > len) {
          if (k > 1000) return 0;
          n = n / len;
          k = k + 1;
        };
        // double check
        if (len <= n) return 0;

        // print(&666602);
        let miner_addr = vector::borrow<address>(&l, n);
  
        // print(&666603);
        let vec = if (exists<TowerProofHistory>(*miner_addr)) {
          *&borrow_global<TowerProofHistory>(*miner_addr).previous_proof_hash
        } else { return 0 };

        // print(&vec);

        // print(&666604);
        // take the last bit (B) from their last proof hash.

        n = (vector::pop_back(&mut vec) as u64);
        // print(&666605);
        i = i + 1;
      };
      // print(&8888);

      n
    }

    //////////////////////
    //      Getters     //
    /////////////////////

    #[view]
    /// Returns number of epochs for input miner's state
    public fun get_miner_list(): vector<address> acquires TowerList {
      if (!exists<TowerList>(@0x0)) {
        return vector::empty<address>()  
      };
      *&borrow_global<TowerList>(@0x0).list
    }

    #[view]
    /// Returns number of epochs for input miner's state
    public fun get_tower_height(node_addr: address): u64 acquires TowerProofHistory {
      if (exists<TowerProofHistory>(node_addr)) {
        return borrow_global<TowerProofHistory>(node_addr).verified_tower_height
      };
      0
    }

    #[view]
    /// Returns number of epochs user successfully mined AND validated

    public fun get_epochs_compliant(node_addr: address): u64 acquires TowerProofHistory {
      if (exists<TowerProofHistory>(node_addr)) {
        return borrow_global<TowerProofHistory>(node_addr).epochs_mining
      };
      0
    }

    #[view]
    /// returns the number of proofs for a miner in the current epoch
    public fun get_count_in_epoch(miner_addr: address): u64 acquires TowerProofHistory {
      if (exists<TowerProofHistory>(miner_addr)) {
        let s = borrow_global<TowerProofHistory>(miner_addr);
        if (s.latest_epoch_mining == reconfiguration::get_current_epoch()) {
          return s.count_proofs_in_epoch
        };
      };
      0
    }

    #[view]
    /// returns the number of proofs for a miner in the current epoch in EXCESS Of the the threshold
    public fun get_count_above_thresh_in_epoch(miner_addr: address): u64 acquires TowerProofHistory {
      if (exists<TowerProofHistory>(miner_addr)) {
        if (borrow_global<TowerProofHistory>(miner_addr).count_proofs_in_epoch > globals::get_epoch_mining_thres_lower()) {
          return borrow_global<TowerProofHistory>(miner_addr).count_proofs_in_epoch - globals::get_epoch_mining_thres_lower()
        }
      };
      0
    }
    // lazily reset proofs_in_epoch intead of looping through list.
    // danger: this is a private function. Do not make public.
    fun lazy_reset_count_in_epoch(miner_addr: address) acquires TowerProofHistory {
      let s = borrow_global_mut<TowerProofHistory>(miner_addr);
      if (s.latest_epoch_mining < reconfiguration::get_current_epoch()) {
        s.count_proofs_in_epoch = 0;
      };
    }

    // // Returns if the miner is above the account creation rate-limit
    // // Permissions: PUBLIC, ANYONE
    // public fun can_create_val_account(node_addr: address): bool acquires TowerProofHistory {
    //   if(testnet::is_testnet() || testnet::is_staging_net()) return true;
    //   // check if rate limited, needs 7 epochs of validating.
    //   if (exists<TowerProofHistory>(node_addr)) { 
    //     return 
    //       borrow_global<TowerProofHistory>(node_addr).epochs_since_last_account_creation 
    //       >= EPOCHS_UNTIL_ACCOUNT_CREATION
    //   };
    //   false 
    // }

    // #[view]
    // /// 
    // public fun get_validator_proofs_in_epoch(): u64 acquires TowerCounter{
    //   let state = borrow_global<TowerCounter>(@ol_framework);
    //   state.validator_proofs_in_epoch
    // }

    // public fun get_fullnode_proofs_in_epoch(): u64 acquires TowerCounter{
    //   let state = borrow_global<TowerCounter>(@ol_framework);
    //   state.fullnode_proofs_in_epoch
    // }

    // public fun get_fullnode_proofs_in_epoch_above_thresh(): u64 acquires TowerCounter{
    //   let state = borrow_global<TowerCounter>(@ol_framework);
    //   state.fullnode_proofs_in_epoch_above_thresh
    // }

    #[view]
    /// number of proof submitted over lifetime of chain
    public fun get_lifetime_proof_count(): u64 acquires TowerCounter{
      let s = borrow_global<TowerCounter>(@ol_framework);
      s.lifetime_proofs
    }

    // public fun danger_migrate_get_lifetime_proof_count(): (u64, u64, u64) acquires TowerStats{
    //   if (exists<TowerStats>(@ol_framework)) {
    //     let s = borrow_global<TowerStats>(@ol_framework);
    //     return (s.proofs_in_epoch, s.validator_proofs, s.fullnode_proofs)
    //   };
    //   (0,0,0)
    // }

    #[view]
    /// returns the current difficulty and security in a tuple (difficulty, security)
    public fun get_difficulty(): (u64, u64) acquires VDFDifficulty {
      if (exists<VDFDifficulty>(@ol_framework )) {
        let v = borrow_global_mut<VDFDifficulty>(@ol_framework );
        return (v.difficulty, v.security)
      } else {
        // we are probably in the middle of a migration
        (5000000, 512)
      }
    }

    //////////////////
    // TEST HELPERS //
    //////////////////

    // Initiates a miner for a testnet
    // Function index: 10
    // Permissions: PUBLIC, SIGNER, TEST ONLY
    public fun test_helper_init_val(
        miner_sig: &signer,
        challenge: vector<u8>,
        solution: vector<u8>,
        difficulty: u64,
        security: u64,
      ) acquires TowerProofHistory, TowerList, TowerCounter {
        assert!(testnet::is_testnet(), 130102014010);

        move_to<TowerProofHistory>(miner_sig, TowerProofHistory {
          previous_proof_hash: vector::empty(),
          verified_tower_height: 0u64,
          latest_epoch_mining: 0u64,
          count_proofs_in_epoch: 0u64,
          epochs_mining: 1u64,
          contiguous_epochs_mining: 0u64,
          // epochs_since_last_account_creation: 10u64, // is not rate-limited
        });

        // Needs difficulty to test between easy and hard mode.
        let proof = Proof {
          challenge,
          difficulty,
          solution,
          security,
        };

        verify_and_update_state(signer::address_of(miner_sig), proof, false);
        // FullnodeState::init(miner_sig);
    }

    // Note: Used only in tests
    public fun test_epoch_reset_counter(vm: &signer) acquires TowerCounter {
      assert!(testnet::is_testnet(), error::invalid_state(130118));
      system_addresses::assert_vm(vm);
      let state = borrow_global_mut<TowerCounter>(@ol_framework);
      state.lifetime_proofs = 0;
      state.lifetime_validator_proofs = 0;
      state.lifetime_fullnode_proofs = 0;
      state.proofs_in_epoch = 0;
      state.validator_proofs_in_epoch = 0;
      state.fullnode_proofs_in_epoch = 0;
      state.validator_proofs_in_epoch_above_thresh = 0;
      state.fullnode_proofs_in_epoch_above_thresh = 0;
    }

    // // Function index: 11
    // // provides a different method to submit from the operator for use in tests 
    // // where the operator cannot sign a transaction
    // // Permissions: PUBLIC, SIGNER, TEST ONLY
    // public fun test_helper_operator_submits(
    //   operator_addr: address, // Testrunner does not allow arbitrary accounts 
    //                           // to submit txs, need to use address, so this will 
    //                           // differ slightly from api
    //   miner_addr: address,
    //   proof: Proof
    // ) acquires TowerProofHistory, TowerList, TowerCounter {
      // assert!(testnet::is_testnet(), 130102014010);
      // 
    //   // Get address, assumes the sender is the signer.
    //   assert!(
    //     ValidatorConfig::get_operator(miner_addr) == operator_addr,
    //     error::requires_address(130119)
    //   );
    //   // Abort if not initialized.
    //   assert!(exists<TowerProofHistory>(miner_addr), error::not_published(130116));

    //   // Check vdf difficulty constant. Will be different in tests than in production.
    //   // Skip this check on local tests, we need tests to send different difficulties.
    //   if (!testnet::is_testnet()){ // todo: remove?
    //     assert!(&proof.difficulty
    //       == &globals::get_vdf_difficulty_baseline(), error::invalid_state(130117));
    //   };

    //   verify_and_update_state(miner_addr, proof, true);
      
    //   // TODO: The operator mining needs its own struct to count mining.
    //   // For now it is implicit there is only 1 operator per validator,
    //   // and that the fullnode state is the place to count.
    //   // This will require a breaking change to TowerState
    //   // FullnodeState::inc_proof_by_operator(operator_sig, miner_addr);
    // }

    #[test_only]
    // Use in testing to mock mining without producing proofs
    public fun test_helper_mock_mining(sender: &signer,  count: u64) acquires TowerProofHistory, TowerCounter {
      assert!(testnet::is_testnet(), error::invalid_state(130118));
      let addr = signer::address_of(sender);
      danger_mock_mining(addr, count)
    }


    #[test_only]
    // mocks mining for an arbitrary account from the vm 
    public fun test_helper_mock_mining_vm(vm: &signer, addr: address, count: u64) acquires TowerProofHistory, TowerCounter {
      assert!(testnet::is_testnet(), error::invalid_state(130120));
      system_addresses::assert_ol(vm);
      danger_mock_mining(addr, count)
    }

    #[test_only]
    fun danger_mock_mining(addr: address, count: u64) acquires TowerProofHistory, TowerCounter {
      // again for safety
      assert!(testnet::is_testnet(), error::invalid_state(130118));


      let i = 0;
      while (i < count) {
        increment_stats(addr);
        let state = borrow_global_mut<TowerProofHistory>(addr);
        // mock verify_and_update
        state.verified_tower_height = state.verified_tower_height + 1;
        state.count_proofs_in_epoch = state.count_proofs_in_epoch + 1;
        i = i + 1;
      };

      let state = borrow_global_mut<TowerProofHistory>(addr);
      state.count_proofs_in_epoch = count;
      state.latest_epoch_mining = reconfiguration::get_current_epoch();
    }

    #[test_only]
    // Get the vm to trigger a reconfig for testing
    public fun test_helper_mock_reconfig(account: &signer, miner_addr: address) acquires TowerProofHistory, TowerCounter {
      system_addresses::assert_ol(account);
      assert!(testnet::is_testnet(), error::invalid_state(130122));
      // update_metrics(account, miner_addr);
      epoch_reset(account);
      update_epoch_metrics_vals(account, miner_addr);
    }

    #[test_only]
    // Get weight of validator identified by address
    public fun test_helper_get_height(miner_addr: address): u64 acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130123));
      assert!(exists<TowerProofHistory>(miner_addr), error::not_found(130124));

      let state = borrow_global<TowerProofHistory>(miner_addr);
      *&state.verified_tower_height
    }

    #[test_only]
    // TODO: remove this and replace tests with get_count_in_epoch
    public fun test_helper_get_count(account: &signer): u64 acquires TowerProofHistory {
        assert!(testnet::is_testnet(), 130115014011);
        let addr = signer::address_of(account);
        get_count_in_epoch(addr)
    }

    #[test_only]
    public fun test_helper_get_nominal_count(miner_addr: address): u64 acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130123));
      assert!(exists<TowerProofHistory>(miner_addr), error::not_found(130124));

      let state = borrow_global<TowerProofHistory>(miner_addr);
      *&state.count_proofs_in_epoch
    }


    #[test_only]
    public fun test_helper_get_contiguous_vm(vm: &signer, miner_addr: address): u64 acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130125));
      system_addresses::assert_ol(vm);
      borrow_global<TowerProofHistory>(miner_addr).contiguous_epochs_mining
    }


    // #[test_only]
    // // Sets the epochs since last account creation variable to allow `account`
    // // to create a new account
    // public fun test_helper_set_rate_limit(account: &signer, value: u64) acquires TowerProofHistory {
    //   assert!(testnet::is_testnet(), error::invalid_state(130126));
    //   let addr = signer::address_of(account);
    //   let state = borrow_global_mut<TowerProofHistory>(addr);
    //   state.epochs_since_last_account_creation = value;
    // }

    #[test_only]
    public fun test_helper_set_epochs_mining(node_addr: address, value: u64) acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130126));

      let s = borrow_global_mut<TowerProofHistory>(node_addr);
      s.epochs_mining = value;
    }

    #[test_only]
    public fun test_helper_set_proofs_in_epoch(node_addr: address, value: u64) acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130126));

      let s = borrow_global_mut<TowerProofHistory>(node_addr);
      s.count_proofs_in_epoch = value;
    }


    #[test_only]
    // returns the previous proof hash for `account`
    public fun test_helper_previous_proof_hash(
      account: &signer
    ): vector<u8> acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130128));
      let addr = signer::address_of(account);
      *&borrow_global<TowerProofHistory>(addr).previous_proof_hash
    }

    #[test_only]
    public fun test_helper_set_weight_vm(vm: &signer, addr: address, weight: u64) acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130113));
      system_addresses::assert_ol(vm);
      let state = borrow_global_mut<TowerProofHistory>(addr);
      state.verified_tower_height = weight;
    }

    #[test_only]
    public fun test_helper_set_weight(account: &signer, weight: u64) acquires TowerProofHistory {
      assert!(testnet::is_testnet(), error::invalid_state(130113));
      let addr = signer::address_of(account);
      let state = borrow_global_mut<TowerProofHistory>(addr);
      state.verified_tower_height = weight;
    }

    #[test_only]
    public fun test_mock_depr_tower_stats(vm: &signer) {
      assert!(testnet::is_testnet(), error::invalid_state(130113));
      system_addresses::assert_vm(vm);
      move_to<TowerStats>(vm, TowerStats{
        proofs_in_epoch: 111,
        validator_proofs: 222,
        fullnode_proofs: 333,
      });
    }

    #[test_only]
    public fun test_get_liftime_proofs(): u64 acquires TowerCounter {
      assert!(testnet::is_testnet(), error::invalid_state(130113));
      
      let s = borrow_global<TowerCounter>(@ol_framework);
      s.lifetime_proofs
    }

    #[test_only]
    public fun test_set_vdf_difficulty(vm: &signer, diff: u64, sec: u64) acquires VDFDifficulty {
      assert!(testnet::is_testnet(), error::invalid_state(130113));
      system_addresses::assert_vm(vm);
      
      let s = borrow_global_mut<VDFDifficulty>(@ol_framework );
      s.difficulty = diff;
      s.security = sec;
    }    

    #[test_only]
    public fun test_danger_destroy_tower_counter(vm: &signer) acquires TowerCounter {
      assert!(testnet::is_testnet(), error::invalid_state(130113));
      system_addresses::assert_vm(vm);
      assert!(exists<TowerCounter>(@ol_framework), error::invalid_state(130115));
        
        // We destroy the data resource for sender
        // move_from and then destructure

        let TowerCounter {
          lifetime_proofs: _,
          lifetime_validator_proofs: _,
          lifetime_fullnode_proofs: _,
          proofs_in_epoch: _,
          validator_proofs_in_epoch: _,
          fullnode_proofs_in_epoch: _,
          validator_proofs_in_epoch_above_thresh: _,
          fullnode_proofs_in_epoch_above_thresh: _,
       } = move_from<TowerCounter>(@ol_framework);
    }

    //////// TESTS ////////

    #[test]
    fun parse_challenge() {
        // First 32 bytes (64 hex characters) make up the auth_key. Of this,
        // the first 16 bytes (32 hex characters) make up the auth_key pefix
        // the last 16 bytes of the auth_key make up the account address
        // The native function implemented in Rust parses this and gives out the
        // address. This is then confirmed in the the TowerState module (move-space)
        // to be the same address as the one passed in

        let challenge = x"232fb6ae7221c853232fb6ae7221c853000000000000000000000000deadbeef";
        let new_account_address = @0x232fb6ae7221c853232fb6ae7221c853000000000000000000000000deadbeef;

        // Parse key and check
        first_challenge_includes_address(new_account_address, &challenge);
        // Note: There is a assert statement in this function already
        // which checks to confim that the parsed address and new_account_address
        // the same. Execution of this guarantees that the test of the native
        // function passed.

        challenge = x"232fb6ae7221c853232fb6ae7221c853000000000000000000000000deadbeef00000000000000000000000000000000000000000000000000000000000000000000000000004f6c20746573746e6574640000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000070726f74657374732072616765206163726f737320416d6572696361";

        let new_account_address = @0x232fb6ae7221c853232fb6ae7221c853000000000000000000000000deadbeef;
        first_challenge_includes_address(new_account_address, &challenge);
    }

}
