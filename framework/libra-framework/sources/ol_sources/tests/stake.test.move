#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_stake {
  use ol_framework::mock;
  use std::vector;
  use ol_framework::stake;
  use ol_framework::testnet;
  use ol_framework::cases;

  // use aptos_std::debug::print;

  // Scenario: can take 6 already initialized validators, from a previous set
  // and reduce the set to 3 of those validators.
  #[test(root = @ol_framework)]
  fun bulk_update_validators(root: signer) {
    let set = mock::genesis_n_vals(&root, 6);
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

    
    stake::test_set_next_vals(&root, cfg_list);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 3, 1001);

  }

  // Scenario: in production mode, we can't have fewer than 4 validators.
  // when that happens, the first failover is to get the validators with the most successful proposals.
  // we start with 10 validators.
  // we assume only 3 are performant. That's under the threshold of 4.
  // we expect the failover to get 10/2 = 5 validators, sorted by performance.
  #[test(root = @ol_framework)]
  fun minimum_four_failover(root: signer) {

    let set = mock::genesis_n_vals(&root, 10);
    testnet::unset(&root); // set to production mode

    let alice = vector::borrow(&set, 0);
    assert!(stake::is_valid(*alice), 1000);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 10, 1001);

    let all_vals = mock::personas();

    let new_list = vector::empty<address>();
    let i = 0;
    while (i < 4) {
      vector::push_back(&mut new_list, *vector::borrow(&all_vals, i));
      i = i + 1;
    };

    let cfg_list = stake::check_failover_rules(new_list);
    assert!(vector::length(&cfg_list) == 5, 1003);

  }


  // Scenario: in production mode, we can't have fewer than 4 validators.
  // when that happens, the first failover is to get the validators with the most successful proposals.
  #[test(root = @ol_framework)]
  fun sorted_vals_by_props(root: signer) {

    let set = mock::genesis_n_vals(&root, 8);
    testnet::unset(&root); // set to production mode

    let alice = vector::borrow(&set, 0);
    assert!(stake::is_valid(*alice), 1000);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 8, 1001);

    let i = 0;
    while (i < vector::length(&set)) {
      let addr = vector::borrow(&set, i);
      stake::mock_performance(&root, *addr, i, 0); // increasing performance for each
      i = i + 1;
    };

    let cfg_list = stake::get_sorted_vals_by_props(3);
    
    assert!(vector::length(&cfg_list) == 3, 1002);
    assert!(vector::contains(&cfg_list, &@0x10011), 1003); // take the last persona
    assert!(!vector::contains(&cfg_list, &@0x1000a), 1004); // alice should not be here.
  }


  // Scenario: in production mode, we can't have fewer than 4 validators.
  // if we try to pass less, no change will happen (does not abort)
  #[test(root = @ol_framework)]
  fun jail_list(root: signer) {

    mock::genesis_n_vals(&root, 6);
    mock::mock_all_vals_good_performance(&root);
    // all validators bid
    mock::pof_default();

    // now make Eve not compliant
    let eve = @0x1000e;
    mock::mock_case_4(&root, eve);
    assert!(cases::get_case(eve) == 4, 735701);

    let v = cases::get_jailed_set();
    assert!(vector::contains(&v, &eve), 735702);

  }


}