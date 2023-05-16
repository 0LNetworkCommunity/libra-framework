#[test_only]
module ol_framework::test_pof {
  use ol_framework::mock;
  use ol_framework::account;
  use ol_framework::proof_of_fee;
  use ol_framework::reconfiguration;
  use ol_framework::jail;
  use ol_framework::slow_wallet;
  use ol_framework::vouch;
  use ol_framework::testnet;
  use aptos_framework::stake;
  use std::vector;

  // use aptos_std::debug::print;

  #[test_only]
  fun mock_good_bid(vm: &signer, alice: &address) {
    let a_sig = account::create_signer_for_test(*alice);
    proof_of_fee::set_bid(&a_sig, 1, 10000);
    let (bid, expires) = proof_of_fee::current_bid(*alice);
    assert!(bid == 1, 1001);
    assert!(expires == 10000, 1002);
    
    slow_wallet::slow_wallet_epoch_drip(vm, 500000);
    let coin = slow_wallet::unlocked_amount(*alice);
    let (r, _, _) = proof_of_fee::get_consensus_reward();
    let bid_cost = (bid * r) / 1000;
    assert!(coin > bid_cost, 1005);
  }

  #[test]
  fun pof_set_retract () {
    // genesis();
    
    let set = mock::genesis_n_vals(4);
    let alice = vector::borrow(&set, 0);
    stake::is_valid(*alice);

    let a_sig = account::create_signer_for_test(*alice);
    proof_of_fee::init(&a_sig);

    let (bid, expires) = proof_of_fee::current_bid(*alice);
    assert!(bid == 0, 1001);
    assert!(expires == 0, 1002);


    proof_of_fee::set_bid(&a_sig, 100, 0);
    let (bid, expires) = proof_of_fee::current_bid(*alice);
    assert!(bid == 100, 1003);
    assert!(expires == 0, 1004);


    // now retract
    proof_of_fee::retract_bid(&a_sig);
    let (bid, expires) = proof_of_fee::current_bid(*alice);
    let (is_rectracted, epoch) = proof_of_fee::is_already_retracted(*alice);
    assert!(is_rectracted, 1007);

    let this_epoch = reconfiguration::get_current_epoch();

    assert!(epoch == this_epoch, 1008);
    assert!(bid == 0, 1009);
    assert!(expires == 0, 10010);
  }

  #[test(vm = @ol_framework)]
  fun audit_happy (vm: signer) {
    let set = mock::genesis_n_vals(4);
    let alice = vector::borrow(&set, 0);

    assert!(stake::is_valid(*alice), 1000);
    assert!(!jail::is_jailed(*alice), 1001);

    mock_good_bid(&vm, alice);


    // // should NOTE pass audit.
    assert!(proof_of_fee::audit_qualification(alice), 1006);
  }

  #[test(vm = @ol_framework)]
  fun audit_vouch (vm: signer) {
    let set = mock::genesis_n_vals(4);
    let alice = vector::borrow(&set, 0);

    assert!(!jail::is_jailed(*alice), 1001);
    
    mock_good_bid(&vm, alice);

    testnet::unset(&vm);

    assert!(vouch::unrelated_buddies_above_thresh(*alice), 1006);
    assert!(proof_of_fee::audit_qualification(alice), 1008);

    vouch::test_set_buddies(*alice, vector::empty());
    assert!(!vouch::unrelated_buddies_above_thresh(*alice), 1006);
    assert!(!proof_of_fee::audit_qualification(alice), 1008);
  }

  #[test(vm = @ol_framework)]
  fun audit_no_funds (vm: signer) {
    let set = mock::genesis_n_vals(4);
    let alice = vector::borrow(&set, 0);

    assert!(!jail::is_jailed(*alice), 1001);
    let a_sig = account::create_signer_for_test(*alice);

    proof_of_fee::set_bid(&a_sig, 100, 10000); // 10 pct
    let (bid, expires) = proof_of_fee::current_bid(*alice);
    assert!(bid == 100, 1001);
    assert!(expires == 10000, 1002);
    
    // NOT ENOUGH FUNDS WERE UNLOCKED
    slow_wallet::slow_wallet_epoch_drip(&vm, 500);
    let coin = slow_wallet::unlocked_amount(*alice);
    let (r, _, _) = proof_of_fee::get_consensus_reward();
    let bid_cost = (bid * r) / 1000;
    assert!(coin < bid_cost, 1005);

    // should NOTE pass audit.
    assert!(!proof_of_fee::audit_qualification(alice), 1006);
  }

  #[test(vm = @ol_framework)]
  fun audit_expired(vm: signer) {
    let set = mock::genesis_n_vals(4);
    let alice = vector::borrow(&set, 0);

    assert!(!jail::is_jailed(*alice), 1001);

    // initially set a good bid
    mock_good_bid(&vm, alice);


    // has a bid which IS expired
    // test runner is at epoch 1, they put expiry at 0.
    // TODO: Improve this test by doing more advanced epochs
    let a_sig = account::create_signer_for_test(*alice);

    // let this_epoch = reconfiguration::get_current_epoch();


    
    reconfiguration::reconfigure_for_test();
    stake::end_epoch();
    
    // let this_epoch = reconfiguration::get_current_epoch();


    proof_of_fee::set_bid(&a_sig, 1, 0); 
    let (bid, expires) = proof_of_fee::current_bid(*alice);
    assert!(bid == 1, 1006);
    assert!(expires == 0, 1007);
    // should NOT pass audit.
    assert!(!proof_of_fee::audit_qualification(alice), 1008);

    // fix it
    proof_of_fee::set_bid(&a_sig, 1, 1000); 
    assert!(proof_of_fee::audit_qualification(alice), 1009);
  }

  #[test(vm = @ol_framework)]
  fun audit_jail (vm: signer) {
    let set = mock::genesis_n_vals(4);
    let alice = vector::borrow(&set, 0);

    assert!(!jail::is_jailed(*alice), 1001);

    mock_good_bid(&vm, alice);

    // EVERYTHING ABOVE HERE IS PASSING.
    // is now jailed.
    jail::jail(&vm, *alice);
    assert!(jail::is_jailed(*alice), 1006);
    // won't pass audit
    assert!(!proof_of_fee::audit_qualification(&*alice), 1007);
    
  }

  #[test(vm = @ol_framework)]
  fun sorted_vals_happy(vm: signer) {
    let set = mock::genesis_n_vals(4);
    let len = vector::length(&set);
    let i = 0;
    while (i < len) {
      let addr = vector::borrow(&set, i);
      mock_good_bid(&vm, addr);
      i = i + 1;
    };
    // mock_good_bid(&vm, alice);
    let (val_universe, _their_bids, _their_expiry) = mock::pof_default();
    let sorted = proof_of_fee::get_sorted_vals(false);

    let len = vector::length(&sorted);
    assert!(len == vector::length(&val_universe), 1000);
    assert!(vector::length(&sorted) == vector::length(&val_universe), 1002);

    let sorted_two = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted_two) == vector::length(&val_universe), 1003);
  }


  #[test]
  fun sorted_vals_none_qualify() {
    let _set = mock::genesis_n_vals(4);
    let (val_universe, _their_bids, _their_expiry) = mock::pof_default();
    let sorted = proof_of_fee::get_sorted_vals(false);

    let len = vector::length(&sorted);
    assert!(len == vector::length(&val_universe), 1000);
    assert!(vector::length(&sorted) == vector::length(&val_universe), 1002);

    let sorted = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted) != vector::length(&val_universe), 1003);
    assert!(vector::length(&sorted) == 0, 1004);
  }

  #[test(vm = @ol_framework)]
  fun sorted_vals_jail(vm: signer) {
    let set = mock::genesis_n_vals(4);
    let len = vector::length(&set);
    let i = 0;
    while (i < len) {
      let addr = vector::borrow(&set, i);
      mock_good_bid(&vm, addr);
      i = i + 1;
    };

    // jail alice, the filtered sort should have one less.
    let alice = vector::borrow(&set, 0);
    jail::jail(&vm, *alice);

    let (val_universe, _their_bids, _their_expiry) = mock::pof_default();


    // let (val_universe, _their_bids, _their_expiry) = mock::pof_default();
    let sorted = proof_of_fee::get_sorted_vals(false);
    let len = vector::length(&sorted);
    assert!(len == vector::length(&val_universe), 1000);
    assert!(vector::length(&sorted) == vector::length(&val_universe), 1002);


    let sorted_two = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted_two) != vector::length(&val_universe), 1004);
    assert!(vector::length(&sorted_two) == vector::length(&val_universe) - 1, 1005);

  }



  #[test(vm = @ol_framework)]
  fun sorted_vals_expired_bid(vm: signer) {
    let set = mock::genesis_n_vals(4);
    let (val_universe, _their_bids, _their_expiry) = mock::pof_default();

    let len = vector::length(&set);
    let i = 0;
    while (i < len) {
      let addr = vector::borrow(&set, i);
      mock_good_bid(&vm, addr);
      i = i + 1;
    };

    // set an expired bid for alice
    let alice = vector::borrow(&set, 0);
    let alice_sig = account::create_signer_for_test(*alice);
    proof_of_fee::set_bid(&alice_sig, 55, 1);
  

    mock::trigger_epoch();
    mock::trigger_epoch();


    // let (val_universe, _their_bids, _their_expiry) = mock::pof_default();
    let sorted = proof_of_fee::get_sorted_vals(false);
    let len = vector::length(&sorted);
    assert!(len == vector::length(&val_universe), 1000);
    assert!(vector::length(&sorted) == vector::length(&val_universe), 1002);


    let sorted_two = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted_two) != vector::length(&val_universe), 1004);
    assert!(vector::length(&sorted_two) == vector::length(&val_universe) - 1, 1005);

  }

  // We can send the fill seats function a list of validators, and the list of performing validators, and it will return the winning bidders and the bid.
  #[test(vm = @ol_framework)]
  fun fill_seats_happy(vm: signer) {
    let set = mock::genesis_n_vals(5);
    let len = vector::length(&set);

    mock::pof_default();
    
    slow_wallet::slow_wallet_epoch_drip(&vm, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted) == vector::length(&set), 1003);

    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&vm, len, &sorted, &sorted);

    assert!(vector::contains(&seats, vector::borrow(&set, 0)), 1004);

    // filling the seat updated the computation of the consensu reward.
    let (reward, clear_price, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 1005);
    assert!(clear_price == 1, 1006);
    assert!(median_bid == 3, 1007);

  }
  
  // We fill all the seats, and run the thermostat
  // the thermostat is a noop since there is not enough historical data.
  #[test(vm = @ol_framework)]
  fun fill_seats_happy_and_noop_thermostat(vm: signer) {
    let set = mock::genesis_n_vals(5);
    mock::pof_default();
    
    slow_wallet::slow_wallet_epoch_drip(&vm, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted) == vector::length(&set), 1003);

    let len = vector::length(&set);
    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&vm, len, &sorted, &sorted);

    assert!(vector::contains(&seats, vector::borrow(&set, 0)), 1004);

    // filling the seat updated the computation of the consensu reward.
    let (reward, clear_price, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 1005);
    assert!(clear_price == 1, 1006);
    assert!(median_bid == 3, 1007);

    // we expect no change in the reward_thermostat because there haven't been 5 epochs or more of historical data.
    proof_of_fee::reward_thermostat(&vm);

    let (reward, win_bid, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 1008);
    assert!(win_bid == 1, 1009);
    assert!(median_bid == 3, 1010);
  }


  // Scenario: Eve does not bid. So we have fewer bidders than seats
  // Otherwise Eve is a performing and valid validator
  // For all lists we are using the validators in the ValidatorUniverse
  // the desired validator set is the same size as the universe.
  // All validators in the universe are properly configured
  // All validators performed perfectly in the previous epoch.
  // They have all placed bids, per TestFixtures::pof_default().

  #[test(vm = @ol_framework)]
  fun fill_seats_few_bidders(vm: signer) {
    let set = mock::genesis_n_vals(5);
    mock::pof_default();


    // Ok now EVE changes her mind. Will force the bid to expire.
    let a_sig = account::create_signer_for_test(*vector::borrow(&set, 4));
    proof_of_fee::set_bid(&a_sig, 0, 0);
    mock::trigger_epoch();
    
    slow_wallet::slow_wallet_epoch_drip(&vm, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted) != vector::length(&set), 1003);

    // let len = vector::length(&set);
    assert!(vector::length(&set) == 5, 1004);
    assert!(vector::length(&sorted) == 4, 1005);

    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&vm, vector::length(&set), &sorted, &sorted);

    // EVE is not in the seats
    assert!(!vector::contains(&seats, vector::borrow(&set, 4)), 1004);
    // Alice is
    assert!(vector::contains(&seats, vector::borrow(&set, 0)), 1005);

    // filling the seat updated the computation of the consensu reward.
    let (reward, clear_price, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 1006);
    assert!(clear_price == 1, 1007);
    assert!(median_bid == 2, 1008);

  }

}