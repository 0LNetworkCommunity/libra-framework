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
  // use aptos_std::debug::print;
  // #[test_only]
  // use ol_framework::gas_coin::GasCoin;
  // #[test_only]
  // use aptos_framework::stake;

  #[test(root=@ol_framework, alice=@0x1000a)]
  fun test_pay_reward(root: &signer, alice: address) {

    mock::genesis_n_vals(1);

    let (burn_cap, mint_cap) = gas_coin::initialize_for_test_without_aggregator_factory(root);
    let new_coin = coin::mint(10000, &mint_cap);
    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);

    rewards::test_helper_pay_reward(root, alice, new_coin, 1);

    let b = coin::balance<GasCoin>(alice);
    // print(&b);
    assert!(b == 10000, 7357001);
  }
}