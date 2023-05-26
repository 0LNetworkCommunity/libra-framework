
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_meta {
  use aptos_framework::reconfiguration;
  use aptos_framework::stake;
  
  use ol_framework::mock;

  // can we trigger a reconfiguration and get to a new epoch?
  // see also mock::trigger_epoch()  and meta_epoch() test

  #[test(vm = @vm_reserved)]
  fun test_reconfigure_custom() {

    mock::genesis();
    // NOTE: There was no genesis END event here. 
    // Which means we need to use reconfigure_for_test_custom

    let a = reconfiguration::get_current_epoch();

    // create a new epoch
    stake::end_epoch();

    reconfiguration::reconfigure_for_test_custom();

    let b = reconfiguration::get_current_epoch();


    assert!(a == 0, 10001);
    assert!(b == 1, 10002);
    
  }

  #[test(vm = @vm_reserved)]
  // can we trigger a reconfiguration and get to a new epoch?
  fun test_reconfigure_ol_setup() {
    // NOTE: genesis_n_vals, DOES trigger a genesis END event.
    mock::genesis_n_vals(4);




    let a = reconfiguration::get_current_epoch();

    // create a new epoch
    reconfiguration::reconfigure_for_test();
    // stake::end_epoch();

    let b = reconfiguration::get_current_epoch();

    assert!(a == 0, 10001);
    assert!(b == 1, 10002);
  }

  #[test(root = @ol_framework)]
  fun test_reconfigure_mock_trigger(root: signer) {
    mock::genesis();
    let a = reconfiguration::get_current_epoch();

    mock::trigger_epoch(&root);
    let b = reconfiguration::get_current_epoch();
    
    assert!(a == 0, 10001);
    assert!(b == 1, 10002);
    
  }
}
