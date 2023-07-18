
#[test_only]
module ol_framework::test_burn {
  use ol_framework::mock;
  use ol_framework::gas_coin::{Self, GasCoin};
  use ol_framework::ol_account;
  use ol_framework::match_index;
  use ol_framework::burn;
  use ol_framework::receipts;
  use ol_framework::community_wallet;
  use aptos_framework::coin;
  use aptos_std::debug::print;
  use std::signer;
  use std::vector;

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



    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    fun test_match_index(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.
      // After several epochs, EVE prefers Bob, and the proportion should change.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);

      let (communityA, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let addr_A = signer::address_of(&communityA);
      community_wallet::init_community(&communityA, vals); // make all the vals signers

      let (communityB, _cap) = ol_account::ol_create_resource_account(bob, b"0xdeadbeef");
      let addr_B = signer::address_of(&communityB);
      community_wallet::init_community(&communityB, vals); // make all the vals signers

      let eve_donation_to_A = 42;
      ol_account::transfer(eve, addr_A, eve_donation_to_A);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_A);
      assert!(total_funds_sent_by_eve == eve_donation_to_A, 7357001);

      let eve_donation_to_B = 1;
      ol_account::transfer(eve, addr_B, eve_donation_to_B);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_B);
      assert!(total_funds_sent_by_eve == eve_donation_to_B, 7357002);

      let qualifying = vector::singleton(addr_A);
      vector::push_back(&mut qualifying, addr_B);

      match_index::test_reset_ratio(root, qualifying);
      let (a, b, c) = match_index::get_ratios();

      let c = ol_account::withdraw(alice, 100000);

      burn::set_send_community(alice, true);
      burn::burn_with_user_preference(root, @0x1000a, c);
      // burn::epoch_burn_fees(root, 100);


      let(_, balance_pre) = ol_account::balance(addr_A);
      print(&balance_pre);

      let (_lifetime_burn, lifetime_match) = burn::get_lifetime_tracker();
      // print(&lifetime_burn);
      // print(&lifetime_match);
      assert!(lifetime_match > 0, 7357003);

      let(_, balance) = ol_account::balance(addr_A);
      print(&balance);

      // nothing should have been burned, it was a refund
      // assert!(balance > balance_pre, 7357004);
    }
}