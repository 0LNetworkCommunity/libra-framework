
#[test_only]
module ol_framework::test_make_whole {
  use ol_framework::mock;
  use ol_framework::make_whole;

  use ol_framework::libra_coin;
  use ol_framework::ol_account;
  // use ol_framework::match_index;
  // use ol_framework::burn;
  // use ol_framework::receipts;
  // use ol_framework::community_wallet;
  // use ol_framework::transaction_fee;
  // use ol_framework::fee_maker;
  // use diem_framework::coin;
  // use std::signer;
  // use std::vector;
  // use std::option;
  // use std::fixed_point32;

  // use diem_std::debug::print;

  struct TestOops has key {}
  #[test(root = @ol_framework, alice = @0x1000a)]
  fun test_incident_init(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 1);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();

    let alice_burn = 5;
    let coin = ol_account::withdraw(alice, alice_burn);

    make_whole::init_incident<TestOops>(alice, coin, false);

    let supply = libra_coin::supply();
    assert!(supply == supply_pre, 7357001);

  }

}
