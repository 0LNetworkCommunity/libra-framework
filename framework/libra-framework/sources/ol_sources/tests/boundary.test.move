// e2e tests for epoch boundary
#[test_only]
module ol_framework::test_boundary {
  use std::vector;
  use diem_std::bls12381;
  use ol_framework::mock;

  // use ol_framework::account;
  // use ol_framework::proof_of_fee;
  // use ol_framework::reconfiguration;
  // use ol_framework::jail;
  use ol_framework::slow_wallet;
  // use ol_framework::vouch;
  // use ol_framework::testnet;
  use ol_framework::validator_universe;
  use diem_framework::stake;
  use ol_framework::epoch_boundary;
  use ol_framework::ol_account;



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


  #[test(root = @ol_framework, marlon_rando = @0x12345)]
  fun e2e_add_validator_happy(root: signer, marlon_rando: signer) {
    let _vals = common_test_setup(&root);

    // generate credentials for validator registration
    ol_account::create_account(&root, @0x12345);
    let (_sk, pk, pop) = stake::generate_identity();
    let pk_bytes = bls12381::public_key_to_bytes(&pk);
    let pop_bytes = bls12381::proof_of_possession_to_bytes(&pop);
    validator_universe::register_validator(&marlon_rando, pk_bytes, pop_bytes, b"123", b"abc");

    let vals = validator_universe::get_eligible_validators();
    assert!(vector::length(&vals) == 11, 7357000);
    mock::mock_bids(&vals);
    mock::trigger_epoch(&root);

    // let vals_post = stake::get_current_validators();
    // print(&)

    assert!(epoch_boundary::get_reconfig_success(), 7357001);

    // all validators were compliant, should be +1 of the 10 vals
    assert!(epoch_boundary::get_seats_offered() == 11, 7357002);

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
}