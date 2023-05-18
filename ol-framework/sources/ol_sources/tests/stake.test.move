#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_stake {
  use ol_framework::mock;
  use std::vector;
  use ol_framework::stake;
  use ol_framework::testnet;


  // Scenario: can take 6 already initialized validators, from a previous set
  // and reduce the set to 3 of those validators.
  #[test]
  fun bulk_update_validators() {
    let set = mock::genesis_n_vals(6);
    let alice = vector::borrow(&set, 0);
    assert!(stake::is_valid(*alice), 1000);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 6, 1001);

    let all_vals = mock::personas();

    let new_list = vector::empty<address>();
    let i = 0;
    while (i < 3) {
      vector::push_back(&mut new_list, *vector::borrow(&all_vals, i));
      i = i + 1;
    };

    let (cfg_list, _weight) = stake::test_make_val_cfg(&new_list);

    
    stake::test_set_next_vals(cfg_list);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 3, 1001);

  }

  // Scenario: in production mode, we can't have fewer than 4 validators.
  // if we try to pass less, no change will happen (does not abort)
  #[test(root = @ol_framework)]
  fun minimum_four_failover(root: &signer) {

    let set = mock::genesis_n_vals(6);
    testnet::unset(root); // set to production mode

    let alice = vector::borrow(&set, 0);
    assert!(stake::is_valid(*alice), 1000);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 6, 1001);

    let all_vals = mock::personas();

    let new_list = vector::empty<address>();
    let i = 0;
    while (i < 3) {
      vector::push_back(&mut new_list, *vector::borrow(&all_vals, i));
      i = i + 1;
    };

    let (cfg_list, _weight) = stake::test_make_val_cfg(&new_list);

    
    stake::test_set_next_vals(cfg_list);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) != 3, 1001);

  }
}