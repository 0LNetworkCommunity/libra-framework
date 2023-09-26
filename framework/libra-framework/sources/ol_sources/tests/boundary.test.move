// e2e tests for epoch boundary
#[test_only]
module ol_framework::test_boundary {

  use ol_framework::mock;
  // use ol_framework::account;
  // use ol_framework::proof_of_fee;
  // use ol_framework::reconfiguration;
  // use ol_framework::jail;
  use ol_framework::slow_wallet;
  // use ol_framework::vouch;
  // use ol_framework::testnet;
  // use ol_framework::globals;
  use diem_framework::stake;
  use ol_framework::epoch_boundary;
  use std::vector;

  use diem_std::debug::print;

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
    let vals = common_test_setup(&root);
    print(&vals);

    mock::trigger_epoch(&root);

    let vals_post = stake::get_current_validators();

    print(&vals_post);
    assert!(epoch_boundary::get_reconfig_success(), 7357001);

    // all validators were compliant, should be +1 of the 10 vals
    assert!(epoch_boundary::get_seats_offered() == 11, 7357002);

    // all vals had winning bids, but it was less than the seats on offer
    assert!(vector::length(&epoch_boundary::get_auction_winners()) == 10, 7357003);
    // all of the auction winners became the validators ulitmately
    assert!(vector::length(&epoch_boundary::get_actual_vals()) == 10, 7357004);
  }
}