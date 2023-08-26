
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_meta {
  use diem_framework::reconfiguration;
  use diem_framework::stake;

  use ol_framework::mock;

  // can we trigger a reconfiguration and get to a new epoch?
  // see also mock::trigger_epoch()  and meta_epoch() test

  #[test(root = @ol_framework)]
  fun test_reconfigure_custom(root: signer) {

    mock::ol_test_genesis(&root);
    // NOTE: There was no genesis END event here.
    // Which means we need to use test_helper_increment_epoch_dont_reconfigure

    let a = reconfiguration::get_current_epoch();

    // create a new epoch
    stake::end_epoch();

    reconfiguration::test_helper_increment_epoch_dont_reconfigure();

    let b = reconfiguration::get_current_epoch();


    assert!(a == 0, 10001);
    assert!(b == 1, 10002);

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
  //   mock::ol_initialize_coin(&root);
  //   let a = reconfiguration::get_current_epoch();

  //   mock::trigger_epoch(&root);
  //   let b = reconfiguration::get_current_epoch();

  //   assert!(a == 0, 10001);
  //   assert!(b == 1, 10002);

  // }
}
