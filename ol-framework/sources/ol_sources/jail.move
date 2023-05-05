///////////////////////////////////////////////////////////////////////////
// 0L Module
// Jail
///////////////////////////////////////////////////////////////////////////
// Controllers for keeping non-performing validators out of the set.
// File Prefix for errors: 1001
///////////////////////////////////////////////////////////////////////////


// The objective of the jail module is to allow a validator to do necessary 
// maintenanance on the node before attempting to rejoin the validator set 
// before they are ready.
// The jail module also incorporates some non-monetary reputation staking
// features of Vouch, which cannot be included in that module due to code
// cyclical dependencies.
// Summary:
// If Alice fails to validate at the threshold necessary (is either a Case 1, 
// or Case 2), then her Jail state gets updated, with is_jailed=true. This means 
// she cannot re-enter the validator set, until someone that vouched for Alice 
// (Bob) sends an un_jail transaction. Bob has interest in seeing Alice rejoin
// and be performing since Bob's Jail state also gets updated based on Alice's performance.
// On Bob's Jail card there is a lifetime_vouchees_jailed, every time Alice
// (because Bob has vouched for her) gets marked for jail, this number gets 
// incremented on Bob's state too.
// It is recursive. If Alice is vouching for Carol, and Carol gets a Jail term. 
// Then both Alice, and Bob will have lifetime_vouchees_jailed incremented. 
// Even if Carol and Bob never met.
// Not included in this code, but included in Vouch, is the logic for the deposit 
// the Vouchers put on the Vouchee accounts. So besides the permanent mark on 
// the account, Bob will lose the deposit.


module ol_framework::jail {
  use aptos_framework::system_addresses;
  // use DiemFramework::TowerState; // todo v7
  use std::signer;
  use std::vector;
  // use DiemFramework::Vouch; // todo v7

  struct Jail has key {
      is_jailed: bool,
      // number of times the validator was dropped from set. Does not reset.
      lifetime_jailed: u64,
      // number of times a downstream validator this user has vouched for has
      // been jailed.
      // this is recursive. So if a validator I vouched for, vouched for
      // a third validator that failed, this number gets incremented.
      lifetime_vouchees_jailed: u64,
      // validator that was jailed and qualified to enter the set, but fails
      // to complete epoch.
      // this resets as soon as they rejoin successfully.
      // this counter is used for ordering prospective validators entering a set.
      consecutive_failure_to_rejoin: u64,

  }

  public fun init(val_sig: &signer) {
    let addr = signer::address_of(val_sig);
    if (!exists<Jail>(addr)) {
      move_to<Jail>(val_sig, Jail {
        is_jailed: false,
        lifetime_jailed: 0,
        lifetime_vouchees_jailed: 0,
        consecutive_failure_to_rejoin: 0,

      });
    }
  }


  public fun is_jailed(validator: address): bool acquires Jail {
    if (!exists<Jail>(validator)) {
      return false
    };
    borrow_global<Jail>(validator).is_jailed
  }

  public fun jail(vm: &signer, validator: address) acquires Jail{
    system_addresses::assert_vm(vm);
    if (exists<Jail>(validator)) {
      let j = borrow_global_mut<Jail>(validator);
      j.is_jailed = true;
      j.lifetime_jailed = j.lifetime_jailed + 1;
      j.consecutive_failure_to_rejoin = j.consecutive_failure_to_rejoin + 1;
    };

    // inc_voucher_jail(validator); // todo v7
  }

  public fun remove_consecutive_fail(vm: &signer, validator: address) acquires Jail {
    system_addresses::assert_vm(vm);
    if (exists<Jail>(validator)) {
      let j = borrow_global_mut<Jail>(validator);
      j.consecutive_failure_to_rejoin = 0;
    }
  }

  //////// V6 //////
  // V6 NOTE: there's no practical or explicit hurdle that the validator
  // needs to overcome now that we have deprecated Towers.
  // So absent some other check that is objective, we could
  // lean on the Vouch network.
  // There was a plan elsewhere discussed to make a
  // voucher be the only one that can unjail a validator.
  // We can promote that idea in V6. Seems like it fits 
  // well with POF.

  // public fun self_unjail(sender: &signer) acquires Jail {
  //   // only a validator can un-jail themselves.
  //   let self = signer::address_of(sender);

  //   // check the node has been mining before unjailing.
  //   assert!(TowerState::node_above_thresh(self), 100104);
  //   unjail(self);
  // }    

  // todo v7
  // public fun vouch_unjail(sender: &signer, addr: address) acquires Jail {
  //   // only a validator can un-jail themselves.
  //   let voucher = signer::address_of(sender);

  //   let buddies = Vouch::buddies_in_set(addr);
  //   // // print(&buddies);
  //   let (is_found, _idx) = vector::index_of(&buddies, &voucher);
  //   assert!(is_found, 100103);

  //   // // check the node has been mining before unjailing.
  //   // assert!(TowerState::node_above_thresh(addr), 100104);
  //   unjail(addr);
  // }

  fun unjail(addr: address) acquires Jail {
    if (exists<Jail>(addr)) {
      borrow_global_mut<Jail>(addr).is_jailed = false;
    };
  }

  public fun sort_by_jail(vec_address: vector<address>): vector<address> acquires Jail {

    // Sorting the accounts vector based on value (weights).
    // Bubble sort algorithm
    let length = vector::length(&vec_address);

    let i = 0;
    while (i < length){
      let j = 0;
      while(j < length-i-1){

        let value_j = get_failure_to_join(*vector::borrow(&vec_address, j));
        let value_jp1 = get_failure_to_join(*vector::borrow(&vec_address, j + 1));

        if(value_j > value_jp1){
          vector::swap<address>(&mut vec_address, j, j+1);
        };
        j = j + 1;
      };
      i = i + 1;
    };

    vec_address
  }

  // todo v7
  // fun inc_voucher_jail(addr: address) acquires Jail {
  //   let buddies = Vouch::get_buddies(addr);
  //   let i = 0;
  //   while (i < vector::length(&buddies)) {
  //     let voucher = *vector::borrow(&buddies, i);
  //     if (exists<Jail>(voucher)) {
  //       let v = borrow_global_mut<Jail>(voucher);
  //       v.lifetime_vouchees_jailed = v.lifetime_vouchees_jailed + 1;
  //     };
  //     i = i + 1;
  //   }
  // }

  ///////// GETTERS //////////

  public fun get_failure_to_join(addr: address): u64 acquires Jail {
    if (exists<Jail>(addr)) {
      return borrow_global<Jail>(addr).consecutive_failure_to_rejoin
    };
    0
  }

  public fun get_vouchee_jail(addr: address): u64 acquires Jail {
    if (exists<Jail>(addr)) {
      return borrow_global<Jail>(addr).lifetime_vouchees_jailed
    };
    0
  }


  /////// TEST HELPERS ////////
  public fun exists_jail(addr: address): bool {
    exists<Jail>(addr)
  }

}