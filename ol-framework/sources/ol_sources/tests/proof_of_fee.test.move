#[test_only]
module ol_framework::test_pof {
  use ol_framework::mock;
  use ol_framework::account;
  use ol_framework::proof_of_fee;
  use ol_framework::reconfiguration;
  use aptos_framework::stake;
  use std::vector;

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
}