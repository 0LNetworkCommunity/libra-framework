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
  use aptos_framework::stake;
  // use aptos_std::debug::print;

  // friend aptos_framework::stake;
  
  // resource for tracking the universe of accounts that have submitted 
  // a mined proof correctly, with the epoch number.
  struct ValidatorUniverse has key {
      validators: vector<address>
  }

  // // deprecated
  // struct JailedBit has key {
  //     is_jailed: bool
  // }

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

  // abstract this from stake.move
  public fun register_validator(
    account: &signer,
    consensus_pubkey: vector<u8>,
    proof_of_possession: vector<u8>,
    network_addresses: vector<u8>,
    fullnode_addresses: vector<u8>,
  ) acquires ValidatorUniverse {


      // let owner_address = signer::address_of(account);

      stake::initialize_validator(account, consensus_pubkey, proof_of_possession, network_addresses, fullnode_addresses);
      // 0L specific,
      add(account);
      jail::init(account);
  }

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
    jail::init(validator);

  }
  /// This function is called to add validator to the validator universe.
  /// it can only be called by `stake` module, on validator registration.
  fun add(sender: &signer) acquires ValidatorUniverse {
    let addr = signer::address_of(sender);
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    let (in_set, _) = vector::index_of<address>(&state.validators, &addr);
    if (!in_set) {
      let state = borrow_global_mut<ValidatorUniverse>(@aptos_framework);
      vector::push_back<address>(&mut state.validators, addr);
      // unjail(sender);
    }
  }





  // A simple public function to query the EligibleValidators.
  // Function code: 03 Prefix: 220103
  public fun get_eligible_validators(): vector<address> acquires ValidatorUniverse {
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    *&state.validators
  }

  // Is a candidate for validation
  public fun is_in_universe(addr: address): bool acquires ValidatorUniverse {
    let state = borrow_global<ValidatorUniverse>(@aptos_framework);
    vector::contains<address>(&state.validators, &addr)
  }
  // * removed deprecated v3 jail implementation *//

  // Todo: Better name? genesis_helper_add_validator()?
  public fun genesis_helper(vm: &signer, validator: &signer) acquires ValidatorUniverse {
    assert!(signer::address_of(vm) == @aptos_framework, 220101014010);
    add(validator);
  }

  //////// TEST ////////

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