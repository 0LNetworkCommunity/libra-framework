// ValidatorUniverse
///////////////////////////////////////////////////////////////////////////

module diem_framework::validator_universe {
  use std::signer;
  use std::vector;
  use diem_framework::system_addresses;
  use ol_framework::jail;
  use ol_framework::vouch;
  use diem_framework::stake;

  #[test_only]
  use ol_framework::testnet;
  #[test_only]
  use diem_std::bls12381;

  // use diem_std::debug::print;

  friend diem_framework::reconfiguration;

  // resource for tracking the universe of accounts that have submitted
  // a mined proof correctly, with the epoch number.
  struct ValidatorUniverse has key {
      validators: vector<address>
  }

  // Genesis function to initialize ValidatorUniverse struct in 0x0.
  // This is triggered in new epoch by Configuration in Genesis.move
  // Function code: 01 Prefix: 220101
  public fun initialize(vm: &signer){
    // Check for transactions sender is association
    system_addresses::assert_diem_framework(vm);
    move_to<ValidatorUniverse>(vm, ValidatorUniverse {
        validators: vector::empty<address>()
    });
  }

  /// This is the entrypoint for a validator joining the network.
  /// Separates the logic of registration from validator election etc. (in stake.move).
  /// This prevents dependency cycling issues, since stake.move is a large module.
  public entry fun register_validator(
    account: &signer,
    consensus_pubkey: vector<u8>,
    proof_of_possession: vector<u8>,
    network_addresses: vector<u8>,
    fullnode_addresses: vector<u8>,
  ) acquires ValidatorUniverse {
      stake::initialize_validator(account, consensus_pubkey, proof_of_possession, network_addresses, fullnode_addresses);
      vouch::init(account);
      // 0L specific,
      add(account);
      jail::init(account);
  }

  /// This function is called to add validator to the validator universe.
  fun add(sender: &signer) acquires ValidatorUniverse {
    let addr = signer::address_of(sender);
    let state = borrow_global<ValidatorUniverse>(@diem_framework);
    let (elegible_list, _) = vector::index_of<address>(&state.validators, &addr);
    if (!elegible_list) {
      let state = borrow_global_mut<ValidatorUniverse>(@diem_framework);
      vector::push_back<address>(&mut state.validators, addr);
    };
    jail::init(sender);
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
    let state = borrow_global<ValidatorUniverse>(@diem_framework);
    *&state.validators
  }

  // Is a candidate for validation
  #[view]
  public fun is_in_universe(addr: address): bool acquires ValidatorUniverse {
    let state = borrow_global<ValidatorUniverse>(@diem_framework);
    vector::contains<address>(&state.validators, &addr)
  }
  // *  NOTE removed deprecated v3 jail implementation *//


  //////// TEST HELPERS ////////

  #[test_only]
  public fun test_register_validator(
    root: &signer,
    public_key: &bls12381::PublicKey,
    proof_of_possession: &bls12381::ProofOfPossession,
    validator: &signer,
    _amount: u64,
    should_join_validator_set: bool,
    should_end_epoch: bool,
  ) acquires ValidatorUniverse {
    system_addresses::assert_ol(root);
    assert!(testnet::is_testnet(), 220101014014);
    stake::initialize_test_validator(root, public_key, proof_of_possession, validator, _amount, should_join_validator_set, should_end_epoch);

    add(validator);
  }

  #[test_only]
  public fun test_helper_add_self_onboard(vm: &signer, addr:address) acquires ValidatorUniverse {
    assert!(testnet::is_testnet(), 220101014014);
    assert!(signer::address_of(vm) == @diem_framework, 220101015010);
    let state = borrow_global_mut<ValidatorUniverse>(@diem_framework);
    vector::push_back<address>(&mut state.validators, addr);
  }

  #[test_only]
  /// Validator universe is append only, only in tests remove self from validator list.
  public fun remove_self(validator: &signer) acquires ValidatorUniverse {
    assert!(testnet::is_testnet(), 220101014014);
    let val = signer::address_of(validator);
    let state = borrow_global<ValidatorUniverse>(@diem_framework);
    let (in_set, index) = vector::index_of<address>(&state.validators, &val);
    if (in_set) {
        let state = borrow_global_mut<ValidatorUniverse>(@diem_framework);
      vector::remove<address>(&mut state.validators, index);
    }
  }
}
