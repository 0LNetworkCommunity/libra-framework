
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_reconfiguration {
  use std::vector;
  use aptos_framework::reconfiguration;
  use aptos_framework::stake;
  use ol_framework::mock;
  use ol_framework::musical_chairs;
  use aptos_std::debug::print;


  #[test(root = @ol_framework)]
  fun reconfig_happy_case(root: signer) {
      let vals = mock::genesis_n_vals(5);
      assert!(vector::length(&vals) == 5, 7357001);
      let vals = stake::get_current_validators();
      assert!(vector::length(&vals) == 5, 7357002);


      // all vals compliant
      mock::mock_all_vals_good_performance(&root);

      let (good, bad, _ratio) = musical_chairs::test_eval_compliance(&root, vals);
      // print(&good);
      assert!(vector::length(&good) == 5, 7357003);
      assert!(vector::length(&bad) == 0, 7357004);


    mock::trigger_epoch(&root);
    // let b = reconfiguration::get_current_epoch();
    

    let vals = stake::get_current_validators();
    print(&vals);

    
    // assert!(a == 0, 10001);
    // assert!(b == 1, 10002);
    
  }

  #[test(root = @ol_framework)]
  fun drop_non_performing(root: signer) {
    // mock::genesis();
    let vals = mock::genesis_n_vals(5);
    mock::mock_all_vals_good_performance(&root);

    let a = reconfiguration::get_current_epoch();

    // make alice non performant
    mock::mock_case_4(&root, *vector::borrow(&vals, 0));


    mock::trigger_epoch(&root);
    let b = reconfiguration::get_current_epoch();
    

    let vals = stake::get_current_validators();
    print(&vals);

    
    assert!(a == 0, 10001);
    assert!(b == 1, 10002);
    
  }
}
