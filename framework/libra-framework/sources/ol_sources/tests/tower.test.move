#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.

// NOTE: Cousin Alice: we have the Move Alice account 1000a, but in the VDF fixtures we
// have RUST alice 0x8751. The VDF fixture uses this one.
module ol_framework::test_tower {
  use ol_framework::mock;
  use ol_framework::tower_state;
  use ol_framework::vdf_fixtures;
  use std::signer;
  use ol_framework::ol_account;
  use ol_framework::proof_of_fee;
  use ol_framework::stake;
  use diem_framework::timestamp;
  // use diem_framework::coin;
  // use ol_framework::gas_coin::LibraCoin as GasCoin;
  use std::vector;

  use std::debug::print;

  #[test(root = @ol_framework)]
  fun epoch_changes_difficulty(root: signer) {
    mock::genesis_n_vals(&root, 4);
    // mock::ol_initialize_coin(&root);

    mock::tower_default(&root); // make all the validators initialize towers
    // because we need randomness for the toy rng

    let (diff, sec) = tower_state::get_difficulty();

    // check the state started with the testnet defaults
    assert!(diff==100, 735701);
    assert!(sec==512, 735702);

    mock::trigger_epoch(&root);

    let (diff, _sec) = tower_state::get_difficulty();
    assert!(diff!=100, 735703);
  }

  #[test(root = @ol_framework, cousin_alice = @0x87515d94a244235a1433d7117bc0cb154c613c2f4b1e67ca8d98a542ee3f59f5)]
  fun init_tower_state(root: signer, cousin_alice: signer){
    mock::ol_test_genesis(&root);
    // mock::ol_initialize_coin(&root);
    timestamp::fast_forward_seconds(1);
    ol_account::create_account(&root, signer::address_of(&cousin_alice));

    tower_state::init_miner_state(
          &cousin_alice,
          &vdf_fixtures::alice_0_easy_chal(),
          &vdf_fixtures::alice_0_easy_sol(),
          vdf_fixtures::easy_difficulty(),
          vdf_fixtures::security(),
      );

    }


  #[test(root = @ol_framework)]
  fun toy_rng_state(root: signer) {
    mock::genesis_n_vals(&root, 4);
    // mock::ol_initialize_coin(&root);

    mock::tower_default(&root); // make all the validators initialize towers
    // because we need randomness for the toy rng

    let num = tower_state::toy_rng(1, 3, 0);

    assert!(num == 116, 7357001);

  }

  #[test(root = @ol_framework)]
  fun toy_rng_minimal(root: signer) {
    // use diem_std::debug::print;
    mock::genesis_n_vals(&root, 1);
    // mock::ol_initialize_coin(&root);

    mock::tower_default(&root); // make all the validators initialize towers
    // because we need randomness for the toy rng

    let num = tower_state::toy_rng(0, 1, 0);
    assert!(num == 116, 7357001);

  }


  #[test(root = @ol_framework, cousin_alice = @0x87515d94a244235a1433d7117bc0cb154c613c2f4b1e67ca8d98a542ee3f59f5)]
  fun miner_receives_reward(root: signer, cousin_alice: signer) {
    let a_addr = signer::address_of(&cousin_alice);
    let vals = mock::genesis_n_vals(&root, 5);
    // mock::ol_initialize_coin(&root);
    mock::pof_default();
    ol_account::create_account(&root, a_addr);
    proof_of_fee::fill_seats_and_get_price(&root, 5, &vals, &vals);

    assert!(vector::length(&vals) == 5, 7357001);
    let vals = stake::get_current_validators();
    assert!(vector::length(&vals) == 5, 7357002);

    tower_state::minerstate_commit(
      &cousin_alice,
      vdf_fixtures::alice_0_easy_chal(),
      vdf_fixtures::alice_0_easy_sol(),
      vdf_fixtures::easy_difficulty(),
      vdf_fixtures::security(),
    );

    let count_pre = tower_state::get_count_in_epoch(a_addr);
    print(&count_pre);

    // all vals compliant
    mock::mock_all_vals_good_performance(&root);

    let (_, alice_bal_pre) = ol_account::balance(@0x1000a);
    assert!(alice_bal_pre == 0, 73570003);

    mock::trigger_epoch(&root);

    let (_, alice_bal) = ol_account::balance(@0x1000a);

    assert!(alice_bal_pre < alice_bal, 73570003);
  }

}