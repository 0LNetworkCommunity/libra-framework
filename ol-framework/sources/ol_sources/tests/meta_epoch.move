
module ol_framework::test_meta {

  #[test_only]
  use aptos_std::debug::print;
  #[test_only]
  use aptos_framework::reconfiguration;
  #[test_only]
  use ol_framework::mock;
  #[test_only]
  use aptos_framework::stake;

  
  #[test]
  /// test we can trigger an epoch reconfiguration.
  public fun meta_epoch() {
    mock::genesis();
    print(&1001);
    let epoch = reconfiguration::current_epoch();
    print(&epoch);
    // reconfiguration::reconfigure_for_test();
    stake::end_epoch();
    reconfiguration::reconfigure_for_test_custom();
    print(&1002);
    let new_epoch = reconfiguration::current_epoch();
    print(&new_epoch);
    assert!(new_epoch > epoch, 7357001);

  }
}