#[test_only]
module ol_framework::test_last_goodbye {
  use diem_framework::reconfiguration;
  use ol_framework::mock;
  use ol_framework::last_goodbye;
  use diem_framework::stake;
  use std::vector;

  // use diem_std::debug::print;

  #[test(framework = @ol_framework, vm = @0x0, _alice = @0x1000a, bob = @0x1000b)]
  fun last_goodbye_meta(framework: &signer, vm: &signer, _alice: &signer, bob: &signer) {
    // Scenario: Testing that if an action expires voting cannot be done.

    let _vals = mock::genesis_n_vals(framework, 1);
    // we are at epoch 0
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 0, 7357001);

    last_goodbye::danger_test_last_goodby(vm, bob);
    last_goodbye::danger_framework_gc(vm);

    last_goodbye::danger_user_gc(vm, bob);

    mock::trigger_epoch(framework); // epoch 1
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 1, 7357002);

    let vals = stake::get_current_validators();
    assert!(!vector::contains(&vals, &@0x1000b), 7357003);

  }


  // when we drop accounts through hard fork we should have no boundary
  // account processing issues.
  #[test(vm = @0x0, framework = @ 0x1, alice = @0x1111a)]
  fun e2e_boundary_happy_drop_accounts(vm: signer, framework: signer, alice: signer) {
    use ol_framework::last_goodbye;

    let _vals = common_test_setup(&framework);

    last_goodbye::danger_test_last_goodby(&vm, &alice);

    mock::trigger_epoch(&framework);

  }
}
