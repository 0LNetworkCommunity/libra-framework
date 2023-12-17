
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_meta {
  use diem_framework::reconfiguration;
  use ol_framework::mock;
    use diem_std::debug::print;

  #[test(root = @ol_framework)]
  fun mock_epochs_advance(root: &signer) {
    // Scenario: Testing that if an action expires voting cannot be done.

    let _vals = mock::genesis_n_vals(root, 2);
    // we are at epoch 0
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 0, 7357001);
    mock::trigger_epoch(root); // epoch 1
    let epoch = reconfiguration::get_current_epoch();
    print(&epoch);

    assert!(epoch == 1, 7357002);

    mock::trigger_epoch(root); // epoch 2
    mock::trigger_epoch(root); // epoch 3 -- voting ended here
    mock::trigger_epoch(root); // epoch 4 -- now expired

    let epoch = reconfiguration::get_current_epoch();
    print(&epoch);
    assert!(epoch == 4, 7357003);

  }


  #[test(root = @ol_framework)]
  // can we trigger a reconfiguration and get to a new epoch?
  fun test_reconfigure_ol_setup(root: signer) {
    // NOTE: genesis_n_vals, DOES trigger a genesis END event.
    mock::genesis_n_vals(&root, 4);

    let a = reconfiguration::get_current_epoch();

    // create a new epoch
    reconfiguration::reconfigure_for_test();

    let b = reconfiguration::get_current_epoch();

    assert!(a == 0, 10001);
    assert!(b == 1, 10002);
  }

  // #[test(root = @ol_framework)]
  // fun test_reconfigure_mock_trigger(root: signer) {
  //   mock::ol_test_genesis(&root);
  //   // mock::ol_initialize_coin(&root);
  //   let a = reconfiguration::get_current_epoch();

  //   mock::trigger_epoch(&root);
  //   let b = reconfiguration::get_current_epoch();

  //   assert!(a == 0, 10001);
  //   assert!(b == 1, 10002);

  // }
}
