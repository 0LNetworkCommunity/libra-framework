
#[test_only]
module ol_framework::test_burn {
  use ol_framework::mock;
  use ol_framework::gas_coin::{Self, GasCoin};
  use ol_framework::ol_account;
  use ol_framework::match_index;
  use ol_framework::burn;
  use ol_framework::receipts;
  use ol_framework::community_wallet;
  use ol_framework::transaction_fee;
  use aptos_framework::coin;
  use std::signer;
  use std::vector;
  use std::fixed_point32;

  // use aptos_std::debug::print;

  #[test(root = @ol_framework, alice = @0x1000a)]

  fun burn_reduces_supply(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 1);
    mock::ol_initialize_coin_and_fund_vals(root, 10000);
    let supply_pre = gas_coin::supply();

    let alice_burn = 5;
    let c = ol_account::withdraw(alice, alice_burn);
    coin::user_burn<GasCoin>(c);
    let supply = gas_coin::supply();
    assert!(supply == (supply_pre - alice_burn), 7357001);

  }



    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    fun burn_to_match_index(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.

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

      let eve_donation_to_A = 75;
      ol_account::transfer(eve, addr_A, eve_donation_to_A);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_A);
      assert!(total_funds_sent_by_eve == eve_donation_to_A, 7357001);

      let eve_donation_to_B = 25;
      ol_account::transfer(eve, addr_B, eve_donation_to_B);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_B);
      assert!(total_funds_sent_by_eve == eve_donation_to_B, 7357002);

      let qualifying = vector::singleton(addr_A);
      vector::push_back(&mut qualifying, addr_B);

      // simulate index recalculation at epoch boundary
      match_index::test_reset_ratio(root, qualifying);
      let (_addrs_vec, _index_vec, ratio_vec) = match_index::get_ratios();

      // so far addr_A should have 75 coins from Eve's transfer
      let(_, balance_pre) = ol_account::balance(addr_A);
      assert!(balance_pre == eve_donation_to_A, 7357003);

      // let's burn some of bob's coins
      // first he sets his burn preference
      burn::set_send_community(bob, true);
      let bob_burn = 100000;
      let c = ol_account::withdraw(bob, bob_burn);
      // then the system burns based on his preference.
      burn::burn_with_user_preference(root, signer::address_of(bob), c);
      // check that a recycle to match index happened instead of a straight burn
      let (_lifetime_burn, lifetime_match) = burn::get_lifetime_tracker();
      assert!(lifetime_match > 0, 7357003);


      // the balance of address A should be increase by the ratio above 75% *  bob's burn of 100_000
      let ratio_A = vector::borrow(&ratio_vec, 0);
      let bob_burn_share_A = fixed_point32::multiply_u64(bob_burn, *ratio_A);

      let(_, balance) = ol_account::balance(addr_A);
      assert!(balance > balance_pre, 7357004);
      assert!(balance == (bob_burn_share_A + eve_donation_to_A), 7357005);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    fun simple_burn(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.

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

      let eve_donation_to_A = 75;
      ol_account::transfer(eve, addr_A, eve_donation_to_A);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_A);
      assert!(total_funds_sent_by_eve == eve_donation_to_A, 7357001);

      let eve_donation_to_B = 25;
      ol_account::transfer(eve, addr_B, eve_donation_to_B);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_B);
      assert!(total_funds_sent_by_eve == eve_donation_to_B, 7357002);

      let qualifying = vector::singleton(addr_A);
      vector::push_back(&mut qualifying, addr_B);

      // simulate index recalculation at epoch boundary
      match_index::test_reset_ratio(root, qualifying);
      // let (_addrs_vec, _index_vec, ratio_vec) = match_index::get_ratios();

      // so far addr_A should have 75 coins from Eve's transfer
      let(_, balance_pre) = ol_account::balance(addr_A);
      assert!(balance_pre == eve_donation_to_A, 7357003);

      // let's burn some of bob's coins
      // first he sets his burn preference
      // IMPORTANT: BOB IS SETTING PREFERENCE TO SIMPLE BURN
      burn::set_send_community(bob, false);
      let bob_burn = 100000;
      let c = ol_account::withdraw(bob, bob_burn);
      // then the system burns based on his preference.
      burn::burn_with_user_preference(root, signer::address_of(bob), c);

      // no match recycling happened
      let (lifetime_burn, lifetime_match) = burn::get_lifetime_tracker();
      assert!(lifetime_match == 0, 7357003);
      assert!(lifetime_burn > 0, 7357004);


      // NONE OF THE MATCH INDEX ACCOUNTS HAVE ANY CHANGE
      let(_, balance) = ol_account::balance(addr_A);
      assert!(balance == balance_pre, 7357005);
      assert!(balance == eve_donation_to_A, 7357006);
    }


    #[test(root = @ol_framework, alice = @0x1000a)]
    fun epoch_fees_burn(root: &signer, alice: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.

      let n_vals = 5;
      let _vals = mock::genesis_n_vals(root, n_vals); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);

      let supply_pre = gas_coin::supply();
      assert!(supply_pre == (n_vals * 100000), 73570000);

      // put some fees in the system fee account
      let alice_fee = 100000;
      let c = ol_account::withdraw(alice, alice_fee);
      transaction_fee::user_pay_fee(alice, c);

      let fees = transaction_fee::system_fees_collected();
      assert!(fees == alice_fee, 73570001);

      mock::trigger_epoch(root);

      let fees = transaction_fee::system_fees_collected();
      assert!(fees == 0, 73570002);

      let supply = gas_coin::supply();
      assert!(supply == (supply_pre-alice_fee), 73570003);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    fun epoch_fees_match(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.

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

      let eve_donation_to_A = 75;
      ol_account::transfer(eve, addr_A, eve_donation_to_A);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_A);
      assert!(total_funds_sent_by_eve == eve_donation_to_A, 7357001);

      let eve_donation_to_B = 25;
      ol_account::transfer(eve, addr_B, eve_donation_to_B);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), addr_B);
      assert!(total_funds_sent_by_eve == eve_donation_to_B, 7357002);

      let qualifying = vector::singleton(addr_A);
      vector::push_back(&mut qualifying, addr_B);

      // simulate index recalculation at epoch boundary
      match_index::test_reset_ratio(root, qualifying);
      let (_addrs_vec, _index_vec, ratio_vec) = match_index::get_ratios();

      // so far addr_A should have 75 coins from Eve's transfer
      let(_, balance_pre) = ol_account::balance(addr_A);
      assert!(balance_pre == eve_donation_to_A, 7357003);

      // let's burn some of bob's coins
      // first he sets his burn preference
      burn::set_send_community(bob, true);
      let bob_burn = 100000;
      let c = ol_account::withdraw(bob, bob_burn);
      // then the system burns based on his preference.
      burn::burn_with_user_preference(root, signer::address_of(bob), c);
      // check that a recycle to match index happened instead of a straight burn
      let (_lifetime_burn, lifetime_match) = burn::get_lifetime_tracker();
      assert!(lifetime_match > 0, 7357003);


      // the balance of address A should be increase by the ratio above 75% *  bob's burn of 100_000
      let ratio_A = vector::borrow(&ratio_vec, 0);
      let bob_burn_share_A = fixed_point32::multiply_u64(bob_burn, *ratio_A);

      let(_, balance) = ol_account::balance(addr_A);
      assert!(balance > balance_pre, 7357004);
      assert!(balance == (bob_burn_share_A + eve_donation_to_A), 7357005);
    }

    // #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    // fun fee_makers_calc(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
    //   // Scenario:
    //   assert!(TransactionFee::get_fees_collected()==0, 735701);
    //   let coin = Diem::mint<GAS>(&vm, 1);
    //   TransactionFee::pay_fee_and_track(@Alice, coin);

    //   let fee_makers = TransactionFee::get_fee_makers();
    //   // print(&fee_makers);
    //   assert!(Vector::length(&fee_makers)==1, 735702);
    //   assert!(TransactionFee::get_fees_collected()==1, 735703);
    // }

     #[test(root=@ol_framework, alice=@0x1000a)]
    fun track_fees(root: &signer, alice: &signer) {
      // use ol_framework::gas_coin;
      let _vals = mock::genesis_n_vals(root, 1); // need to include eve to init funds

      let (burn, mint) = gas_coin::initialize_for_test(root);

      coin::destroy_burn_cap(burn);
      // gas_coin::restore_mint_cap(root, mint);
      // coin::destroy_mint_cap(mint);

    //   assert!(TransactionFee::get_fees_collected()==0, 735701);
      let coin = coin::mint<GasCoin>(100, &mint);
      coin::destroy_mint_cap(mint);

      transaction_fee::user_pay_fee(alice, coin);
      // coin::destroy_mint_cap(mint);

      // transaction_fee::pay_fee(alice, coin);

    //   let fee_makers = TransactionFee::get_fee_makers();
    //   // print(&fee_makers);
    //   assert!(Vector::length(&fee_makers)==1, 735702);
    //   assert!(TransactionFee::get_fees_collected()==1, 735703);

    }
}