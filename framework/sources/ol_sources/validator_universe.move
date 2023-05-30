///////////////////////////////////////////////////////////////////////////
// 0L Module
// ValidatorUniverse
///////////////////////////////////////////////////////////////////////////
// Stores all the validators who submitted a vdf proof.
// File Prefix for errors: 2201
///////////////////////////////////////////////////////////////////////////

module aptos_framework::validator_universe {
  use std::signer;
  use std::vector;
  use aptos_framework::system_addresses;
  use ol_framework::jail;
  use ol_framework::cases;
  use aptos_framework::stake;
  // use aptos_std::debug::print;

  friend aptos_framework::reconfiguration;
  
  // resource for tracking the universe of accounts that have submitted 
  // a mined proof correctly, with the epoch number.
  struct ValidatorUniverse has key {
      validators: vector<address>
  }

  // * DEPRECATED JailBit struct, now in jail.move * //

  // Genesis function to initialize ValidatorUniverse struct in 0x0.
  // This is triggered in new epoch by Configuration in Genesis.move
  // Function code: 01 Prefix: 220101
  public fun initialize(vm: &signer){
    // Check for transactions sender is association
    system_addresses::assert_aptos_framework(vm);
    move_to<ValidatorUniverse>(vm, ValidatorUniverse {
        validators: vector::empty<address>()
    });
  }

  /// This is the entrypoint for a validator joining the network.
  /// Separates the logic of registration from validator election etc. (in stake.move).
  /// This prevents dependency cycling issues, since stake.move is a large module.
  public fun register_validator(
    account: &signer,
    consensus_pubkey: vector<u8>,
    proof_of_possession: vector<u8>,
    network_addresses: vector<u8>,
    fullnode_addresses: vector<u8>,
  ) acquires ValidatorUniverse {
      stake::initialize_validator(account, consensus_pubkey, proof_of_possession, network_addresses, fullnode_addresses);
      // 0L specific,
      add(account);
      jail::init(account);
  }


  /// This function is called to add validator to the validator universe.
  /// it can only be called by `stake` module, on validator registration.
  fun add(sender: &signer) acquires ValidatorUniverse {
    let addr = signer::address_of(sender);
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    let (elegible_list, _) = vector::index_of<address>(&state.validators, &addr);
    if (!elegible_list) {
      let state = borrow_global_mut<ValidatorUniverse>(@aptos_framework);
      vector::push_back<address>(&mut state.validators, addr);
    };
    jail::init(sender);
  }

  /// Used at epoch boundaries to evaluate the performance of the validator.
  /// only root can call this, and only by friend modules (reconfiguration). Belt and suspenders.
  public(friend) fun maybe_jail(root: &signer, validator: address): bool {
    maybe_jail_impl(root, validator)
  }

  #[test_only]
  /// test helper for maybe_jail
  public fun test_maybe_jail(root: &signer, validator: address): bool {
    maybe_jail_impl(root, validator)
  }

  /// Common implementation for maybe_jail.
  fun maybe_jail_impl(root: &signer, validator: address): bool {

    if (
      !stake::is_valid(validator) || // check if there are issues with config. belt and suspenders
      cases::get_case(validator) == 4
    
    ) {
      jail::jail(root, validator);
      return true
    };

    false
  }

  //////// GENESIS ////////
  /// For 0L genesis, initialize and add the validators
  /// both root and validator need to sign. This is only possible at genesis.
  public fun genesis_helper_add_validator(root: &signer, validator: &signer) acquires ValidatorUniverse {
    system_addresses::assert_ol(root);
    add(validator);
  }

  //////// GETTERS ////////
  // A simple public function to query the EligibleValidators.
  // Function code: 03 Prefix: 220103
  #[view]
  public fun get_eligible_validators(): vector<address> acquires ValidatorUniverse {
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    *&state.validators
  }

  // Is a candidate for validation
  #[view]
  public fun is_in_universe(addr: address): bool acquires ValidatorUniverse {
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    vector::contains<address>(&state.validators, &addr)
  }
  // *  NOTE removed deprecated v3 jail implementation *//



  //////// TEST HELPERS ////////
  #[test_only]
  use aptos_std::bls12381;

  #[test_only]
  public fun test_register_validator(
    public_key: &bls12381::PublicKey,
    proof_of_possession: &bls12381::ProofOfPossession,
    validator: &signer,
    _amount: u64,
    should_join_validator_set: bool,
    should_end_epoch: bool,
  ) acquires ValidatorUniverse {
    stake::initialize_test_validator(public_key, proof_of_possession, validator, _amount, should_join_validator_set, should_end_epoch);

    add(validator);
  }

  #[test_only]
  use ol_framework::testnet;


  #[test_only]
  public fun test_helper_add_self_onboard(vm: &signer, addr:address) acquires ValidatorUniverse {
    assert!(testnet::is_testnet(), 220101014014);
    assert!(signer::address_of(vm) == @aptos_framework, 220101015010);
    let state = borrow_global_mut<ValidatorUniverse>(@aptos_framework);
    vector::push_back<address>(&mut state.validators, addr);
  }

  #[test_only]
  /// Validator universe is append only, only in tests remove self from validator list.
  public fun remove_self(validator: &signer) acquires ValidatorUniverse {
    let val = signer::address_of(validator);
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    let (in_set, index) = vector::index_of<address>(&state.validators, &val);
    if (in_set) {
        let state = borrow_global_mut<ValidatorUniverse>(@aptos_framework);
      vector::remove<address>(&mut state.validators, index);
    }
  }


}