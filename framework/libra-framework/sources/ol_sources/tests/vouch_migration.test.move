/*

  This test is to ensure that the migration of vouches
  from legacy to new validators is working as expected.

  1. Legacy data:
    - alice vouches for bob
    - carol vouches for bob
    - alice vouches for carol
  2. New data:
    - dave vouches for eve
    - eve vouches for frank

  trigger epoch to execute migration
  - check migrated vals vouches

 */

#[test_only]
module ol_framework::test_vouch_migration {
  use std::vector;
  use ol_framework::vouch;
  use ol_framework::mock;
  use ol_framework::migrations;

  // use diem_std::debug::print;

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e, frank = @0x1000f)]
  fun migrate_given_and_received_vouches(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer) {
    vouch::set_vouch_price(root, 0);

    // init validators with legacy struct MyVouches
    vouch::legacy_init(alice);
    vouch::legacy_init(bob);
    vouch::legacy_init(carol);

    // create validators
    mock::create_vals(root, 6, false);

    // alice vouches for bob using legacy struct MyVouches
    vouch::legacy_vouch_for(alice, @0x1000b);

    // carol vouches for bob using legacy struct MyVouches
    vouch::legacy_vouch_for(carol, @0x1000b);

    // alice vouches for carol using legacy struct MyVouches
    vouch::legacy_vouch_for(alice, @0x1000c);

    // check bob received vouches on legacy struct MyVouches
    let (received_vouches, received_epochs) = vouch::get_legacy_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a, @0x1000c], 73570004);
    assert!(received_epochs == vector[0, 0], 73570005);

    // check legacy validators only have legacy struct MyVouches
    assert!(vouch::is_legacy_init(@0x1000a), 73570001);
    assert!(vouch::is_legacy_init(@0x1000b), 73570002);
    assert!(vouch::is_legacy_init(@0x1000c), 73570003);
    assert!(!vouch::is_init(@0x1000a), 73570001);
    assert!(!vouch::is_init(@0x1000b), 73570002);
    assert!(!vouch::is_init(@0x1000c), 73570003);

    // check new validators only have new structs initialized
    assert!(vouch::is_init(@0x1000d), 73570006);
    assert!(vouch::is_init(@0x1000e), 73570007);
    assert!(!vouch::is_legacy_init(@0x1000d), 73570008);
    assert!(!vouch::is_legacy_init(@0x1000e), 73570009);

    // check legacy vouches
    let (received_vouches, received_epochs) = vouch::get_legacy_vouches(@0x1000a);
    assert!(received_vouches == vector::empty(), 73570010);
    assert!(received_epochs == vector::empty(), 73570011);
    let (received_vouches, received_epochs) = vouch::get_legacy_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a, @0x1000c], 73570012);
    assert!(received_epochs == vector[0, 0], 73570013);
    let (received_vouches, received_epochs) = vouch::get_legacy_vouches(@0x1000c);
    assert!(received_vouches == vector[@0x1000a], 73570014);
    assert!(received_epochs == vector[0], 73570015);

    // dave vouches for eve using new structs
    vouch::vouch_for(dave, @0x1000e);

    // eve vouches for frank using new structs
    vouch::vouch_for(eve, @0x1000f);

    // check eve received vouches on new struct Vouches
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000e);
    assert!(received_vouches == vector[@0x1000d], 73570006);
    assert!(received_epochs == vector[0], 73570007);

    // check dave given vouches on new struct Vouches
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000d);
    assert!(given_vouches == vector[@0x1000e], 73570008);
    assert!(given_epochs == vector[0], 73570009);

    // check last migration number
    assert!(migrations::get_last_migration_number() == 0, 73570010);

    // It is show time!!! RUN MIGRATION
    mock::trigger_epoch(root);

    // check last migration number
    assert!(migrations::has_migration_executed(1), 73570011);

    // check structs initialized after migration
    assert!(vouch::is_init(@0x1000a), 73570010);
    assert!(vouch::is_init(@0x1000b), 73570011);
    assert!(vouch::is_init(@0x1000c), 73570012);
    assert!(!vouch::is_legacy_init(@0x1000a), 73570013);
    assert!(!vouch::is_legacy_init(@0x1000b), 73570014);
    assert!(!vouch::is_legacy_init(@0x1000c), 73570015);

    // check alice vouches migrated
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b, @0x1000c], 73570016);
    assert!(given_epochs == vector[0, 0], 73570017);
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000a);
    assert!(received_vouches == vector::empty(), 73570018);
    assert!(received_epochs == vector::empty(), 73570019);

    // check bob vouches migrated
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000b);
    assert!(given_vouches == vector::empty(), 73570020);
    assert!(given_epochs == vector::empty(), 73570021);
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a, @0x1000c], 73570018);
    assert!(received_epochs == vector[0, 0], 73570019);

    // check carol vouches migrated
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000c);
    assert!(given_vouches == vector[@0x1000b], 73570020);
    assert!(given_epochs == vector[0], 73570021);
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000c);
    assert!(received_vouches == vector[@0x1000a], 73570022);
    assert!(received_epochs == vector[0], 73570023);

    // check dave vouches are okay
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000d);
    assert!(given_vouches == vector[@0x1000e], 73570024);
    assert!(given_epochs == vector[0], 73570025);
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000d);
    assert!(received_vouches == vector::empty(), 73570026);
    assert!(received_epochs == vector::empty(), 73570027);

    // check eve vouches are okay
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000e);
    assert!(received_vouches == vector[@0x1000d], 73570028);
    assert!(received_epochs == vector[0], 73570029);
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000e);
    assert!(given_vouches == vector[@0x1000f], 73570030);
    assert!(given_epochs == vector[0], 73570031);

    // check frank vouches are okay
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000f);
    assert!(received_vouches == vector[@0x1000e], 73570032);
    assert!(received_epochs == vector[0], 73570033);
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000f);
    assert!(given_vouches == vector::empty(), 73570034);
    assert!(given_epochs == vector::empty(), 73570035);
  }
}
