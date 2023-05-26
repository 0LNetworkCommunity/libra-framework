/////////////////////////////////////////////////////////////////////////
// 0L Module
// Stats Module
// Error code: 1900
/////////////////////////////////////////////////////////////////////////

module ol_framework::stats {
  use std::error;
  use std::fixed_point32;
  use std::signer;
  use ol_framework::testnet;
  use ol_framework::globals;
  use std::vector;

  // TODO: yes we know this slows down block production. In "make it fast"
  // mode this will be moved to Rust, in the vm execution block prologue. TBD.
  
  struct SetData has copy, drop, store {
    addr: vector<address>,
    prop_count: vector<u64>,
    vote_count: vector<u64>,
    total_votes: u64,
    total_props: u64,
  }

  struct ValStats has copy, drop, key {
    history: vector<SetData>,
    current: SetData
  }

  //Permissions: Public, VM only.
  //Function: 01
  public fun initialize(vm: &signer) {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190001));
    move_to<ValStats>(
      vm, 
      ValStats {
        history: vector::empty(),
        current: blank()
      }
    );
  }
  
  fun blank():SetData {
    SetData {
      addr: vector::empty(),
      prop_count: vector::empty(),
      vote_count: vector::empty(),
      total_votes: 0,
      total_props: 0,
    }
  }

  //Permissions: Public, VM only.
  //Function: 02
  public fun init_address(vm: &signer, node_addr: address) acquires ValStats {
    let sender = signer::address_of(vm);

    assert!(sender == @ol_root, error::permission_denied(190002));

    let stats = borrow_global<ValStats>(sender);
    let (is_init, _) = vector::index_of<address>(&stats.current.addr, &node_addr);
    if (!is_init) {
      let stats = borrow_global_mut<ValStats>(sender);
      vector::push_back(&mut stats.current.addr, node_addr);
      vector::push_back(&mut stats.current.prop_count, 0);
      vector::push_back(&mut stats.current.vote_count, 0);
    }
  }

  //Function: 03
  public fun init_set(vm: &signer, set: &vector<address>) acquires ValStats{
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190003));
    let length = vector::length<address>(set);
    let k = 0;
    while (k < length) {
      let node_address = *(vector::borrow<address>(set, k));
      init_address(vm, node_address);
      k = k + 1;
    }
  }

  //Function: 04
  public fun process_set_votes(vm: &signer, set: &vector<address>) acquires ValStats{
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190004));

    let length = vector::length<address>(set);
    let k = 0;
    while (k < length) {
      let node_address = *(vector::borrow<address>(set, k));
      inc_vote(vm, node_address);
      k = k + 1;
    }
  }

  //Permissions: Public, VM only.
  //Function: 05
  public fun node_current_votes(vm: &signer, node_addr: address): u64 acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190005));
    let stats = borrow_global_mut<ValStats>(sender);
    let (is_found, i) = vector::index_of<address>(&mut stats.current.addr, &node_addr);
    if (is_found) return *vector::borrow<u64>(&mut stats.current.vote_count, i)
    else 0
  }

  //Function: 06
  public fun node_above_thresh(
    vm: &signer, node_addr: address, height_start: u64, height_end: u64
  ): bool acquires ValStats{
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190006));
    let range = height_end-height_start;
    // TODO: Change to 5 percent
    let threshold_signing = fixed_point32::multiply_u64(
      range, 
      fixed_point32::create_from_rational(globals::get_signing_threshold(), 100)
    );
    if (node_current_votes(vm, node_addr) >  threshold_signing) { return true };
    return false
  }

  //Function: 07
  public fun network_density(
    vm: &signer, height_start: u64, height_end: u64
  ): u64 acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190007));
    let density = 0u64;
    let nodes = *&(borrow_global_mut<ValStats>(sender).current.addr);
    let len = vector::length(&nodes);
    let k = 0;
    while (k < len) {
      let addr = *(vector::borrow<address>(&nodes, k));
      if (node_above_thresh(vm, addr, height_start, height_end)) {
        density = density + 1;
      };
      k = k + 1;
    };
    return density
  }

  //Permissions: Public, VM only.
  //Function: 08    
  public fun node_current_props(vm: &signer, node_addr: address): u64 acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190008));
    let stats = borrow_global_mut<ValStats>(sender);
    let (is_found, i) = vector::index_of<address>(&mut stats.current.addr, &node_addr);
    if (is_found) return *vector::borrow<u64>(&mut stats.current.prop_count, i)
    else 0
  }

  //Permissions: Public, VM only.
  //Function: 09
  public fun inc_prop(vm: &signer, node_addr: address) acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190009));
    let stats = borrow_global_mut<ValStats>(@ol_root);
    let (is_true, i) = vector::index_of<address>(&mut stats.current.addr, &node_addr);
    // don't try to increment if no state. This has caused issues in the past
    // in emergency recovery.

    if (is_true) {
      let current_count = *vector::borrow<u64>(&mut stats.current.prop_count, i);
      vector::push_back(&mut stats.current.prop_count, current_count + 1);
      vector::swap_remove(&mut stats.current.prop_count, i);
    };

    stats.current.total_props = stats.current.total_props + 1;
  }
  
  //TODO: Duplicate code.
  //Permissions: Public, VM only.
  //Function: 10
  fun inc_vote(vm: &signer, node_addr: address) acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190010));
    let stats = borrow_global_mut<ValStats>(sender);
    let (is_true, i) = vector::index_of<address>(&mut stats.current.addr, &node_addr);
    if (is_true) {
      let test = *vector::borrow<u64>(&mut stats.current.vote_count, i);
      vector::push_back(&mut stats.current.vote_count, test + 1);
      vector::swap_remove(&mut stats.current.vote_count, i);
    } else {
      // debugging rescue mission. Remove after network stabilizes Apr 2022.
      // something bad happened and we can't find this node in our list.
      // // print(&666);
      // // print(&node_addr);
    };
    // update total vote count anyways even if we can't find this person.
    stats.current.total_votes = stats.current.total_votes + 1;
    // // print(&stats.current);
  }

  //Permissions: Public, VM only.
  //Function: 11
  public fun reconfig(vm: &signer, set: &vector<address>) acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190011));
    let stats = borrow_global_mut<ValStats>(sender);
    
    // Keep only the most recent epoch stats
    if (vector::length(&stats.history) > 7) {
      vector::pop_back<SetData>(&mut stats.history); // just drop last record
    };
    vector::push_back(&mut stats.history, *&stats.current);
    stats.current = blank();
    init_set(vm, set);
  }

  //Function: 12
  public fun get_total_votes(vm: &signer): u64 acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190012));
    *&borrow_global<ValStats>(@ol_root).current.total_votes
  }

  //Function: 13
  public fun get_total_props(vm: &signer): u64 acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190013));
    *&borrow_global<ValStats>(@ol_root).current.total_props
  }

  //Function: 14
  public fun get_history(): vector<SetData> acquires ValStats {
    *&borrow_global<ValStats>(@ol_root).history
  }


  // TODO: this code is duplicated with NodeWeight, opportunity to make sorting in to a module.
  public fun get_sorted_vals_by_props(account: &signer, n: u64): vector<address> acquires ValStats {
      assert!(signer::address_of(account) == @ol_root, error::permission_denied(140101));

      //Get all validators from Validator Universe and then find the eligible validators 
      let eligible_validators = 
      *&borrow_global<ValStats>(@ol_root).current.addr;

      let length = vector::length<address>(&eligible_validators);

      // Scenario: The universe of validators is under the limit of the BFT consensus.
      // If n is greater than or equal to accounts vector length - return the vector.
      if(length <= n) return eligible_validators;

      // vector to store each address's node_weight
      let weights = vector::empty<u64>();
      let k = 0;
      while (k < length) {

        let cur_address = *vector::borrow<address>(&eligible_validators, k);
        // Ensure that this address is an active validator
        vector::push_back<u64>(&mut weights, node_current_props(account, cur_address));
        k = k + 1;
      };

      // Sorting the accounts vector based on value (weights).
      // Bubble sort algorithm
      let i = 0;
      while (i < length){
        let j = 0;
        while(j < length-i-1){
          let value_j = *(vector::borrow<u64>(&weights, j));
          let value_jp1 = *(vector::borrow<u64>(&weights, j+1));
          if(value_j > value_jp1){
            vector::swap<u64>(&mut weights, j, j+1);
            vector::swap<address>(&mut eligible_validators, j, j+1);
          };
          j = j + 1;
        };
        i = i + 1;
      };

      // Reverse to have sorted order - high to low.
      vector::reverse<address>(&mut eligible_validators);

      let diff = length - n; 
      while(diff>0){
        vector::pop_back(&mut eligible_validators);
        diff =  diff - 1;
      };

      return eligible_validators
    }


  /// TEST HELPERS
  //Function: 15
  public fun test_helper_inc_vote_addr(vm: &signer, node_addr: address) acquires ValStats {
    let sender = signer::address_of(vm);
    assert!(sender == @ol_root, error::permission_denied(190015));
    assert!(testnet::is_testnet(), error::invalid_state(190015));

    inc_vote(vm, node_addr);
  }

  public fun test_helper_remove_votes(vm: &signer, node_addr: address) acquires ValStats {
    testnet::assert_testnet(vm);

    let stats = borrow_global_mut<ValStats>(@vm_reserved);
    let (is_true, i) = vector::index_of<address>(&mut stats.current.addr, &node_addr);
    if (is_true) {
      let votes = *vector::borrow<u64>(&mut stats.current.vote_count, i);
      vector::push_back(&mut stats.current.vote_count, 0);
      vector::swap_remove(&mut stats.current.vote_count, i);
      stats.current.total_votes = stats.current.total_votes - votes;
    }
  }
}