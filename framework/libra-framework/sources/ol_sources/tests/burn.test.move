
#[test_only]
module ol_framework::test_burn {
  use ol_framework::mock;
  use ol_framework::gas_coin::{Self, GasCoin};
  use ol_framework::ol_account;
  use aptos_framework::coin;
  use aptos_std::debug::print;
  // use std::signer;

  #[test(root = @ol_framework, alice = @0x1000a)]

  fun burn_reduces_supply(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 1);
    mock::ol_initialize_coin_and_fund_vals(root, 10000);
    let supply_pre = gas_coin::supply();
    // print(&supply);

    let alice_burn = 5;
    let c = ol_account::withdraw(alice, alice_burn);
    print(&c);
    coin::user_burn<GasCoin>(c);
    let supply = gas_coin::supply();
    print(&supply);
    assert!(supply == (supply_pre - alice_burn), 7357001);

  }
}