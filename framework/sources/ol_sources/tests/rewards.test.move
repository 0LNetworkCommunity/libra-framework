#[test_only]
module ol_framework::test_rewards {
  // #[test_only]
  // use std::vector;
  #[test_only]
  use ol_framework::rewards;
  #[test_only]
  use aptos_framework::chain_id;
  #[test_only]
  use ol_framework::gas_coin;
  #[test_only]
  use aptos_framework::coin;
  #[test_only]
  use ol_framework::ol_account;
  #[test_only]
  use aptos_std::debug::print;
  #[test_only]
  use ol_framework::gas_coin::GasCoin;

  #[test(root=@ol_framework, alice=@0x123)]
  fun test_pay_reward(root: &signer, alice: address) {

    chain_id::initialize_for_test(root, 4);
    let (burn_cap, mint_cap) = gas_coin::initialize_for_test(root);
    let new_coin = coin::mint(10000, &mint_cap);
    coin::destroy_burn_cap(burn_cap);
    coin::destroy_mint_cap(mint_cap);

    ol_account::create_account(alice);

    rewards::test_helper_pay_reward(root, alice, new_coin);

    let b = coin::balance<GasCoin>(alice);
    print(&b);
    assert!(b == 10000, 7357001);
  }
}