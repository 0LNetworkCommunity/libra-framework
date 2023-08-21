
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_reconfiguration {
  use std::vector;
  use diem_framework::stake;
  use diem_framework::coin;
  use ol_framework::mock;
  use ol_framework::testnet;
  use ol_framework::gas_coin::GasCoin;
  use ol_framework::proof_of_fee;
  // use diem_std::debug::print;

  // Scenario: all genesis validators make it to next epoch
  #[test(root = @ol_framework)]
  fun reconfig_happy_case(root: signer) {
      let vals = mock::genesis_n_vals(&root, 5);
      mock::ol_initialize_coin(&root);
      mock::pof_default();
      assert!(vector::length(&vals) == 5, 7357001);
      let vals = stake::get_current_validators();
      assert!(vector::length(&vals) == 5, 7357002);
      // all vals compliant
      mock::mock_all_vals_good_performance(&root);

      assert!(coin::balance<GasCoin>(@0x1000a) == 0, 7357003);

      let (reward_one, _, _ ) = proof_of_fee::get_consensus_reward();
      // The epoch's reward BEFORE reconfiguration
      assert!(reward_one == 1000000, 7357004);
      // run ol reconfiguration
      mock::trigger_epoch(&root);

      let vals = stake::get_current_validators();

      assert!(vector::length(&vals) == 5, 7357005);
      let alice_bal = coin::balance<GasCoin>(@0x1000a);
      let (_, entry_fee, _ ) = proof_of_fee::get_consensus_reward();
      // need to check that the user paid an PoF entry fee for next epoch.
      // which means the balance will be the nominal reward, net of the PoF clearing price bid
      assert!(alice_bal == (reward_one-entry_fee), 7357006)

  }

  #[test(root = @ol_framework)]
  fun drop_non_performing(root: signer) {
      let vals = mock::genesis_n_vals(&root, 5);
      mock::ol_initialize_coin(&root);
      mock::pof_default();
      assert!(vector::length(&vals) == 5, 7357001);
      let vals = stake::get_current_validators();
      assert!(vector::length(&vals) == 5, 7357002);

      // all vals compliant
      mock::mock_all_vals_good_performance(&root);

      // make alice non performant
      mock::mock_case_4(&root, *vector::borrow(&vals, 0));

      let (reward, _, _ ) = proof_of_fee::get_consensus_reward();

      // run ol reconfiguration
      mock::trigger_epoch(&root);

      let vals = stake::get_current_validators();

      // one validator missing.
      assert!(vector::length(&vals) == 4, 7357003);
      assert!(!vector::contains(&vals, &@0x1000a), 7357004);

      let (_, entry_fee, _ ) = proof_of_fee::get_consensus_reward();

      // alice doesn't get paid
      assert!(coin::balance<GasCoin>(@0x1000a) == 0, 7357005);
      // bob does
      assert!(coin::balance<GasCoin>(@0x1000b) == reward-entry_fee, 7357006);


  }


  // Scenario: no vals qualify, but we failover with same validtors
  // need to unset testnet id, since the failover only applies to non-testnet
  #[test(root = @ol_framework)]
  fun reconfig_failover(root: signer) {
      let vals = mock::genesis_n_vals(&root, 5);
      mock::ol_initialize_coin(&root);
      testnet::unset(&root); // note: needs to happen after genesis.

      mock::pof_default();

      // validators did not perform.
      let i = 0;
      while (i < vector::length(&vals)) {
        mock::mock_case_4(&root, *vector::borrow(&vals, i));
        i = i + 1;
      };

      // run ol reconfiguration
      mock::trigger_epoch(&root);

      let vals = stake::get_current_validators();

      assert!(vector::length(&vals) == 5, 7357003);
  }
}
