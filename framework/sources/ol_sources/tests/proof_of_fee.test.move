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

  const Alice: address = @0x1000a;
  const Bob: address = @0x1000b;
  const Carol: address = @0x1000c;
  const Dave: address = @0x1000d;
  const Eve: address = @0x1000e;
  const Frank: address = @0x1000f;



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


  // Scenario: we are checking that bids expire as the epochs progress.  

  #[test(root = @ol_framework)]
  fun sorted_vals_expired_bid(root: signer) {
    let set = mock::genesis_n_vals(4);
    mock::ol_initialize_coin(&root);
    let (val_universe, _their_bids, _their_expiry) = mock::pof_default();

    let len = vector::length(&set);
    let i = 0;
    while (i < len) {
      let addr = vector::borrow(&set, i);
      mock_good_bid(&root, addr);
      i = i + 1;
    };

    // set an expired bid for alice
    let alice = vector::borrow(&set, 0);
    let alice_sig = account::create_signer_for_test(*alice);
    proof_of_fee::set_bid(&alice_sig, 55, 1);
  
    // advance the epoch 2x, so the previous bid is expired.
    mock::mock_all_vals_good_performance(&root);
    mock::trigger_epoch(&root);
    mock::mock_all_vals_good_performance(&root);
    mock::trigger_epoch(&root);

    // Get all vals but don't filter the ones that have passing bids
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

  #[test(root = @ol_framework)]
  fun fill_seats_few_bidders(root: signer) {
    let set = mock::genesis_n_vals(5);
    mock::pof_default();


    // Ok now EVE changes her mind. Will force the bid to expire.
    let a_sig = account::create_signer_for_test(*vector::borrow(&set, 4));
    proof_of_fee::set_bid(&a_sig, 0, 0);
    
    slow_wallet::slow_wallet_epoch_drip(&root, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);
    assert!(vector::length(&sorted) != vector::length(&set), 1003);

    // let len = vector::length(&set);
    assert!(vector::length(&set) == 5, 1004);
    assert!(vector::length(&sorted) == 4, 1005);

    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&root, vector::length(&set), &sorted, &sorted);

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

// Scenario: We have 5 validators but only 3 seats in the set.
// They have all placed bids, per TestFixtures::pof_default().
// The lowest bidders Alice and Bob, will be excluded.

// For all lists we are using the validators in the ValidatorUniverse
// the desired validator set is the same size as the universe.
// All validators in the universe are properly configured
// All validators performed perfectly in the previous epoch.

  #[test(vm = @ol_framework)]
  fun fill_seats_many_bidders(vm: signer) {
    let set = mock::genesis_n_vals(5);
    mock::pof_default();

    
    slow_wallet::slow_wallet_epoch_drip(&vm, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);



    let set_size = 3;
    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&vm, set_size, &sorted, &sorted);

    assert!(vector::length(&set) == 5, 1004);
    assert!(vector::length(&seats) == 3, 1005);

     // alice, bob, had low bids, and are out
     // see mock::pof_deault for fibonacci bids
    assert!(!vector::contains(&seats, vector::borrow(&set, 0)), 1001);
    assert!(!vector::contains(&seats, vector::borrow(&set, 1)), 1002);
    // carol, dave, and eve are in
    assert!(vector::contains(&seats, vector::borrow(&set, 2)), 1003);
    assert!(vector::contains(&seats, vector::borrow(&set, 3)), 1004);
    assert!(vector::contains(&seats, vector::borrow(&set, 4)), 1004);

    // filling the seat updated the computation of the consensu reward.
    // Median bids and clearing prices will be different than the happy path test.
    let (reward, clear_price, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 1004);
    assert!(clear_price == 3, 1005);
    assert!(median_bid == 5, 1006);
  }


  // Scenario: Here we have 6 validators and 6 seats, but only 4 come
  // from the previous epoch.
  // They have all placed bids, per TestFixtures::pof_default().

  // However Alice and Bob have not been in the last epoch's set.
  // So we consider them "unproven".
  // Alice and Bob happen to also be the lowest bidders. But we will
  // seat them, and their bids will count toward getting the clearing price.

  // In this scenario there will be sufficient seats. 
  // We will open up 2 seats (1/3 of 6 seats).
  // As such both Eve and Frank should be seated.

  // The clearing price will not change. The lowest is still alice
  // who is also seated.


  // For all lists we are using the validators in the ValidatorUniverse
  // the desired validator set is the same size as the universe.
  // All validators in the universe are properly configured
  // All validators performed perfectly in the previous epoch.

  #[test(root = @ol_framework)]
  fun fill_seats_unproven_happy(root: signer) {
    // we need 6 seats so that we can have 4 proven, and 2 unproven slots
    let set = mock::genesis_n_vals(6);
    mock::pof_default();

    slow_wallet::slow_wallet_epoch_drip(&root, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);

    let set_size = 6;

    // Alice and Bob were not in the previous round. They are not "proven"
    let proven_vals = vector::singleton(*vector::borrow(&set, 2));
    vector::push_back(&mut proven_vals, *vector::borrow(&set, 3));
    vector::push_back(&mut proven_vals, *vector::borrow(&set, 4));
    vector::push_back(&mut proven_vals, *vector::borrow(&set, 5));

    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&root, set_size, &sorted, &proven_vals);

    assert!(vector::length(&set) == 6, 1004);
    assert!(vector::length(&seats) == 6, 1005);

    // Alice and Bob must be in, even though they were not "proven".
    assert!(vector::contains(&seats, vector::borrow(&set, 0)), 1001);
    assert!(vector::contains(&seats, vector::borrow(&set, 1)), 1002);
    assert!(vector::contains(&seats, vector::borrow(&set, 2)), 1003);


    // filling the seat updated the computation of the consensu reward.
    // Median bids and clearing prices will be different than the happy path test.
    let (reward, clear_price, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 1004);
    // The clearing price is 1, Alice's lowest bid. Even though she was not "proven"
    assert!(clear_price == 1, 1005);
    assert!(median_bid == 3, 1006);
  }


  // Scenario: Here we have 6 validators and 4 seats, but only 3 come
  // from the previous epoch (proven).
  // This time Eve and Frank are "unproven nodes", they also happen
  // to have the highest bids.
  // They have all placed bids, per TestFixtures::pof_default().

  // At 4 seats, we only have space for 1 unproven node.
  // It should be Frank that gets seated.
  // Eve will not be seated at all, even though she has a higher bid
  // than the proven nodes.

  // The clearing price will not change. The lowest is still Alice
  // who is also seated.

  // For all lists we are using the validators in the ValidatorUniverse
  // the desired validator set is the same size as the universe.
  // All validators in the universe are properly configured
  // All validators performed perfectly in the previous epoch.

  #[test(root = @ol_framework)]
  fun fill_seats_unproven_sad(root: signer) {

    
    // we need 6 seats so that we can have 4 proven, and 2 unproven slots
    let set = mock::genesis_n_vals(6);
    mock::pof_default();

    slow_wallet::slow_wallet_epoch_drip(&root, 500000);

    let sorted = proof_of_fee::get_sorted_vals(true);

    let set_size = 4;

    // Exclude Eve and Frank were not in the previous round. They are not "proven", but they have the highest bids
    let proven_vals = vector::singleton(Alice);
    vector::push_back(&mut proven_vals, Bob);
    vector::push_back(&mut proven_vals, Carol);
    vector::push_back(&mut proven_vals, Dave);

    let (alice_bid, _) = proof_of_fee::current_bid(Alice);
    assert!(alice_bid == 1, 1001);
    let (frank_bid, _) = proof_of_fee::current_bid(Frank);
    assert!(alice_bid < frank_bid, 1002);

    let (seats, _p) = proof_of_fee::fill_seats_and_get_price(&root, set_size, &sorted, &proven_vals);

    assert!(vector::length(&set) == 6, 1003);
    assert!(vector::length(&seats) == 4, 1004);

    // Alice dropped out, even though was "proven", was the lowest bidder for the three slots for proven, but ALSO lower than the highest bidder of unproven nodes FRANK.
    assert!(!vector::contains(&seats, vector::borrow(&set, 0)), 1006);
    assert!(vector::contains(&seats, vector::borrow(&set, 1)), 1007);
    assert!(vector::contains(&seats, vector::borrow(&set, 2)), 1008);
    assert!(vector::contains(&seats, vector::borrow(&set, 3)), 1009);
    // Eve does not get in. There was only one slot for unproven nodes, 
    // and her bid is lower than frank.
    assert!(!vector::contains(&seats, vector::borrow(&set, 4)), 10010);
    assert!(vector::contains(&seats, vector::borrow(&set, 5)), 10011);


    // filling the seat updated the computation of the consensu reward.
    // Median bids and clearing prices will be different than the happy path test.
    let (reward, clear_price, median_bid) = proof_of_fee::get_consensus_reward();
    assert!(reward == 1000000, 10012);
    assert!(clear_price == 2, 10013);
    assert!(median_bid == 3, 10014);
  }

}