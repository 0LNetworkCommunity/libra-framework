#[test_only]
module ol_framework::test_vouch {
  use std::vector;
  use ol_framework::vouch;
  use ol_framework::mock;

  // Happy Day scenarios

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun vouch_for_unrelated(root: &signer, alice: &signer, carol: &signer) {
    // create vals without vouches
    mock::create_vals(root, 3, false);
    vouch::set_vouch_price(root, 0);

    // check that no vouches exist
    vector::for_each(vector[@0x1000a, @0x1000b, @0x1000c], |addr| {
      let (given_vouches, given_epochs) = vouch::get_given_vouches(addr);
      assert!(given_vouches == vector::empty(), 73570001);
      assert!(given_epochs == vector::empty(), 73570002);
      let (received_vouches, received_epochs) = vouch::get_received_vouches(addr);
      assert!(received_vouches == vector::empty(), 73570003);
      assert!(received_epochs == vector::empty(), 73570004);
    });

    // alice vouches for bob
    vouch::vouch_for(alice, @0x1000b);

    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b], 73570005);
    assert!(given_epochs == vector[0], 73570006);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a], 73570007);
    assert!(received_epochs == vector[0], 73570008);

    // fast forward to epoch 1
    mock::trigger_epoch(root);

    // carol vouches for bob
    vouch::vouch_for(carol, @0x1000b);

    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b], 73570005);
    assert!(given_epochs == vector[0], 73570006);

    // check carol
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000c);
    assert!(given_vouches == vector[@0x1000b], 73570009);
    assert!(given_epochs == vector[1], 73570010);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a, @0x1000c], 73570011);
    assert!(received_epochs == vector[0, 1], 73570012);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun revoke_vouch(root: &signer, alice: &signer, carol: &signer) {
    // create vals without vouches
    mock::create_vals(root, 3, false);
    vouch::set_vouch_price(root, 0);

    // alice and carol vouches for bob
    vouch::vouch_for(alice, @0x1000b);
    vouch::vouch_for(carol, @0x1000b);

    // alice revokes vouch for bob
    vouch::revoke(alice, @0x1000b);

    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector::empty(), 73570013);
    assert!(given_epochs == vector::empty(), 73570014);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000c], 73570015);
    assert!(received_epochs == vector[0], 73570016);
  }

  // Sad Day scenarios

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x10001, location = ol_framework::vouch)]
  fun vouch_for_self(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::create_vals(root, 1, false);
    vouch::set_vouch_price(root, 0);

    // alice vouches for herself
    vouch::vouch_for(alice, @0x1000a);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x10001, location = ol_framework::vouch)]
  fun revoke_self_vouch(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::create_vals(root, 1, false);

    // alice try to revoke herself
    vouch::revoke(alice, @0x1000a);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x10005, location = ol_framework::vouch)]
  fun revoke_not_vouched(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::create_vals(root, 2, false);

    // alice vouches for bob
    vouch::revoke(alice, @0x1000b);
  }

}
