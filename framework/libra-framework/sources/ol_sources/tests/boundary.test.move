// e2e tests for epoch boundary
#[test_only]
module ol_framework::test_boundary {
  use std::vector;
  use std::features;
  use diem_std::bls12381;
  use ol_framework::mock;
  use ol_framework::proof_of_fee;
  use ol_framework::jail;
  use ol_framework::slow_wallet;
  use ol_framework::vouch;
  use ol_framework::testnet;
  use ol_framework::validator_universe;
  use ol_framework::epoch_boundary;
  use ol_framework::block;  
  use ol_framework::ol_account;
  use diem_framework::stake;
  use diem_framework::reconfiguration;
  use diem_framework::timestamp;
  use diem_framework::diem_governance;
  
  // use diem_std::debug::print;

  const Alice: address = @0x1000a;
  const Bob: address = @0x1000b;
  const Carol: address = @0x1000c;
  const Dave: address = @0x1000d;
  const Eve: address = @0x1000e;
  const Frank: address = @0x1000f;


  fun common_test_setup(root: &signer): vector<address> {
    let set = mock::genesis_n_vals(root, 10);
    mock::ol_initialize_coin_and_fund_vals(root, 500000, true);
    mock::mock_all_vals_good_performance(root);
    mock::pof_default();
    slow_wallet::slow_wallet_epoch_drip(root, 500000);

    // NOTE: for e2e epoch tests, we need to go into an operating epoch (not 0 or 1). Advance to epoch #2
    reconfiguration::test_helper_increment_epoch_dont_reconfigure(1);
    reconfiguration::test_helper_increment_epoch_dont_reconfigure(1);

    set
  }

  // We need to test e2e of the epoch boundary
  #[test(root = @ol_framework)]
  fun e2e_boundary_happy(root: signer) {
    let _vals = common_test_setup(&root);

    mock::trigger_epoch(&root);

    let _vals_post = stake::get_current_validators();

    assert!(epoch_boundary::get_reconfig_success(), 7357001);

    // all validators were compliant, should be +1 of the 10 vals
    assert!(epoch_boundary::get_seats_offered() == 11, 7357002);

    // all vals had winning bids, but it was less than the seats on offer
    assert!(vector::length(&epoch_boundary::get_auction_winners()) == 10, 7357003);
    // all of the auction winners became the validators ulitmately
    assert!(vector::length(&epoch_boundary::get_actual_vals()) == 10, 7357004);
  }


  #[test(root = @ol_framework, alice = @0x1000a,  marlon_rando = @0x12345)]
  fun e2e_add_validator_happy(root: signer, alice: signer, marlon_rando: signer) {
    let _vals = common_test_setup(&root);

    // generate credentials for validator registration
    // ol_account::create_account(&root, @0x12345);
    ol_account::transfer(&alice, @0x12345, 200000);
    let (_sk, pk, pop) = stake::generate_identity();
    let pk_bytes = bls12381::public_key_to_bytes(&pk);
    let pop_bytes = bls12381::proof_of_possession_to_bytes(&pop);
    validator_universe::register_validator(&marlon_rando, pk_bytes, pop_bytes, b"123", b"abc");

    // MARLON PAYS THE BID
    let vals = validator_universe::get_eligible_validators();
    assert!(vector::length(&vals) == 11, 7357000);
    mock::mock_bids(&vals);

    // MARLON HAS MANY FRIENDS
    vouch::test_set_buddies(@0x12345, vals);

    let (errs, _pass) = proof_of_fee::audit_qualification(@0x12345);
    assert!(vector::length(&errs) == 0, 7357001);

    mock::trigger_epoch(&root);

    assert!(epoch_boundary::get_reconfig_success(), 7357002);

    // all validators were compliant, should be +1 of the 10 vals
    assert!(epoch_boundary::get_seats_offered() == 11, 7357003);

    // NOTE: now MARLON is INCLUDED in this, and we filled all the seats on offer.
    // all vals had winning bids, but it was less than the seats on offer
    assert!(vector::length(&epoch_boundary::get_auction_winners()) == 11, 7357003);
    // all of the auction winners became the validators ulitmately
    assert!(vector::length(&epoch_boundary::get_actual_vals()) == 11, 7357004);

    // another epoch and everyone is compliant as well
    mock::mock_all_vals_good_performance(&root);
    mock::trigger_epoch(&root);
    assert!(epoch_boundary::get_seats_offered() == 12, 7357005);
    assert!(vector::length(&epoch_boundary::get_actual_vals()) == 11, 7357006);
  }

  #[test(root = @ol_framework, alice = @0x1000a,  marlon_rando = @0x12345)]
  fun e2e_add_validator_sad_vouches(root: signer, alice: signer, marlon_rando: signer) {
    let _vals = common_test_setup(&root);
    // this test requires prod settings, since we don't check vouches on testing
    testnet::unset(&root);
    // generate credentials for validator registration
    // ol_account::create_account(&root, @0x12345);
    ol_account::transfer(&alice, @0x12345, 200000);
    let (_sk, pk, pop) = stake::generate_identity();
    let pk_bytes = bls12381::public_key_to_bytes(&pk);
    let pop_bytes = bls12381::proof_of_possession_to_bytes(&pop);
    validator_universe::register_validator(&marlon_rando, pk_bytes, pop_bytes, b"123", b"abc");

    let vals = validator_universe::get_eligible_validators();
    assert!(vector::length(&vals) == 11, 7357000);
    mock::mock_bids(&vals);

    mock::trigger_epoch(&root);

    assert!(epoch_boundary::get_reconfig_success(), 7357002);

    // all validators were compliant, should be +1 of the 10 vals
    assert!(epoch_boundary::get_seats_offered() == 11, 7357003);

    // NOTE: now MARLON is INCLUDED in this, and we filled all the seats on offer.
    // all vals had winning bids, but it was less than the seats on offer
    assert!(vector::length(&epoch_boundary::get_auction_winners()) == 10, 7357003);
    // all of the auction winners became the validators ulitmately
    assert!(vector::length(&epoch_boundary::get_actual_vals()) == 10, 7357004);
  }

  #[test(root = @ol_framework)]
  fun e2e_boundary_excludes_jail(root: signer) {
    let vals = common_test_setup(&root);
    let alice_addr = *vector::borrow(&vals, 0);
    jail::jail(&root, alice_addr);
    assert!(jail::is_jailed(alice_addr), 7357000);
    mock::trigger_epoch(&root);

    let qualified_bidders = epoch_boundary::get_qualified_bidders();
    assert!(vector::length(&qualified_bidders) == (vector::length(&vals) - 1), 7357003);

    // all vals had winning bids, but it was less than the seats on offer
    assert!(vector::length(&epoch_boundary::get_auction_winners()) == vector::length(&qualified_bidders) , 7357003);
    assert!(epoch_boundary::get_reconfig_success(), 7357001);
  }

  #[test(root = @ol_framework, marlon = @0x12345)]
  fun epoch_any_address_trigger(root: &signer, marlon: &signer) {
    common_test_setup(root);
    // testing mainnet, so change the chainid
    testnet::unset(root);
    // test setup advances to epoch #2
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 2, 7357001);
    epoch_boundary::test_set_boundary_ready(root, epoch);

    // test the APIs as root
    timestamp::fast_forward_seconds(1); // needed for reconfig
    epoch_boundary::test_trigger(root);
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 3, 7357002);

    // scenario: marlon has no privileges
    // but he can still trigger an epoch if it is enabled

    epoch_boundary::test_set_boundary_ready(root, epoch);
    timestamp::fast_forward_seconds(1); // needed for reconfig
    diem_governance::trigger_epoch(marlon);
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 4, 7357003);
  }


  #[test(root = @ol_framework, marlon = @0x12345)]
  #[expected_failure(abort_code = 2, location = 0x1::epoch_boundary)]
  fun epoch_any_address_aborts(root: &signer, marlon: &signer) {
    common_test_setup(root);
    // testing mainnet, so change the chainid
    testnet::unset(root);
    // test setup advances to epoch #2
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 2, 7357001);
    epoch_boundary::test_set_boundary_ready(root, epoch);

    // test the APIs as root
    timestamp::fast_forward_seconds(1); // needed for reconfig
    epoch_boundary::test_trigger(root);
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 3, 7357002);

    // scenario: marlon has no privileges
    // if the epoch Boundary Bit is not set, then abort
    // Note: the Epoch will not change, but we can't assert
    // in the test

    ////// NOTE: not calling this`test_set_boundary_ready`
    // epoch_boundary::test_set_boundary_ready(root, epoch);
    //////

    timestamp::fast_forward_seconds(1);
    diem_governance::trigger_epoch(marlon);
    // aborts
  }


  #[test(root = @ol_framework, alice = @0x1000a, marlon = @0x12345)]
  fun test_epoch_trigger_disabled(root: &signer) {
    common_test_setup(root);
    // testing mainnet, so change the chainid
    testnet::unset(root);

    // test setup advances to epoch #2
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 2, 7357001);
    epoch_boundary::test_set_boundary_ready(root, epoch);
    

    // case: trigger not set and flipped
    timestamp::fast_forward_seconds(1); // needed for reconfig
    block::test_maybe_advance_epoch(root, 602000001, 602000000);

    // test epoch advances
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 3, 7357002);

  }

  #[test(root = @ol_framework, alice = @0x1000a, marlon = @0x12345)]
  fun test_epoch_trigger_enabled(root: &signer) {
    common_test_setup(root);
    // testing mainnet, so change the chainid
    testnet::unset(root);
    // test setup advances to epoch #2
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 2, 7357001);
    epoch_boundary::test_set_boundary_ready(root, epoch);

    // case: epoch trigger set
    features::change_feature_flags(root, vector[features::get_epoch_trigger()], vector[]);
    timestamp::fast_forward_seconds(1); // needed for reconfig
    block::test_maybe_advance_epoch(root, 603000001, 602000000);

    // test epoch did not advance and needs to be triggered
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 2, 7357002);

    // test epoch can be triggered and advances
    epoch_boundary::test_trigger(root);
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 3, 7357002);
  }



}
