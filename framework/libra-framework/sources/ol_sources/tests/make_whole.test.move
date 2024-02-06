
#[test_only]
module ol_framework::test_make_whole {
  use ol_framework::mock;
  use ol_framework::make_whole;
  use ol_framework::libra_coin;
  use ol_framework::ol_account;
  use diem_framework::reconfiguration;
  // use diem_std::debug::print;

  // just a unique identifier for the MakeWhole
  struct TestOops has key {}

  #[test(root = @ol_framework, alice = @0x1000a)]
  fun test_incident_init(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 1);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();

    let alice_oops_amount = 5;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    make_whole::init_incident<TestOops>(alice, coin, false);

    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357001);

  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  fun test_create_credit(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();

    let alice_oops_amount = 555;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    make_whole::init_incident<TestOops>(alice, coin, false);
    make_whole::create_each_user_credit<TestOops>(alice, @0x1000b, 55);

    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357001);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  fun test_claim_credit(root: &signer, alice: &signer, bob: &signer) {
    mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();
    let (_, bob_balance_pre) = ol_account::balance(@0x1000b);

    let alice_oops_amount = 555;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    make_whole::init_incident<TestOops>(alice, coin, false);
    make_whole::create_each_user_credit<TestOops>(alice, @0x1000b, 500000);

    // bob claims it
    make_whole::claim_credit<TestOops>(bob, @0x1000a);

    let (_, bob_balance) = ol_account::balance(@0x1000b);

    assert!(bob_balance > bob_balance_pre, 7357001);
    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357002);
  }

  #[test(root = @ol_framework, alice = @0x1000a, carol =
  @0x1000c)]
  #[expected_failure]
  fun test_not_owed(root: &signer, alice: &signer, carol: &signer) {
    mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();
    // let (_, bob_balance_pre) = ol_account::balance(@0x1000b);

    let alice_oops_amount = 555;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    make_whole::init_incident<TestOops>(alice, coin, false);
    make_whole::create_each_user_credit<TestOops>(alice, @0x1000b, 55);

    // CAROL tries to claim it
    make_whole::claim_credit<TestOops>(carol, @0x1000a);

    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357002);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure]
  fun test_claim_credit_twice(root: &signer, alice: &signer, bob: &signer) {
    mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();
    let (_, bob_balance_pre) = ol_account::balance(@0x1000b);

    let alice_oops_amount = 555;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    make_whole::init_incident<TestOops>(alice, coin, false);
    make_whole::create_each_user_credit<TestOops>(alice, @0x1000b, 55);

    // bob claims it
    make_whole::claim_credit<TestOops>(bob, @0x1000a);
    make_whole::claim_credit<TestOops>(bob, @0x1000a);

    let (_, bob_balance) = ol_account::balance(@0x1000b);

    assert!(bob_balance > bob_balance_pre, 7357001);
    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357002);
  }


  #[test(root = @ol_framework, alice = @0x1000a)]
  fun test_expires_with_burn(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();

    let alice_oops_amount = 555;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    let will_burn = true;
    make_whole::init_incident<TestOops>(alice, coin, will_burn);
    make_whole::create_each_user_credit<TestOops>(alice, @0x1000b, 55);

    reconfiguration::test_helper_increment_epoch_dont_reconfigure(100);

    make_whole::lazy_expire<TestOops>(@0x1000a);

    let supply = libra_coin::supply();
    assert!(supply != supply_pre, 7357002);
  }


  #[test(root = @ol_framework, alice = @0x1000a)]
  fun test_expires_returns_to_sponsor(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();
    let (_, alice_pre_balance) = ol_account::balance(@0x1000a);

    let alice_oops_amount = 555;
    let coin = ol_account::withdraw(alice, alice_oops_amount);

    let will_burn = false;
    make_whole::init_incident<TestOops>(alice, coin, will_burn);
    make_whole::create_each_user_credit<TestOops>(alice, @0x1000b, 55);

    let (_, alice_during_balance) = ol_account::balance(@0x1000a);

    assert!(alice_during_balance < alice_pre_balance, 7357001);

    reconfiguration::test_helper_increment_epoch_dont_reconfigure(100);

    make_whole::lazy_expire<TestOops>(@0x1000a);

    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357002);

    let (_, alice_post_balance) = ol_account::balance(@0x1000a);

    assert!(alice_post_balance == alice_pre_balance, 7357003);
  }

}
