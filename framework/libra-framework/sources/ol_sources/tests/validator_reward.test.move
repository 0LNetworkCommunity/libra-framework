
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_reconfiguration {
  use std::vector;
  use diem_framework::stake;
  use diem_framework::transaction_fee;
  // use diem_framework::coin;
  use ol_framework::mock;
  use ol_framework::testnet;
  use ol_framework::libra_coin;
  use ol_framework::proof_of_fee;
  use diem_framework::reconfiguration;
  use ol_framework::epoch_helper;
  use ol_framework::ol_account;
  use ol_framework::infra_escrow;

   use diem_std::debug::print;

  // Scenario: all genesis validators make it to next epoch
  #[test(root = @ol_framework)]
  fun reconfig_reward_happy_case(root: signer) {
    let vals = mock::genesis_n_vals(&root, 5);

    mock::pof_default(&root);
    assert!(vector::length(&vals) == 5, 7357001);
    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 5, 7357002);
    // all vals compliant
    mock::mock_all_vals_good_performance(&root);

    let (unlocked, alice_bal) = ol_account::balance(@0x1000a);
    assert!(unlocked==0, 7367001);
    assert!(alice_bal==0, 7357002);

    let (reward_one, _entry_fee, _, _ ) = proof_of_fee::get_consensus_reward();

    // The epoch's reward BEFORE reconfiguration
    assert!(reward_one == 1000000, 7357004);

    let infra = infra_escrow::infra_escrow_balance();
    print(&555);
    print(&infra);

    let subsidy = transaction_fee::system_fees_collected();
    print(&666);
    print(&subsidy);

    // run ol reconfiguration
    mock::trigger_epoch(&root);

    let infra = infra_escrow::infra_escrow_balance();
    print(&5552);
    print(&infra);

    let subsidy = transaction_fee::system_fees_collected();
    print(&6662);
    print(&subsidy);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 5, 7357005);
    // let alice_bal = libra_coin::balance(@0x1000a);
    let (_unlocked, alice_bal) = ol_account::balance(@0x1000a);

    let (_, entry_fee, _,  _ ) = proof_of_fee::get_consensus_reward();
    // need to check that the user paid an PoF entry fee for next epoch.
    // which means the balance will be the nominal reward, net of the PoF clearing price bid
    assert!(alice_bal == (reward_one - entry_fee), 7357006);

    // test new subsidy
    let (reward_two, _entry_fee, _, _ ) = proof_of_fee::get_consensus_reward();
    print(&777);
    print(&reward_two);
    let new_budget = reward_two * 5;
    print(&new_budget);

    let subsidy = transaction_fee::system_fees_collected();
    print(&888);
    print(&subsidy);

  }

  #[test(root = @ol_framework)]
  fun drop_non_performing(root: signer) {
    let _vals = mock::genesis_n_vals(&root, 5);
    mock::ol_initialize_coin_and_fund_vals(&root, 5555 * 1_000_000, false);
    mock::pof_default(&root);
    assert!(libra_coin::balance(@0x1000a) == 0, 7357000);

    // NOTE: epoch 0 and 1 are a special case, we don't run performance grades on that one. Need to move two epochs ahead
    reconfiguration::test_helper_increment_epoch_dont_reconfigure(1);
    reconfiguration::test_helper_increment_epoch_dont_reconfigure(1);

    assert!(epoch_helper::get_current_epoch() == 2, 7357001);

    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 5, 7357002);

    // all vals compliant
    mock::mock_all_vals_good_performance(&root);

    // make alice non performant
    mock::mock_case_4(&root, *vector::borrow(&vals, 0));

    let (reward, _, _, _ ) = proof_of_fee::get_consensus_reward();

    // run ol reconfiguration
    mock::trigger_epoch(&root);

    let vals = stake::get_current_validators();

    // one validator missing.
    assert!(vector::length(&vals) == 4, 7357003);
    assert!(!vector::contains(&vals, &@0x1000a), 7357004);

    let (_, entry_fee, _, _ ) = proof_of_fee::get_consensus_reward();

    // alice doesn't get paid
    assert!(libra_coin::balance(@0x1000a) == 0, 7357005);
    // bob does
    assert!(libra_coin::balance(@0x1000b) == (reward - entry_fee), 7357006);
  }


  // Scenario: no vals qualify, but we failover with same validtors
  // need to unset testnet id, since the failover only applies to non-testnet
  #[test(root = @ol_framework)]
  fun reconfig_failover(root: signer) {
      let vals = mock::genesis_n_vals(&root, 5);
      // mock::ol_initialize_coin(&root);

      mock::pof_default(&root);

      // validators did not perform.
      let i = 0;
      while (i < vector::length(&vals)) {
        mock::mock_case_4(&root, *vector::borrow(&vals, i));
        i = i + 1;
      };

      testnet::unset(&root); // note: needs to happen after genesis.

      // run ol reconfiguration
      mock::trigger_epoch(&root);

      let vals = stake::get_current_validators();

      assert!(vector::length(&vals) == 5, 7357003);
  }
}
