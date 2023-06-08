#[test_only]
module ol_framework::test_rewards {
  // #[test_only]
  // use std::vector;
  #[test_only]
  use ol_framework::rewards;
  // #[test_only]
  // use ol_framework::ol_account;
  #[test_only]
  use ol_framework::gas_coin::{Self, GasCoin};
  #[test_only]
  use aptos_framework::coin;
  #[test_only]
  use ol_framework::mock;

  // #[test_only]
  // use ol_framework::gas_coin::GasCoin;
  #[test_only]
  use aptos_framework::stake;


  // #[test_only]
  // use aptos_std::debug::print;

  #[test(root = @ol_framework)]
  fun test_pay_reward_single(root: signer) {

    let alice = @0x1000a;

    mock::genesis_n_vals(&root, 1);
    let uid_before = stake::get_reward_event_guid(alice);

    let (burn_cap, mint_cap) = gas_coin::initialize_for_test_without_aggregator_factory(&root);
    let new_coin = coin::mint(10000, &mint_cap);
    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);

    rewards::test_helper_pay_reward(&root, alice, new_coin, 1);
    let uid_after = stake::get_reward_event_guid(alice);
    assert!(uid_after > uid_before, 7357001);

    let b = coin::balance<GasCoin>(alice);
    assert!(b == 10000, 7357002);
  }

  #[test(root=@ol_framework)]
  fun test_pay_reward_all(root: &signer) {

    let alice = @0x1000a;
    let dave = @0x1000d;

    let vals = mock::genesis_n_vals(root, 4);
    let uid_before = stake::get_reward_event_guid(dave);
    assert!(uid_before == 0, 7357000);

    let (burn_cap, mint_cap) = gas_coin::initialize_for_test_without_aggregator_factory(root);
    let new_coin = coin::mint(10000, &mint_cap);
    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);

    rewards::test_process_recipients(root, vals, 1000, &mut new_coin, 1);

    let uid_after = stake::get_reward_event_guid(dave);
    assert!(uid_after > uid_before, 7357001);

    let b = coin::balance<GasCoin>(alice);
    assert!(b == 1000, 7357002);

    let b = coin::balance<GasCoin>(dave);
    assert!(b == 1000, 7357002);

    coin::user_burn(new_coin);
  }
}