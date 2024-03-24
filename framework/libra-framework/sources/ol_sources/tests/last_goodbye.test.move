#[test_only]
module ol_framework::test_last_goodbye {
  use diem_framework::reconfiguration;
  use ol_framework::mock;
  use ol_framework::last_goodbye;
  use diem_framework::stake;
  use std::vector;

  // use diem_std::debug::print;

  #[test(framework = @ol_framework, vm = @0x0, alice = @0x1000a)]
  fun last_goodbye_meta(framework: &signer, vm: &signer, alice: &signer) {
    // Scenario: Testing that if an action expires voting cannot be done.

    let _vals = mock::genesis_n_vals(framework, 10);
    // we are at epoch 0
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 0, 7357001);

    last_goodbye::danger_test_last_goodby(vm, alice);
    last_goodbye::danger_framework_gc(vm);

    last_goodbye::danger_user_gc(vm, alice);

    mock::trigger_epoch(framework); // epoch 1
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 1, 7357002);

    let vals = stake::get_current_validators();
    assert!(!vector::contains(&vals, &@0x1000a), 7357003);

  }
}
