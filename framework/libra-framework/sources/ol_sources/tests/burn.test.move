
#[test_only]
module ol_framework::test_burn {
  use ol_framework::mock;
  use ol_framework::libra_coin;
  use ol_framework::ol_account;
  use ol_framework::match_index;
  use ol_framework::burn;
  use ol_framework::receipts;
  use ol_framework::community_wallet;
  use ol_framework::transaction_fee;
  use ol_framework::fee_maker;
  use diem_framework::coin;
  use std::signer;
  use std::vector;
  use std::option;
  use std::fixed_point32;

  // use diem_std::debug::print;

  #[test(root = @ol_framework, alice = @0x1000a)]

  fun burn_reduces_supply(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 1);
    mock::ol_initialize_coin_and_fund_vals(root, 10000, true);
    let supply_pre = libra_coin::supply();

    let alice_burn = 5;
    let c = ol_account::withdraw(alice, alice_burn);
    burn::burn_and_track(c);
    let supply = libra_coin::supply();
    assert!(supply == (supply_pre - alice_burn), 7357001);

  }



    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    fun burn_to_match_index(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 1000000, true);
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
      burn::vm_burn_with_user_preference(root, signer::address_of(bob), c);
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
      mock::ol_initialize_coin_and_fund_vals(root, 1000000, true);
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
      burn::vm_burn_with_user_preference(root, signer::address_of(bob), c);

      // no match recycling happened
      let (lifetime_burn, lifetime_match) = burn::get_lifetime_tracker();
      assert!(lifetime_match == 0, 7357003);
      assert!(lifetime_burn > 0, 7357004);


      // NONE OF THE MATCH INDEX ACCOUNTS HAVE ANY CHANGE
      let(_, balance) = ol_account::balance(addr_A);
      assert!(balance == balance_pre, 7357005);
      assert!(balance == eve_donation_to_A, 7357006);
    }


    #[test(root = @ol_framework)]
    fun epoch_fees_burn(root: &signer) {
      // Scenario:
      // the network starts.
      // we mock some tx fees going into the collection
      // the epoch turns and the fee account should be empty
      // and the supply should be lower

      let n_vals = 5;
      let _vals = mock::genesis_n_vals(root, n_vals); // need to include eve to init funds
      let genesis_mint = 1000000; // 1 coin per
      let epoch_reward = genesis_mint; // just to be explicit
      mock::ol_initialize_coin_and_fund_vals(root, genesis_mint, true);
      let supply_pre = libra_coin::supply();
      let mocked_tx_fees = 1000000 * 100; // 100 coins in tx fee account
      // 105 coins total
      assert!(supply_pre == mocked_tx_fees + (n_vals * genesis_mint), 73570001);


      let fees = transaction_fee::system_fees_collected();
      assert!(fees == mocked_tx_fees, 73570002);
      // start at epoch 1. NOTE Validators WILL GET PAID FOR EPOCH 1
      mock::trigger_epoch(root);
      let fees = transaction_fee::system_fees_collected();
      // There should be no fees left in tx fee wallet
      assert!(fees == 0, 73570003);

      let validator_rewards = epoch_reward * n_vals;

      // of the initial supply,
      // we expect to burn everything which was in the TX FEEs AFTER PAYING VALIDATOR REWARDS
      let amount_burned_excess_tx_account = mocked_tx_fees - validator_rewards;

      // So the current supply should be lower,
      // The the old supply was reduced by what was burned (the excess in tx bucket)

      let supply_post = libra_coin::supply();

      assert!(supply_post == supply_pre - amount_burned_excess_tx_account, 73570003);

      // this ALSO means that the only supply left in this example
      // is the rewards to validators from genesis, and from epoch 1
      assert!(supply_post == validator_rewards + (n_vals * genesis_mint), 73570003);

    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000d, eve = @0x1000e)]
    fun epoch_fees_match(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
      // Scenario:
      // There are two community wallets. Alice and Bob's
      // EVE donates to both. But mostly to Alice.
      // The Match Index, should reflect that.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 1000000, true);
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
      burn::vm_burn_with_user_preference(root, signer::address_of(bob), c);
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
    //   assert!(Vector::length(&fee_makers)==1, 735702);
    //   assert!(TransactionFee::get_fees_collected()==1, 735703);
    // }

    #[test(root=@ol_framework, alice=@0x1000a)]
    fun track_fees(root: &signer, alice: address) {
      // use ol_framework::libra_coin;
      let _vals = mock::genesis_n_vals(root, 1); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 10000, true);

      let marlon_rando = @0x12345;
      ol_account::create_account(root, marlon_rando);


      let rando_money = 5;
      let coin_option = ol_account::test_vm_withdraw(root, alice, rando_money);

      if (option::is_some(&coin_option)) {
        let c = option::extract(&mut coin_option);
        // shortcut: we have the VM use alices money to pay a fee for marlon.
        transaction_fee::vm_pay_fee(root, marlon_rando, c);
      };
      option::destroy_none(coin_option);


      let fees = transaction_fee::system_fees_collected();
      assert!(fees == 100000005, 735701);
      // Fees will include the initialization by MOCK. ol_initialize_coin_and_fund_vals

      // marlon is the only fee maker (since genesis)
      let fee_makers = fee_maker::get_fee_makers();
      // includes 0x1 which makes a deposit on
      assert!(vector::length(&fee_makers)==1, 735702);

      let marlon_fees_made = fee_maker::get_user_fees_made(marlon_rando);
      assert!(marlon_fees_made == 5, 735703);

      mock::trigger_epoch(root);
      // should reset the fee counter on everything.
      // TODO: no proof of fee was charged on alice. But this is changed in
      // feature branch `coin-methods-hygiene`

      let marlon_fees_made = fee_maker::get_user_fees_made(marlon_rando);
      assert!(marlon_fees_made == 0, 735704);
      let fee_makers = fee_maker::get_fee_makers();
      assert!(vector::length(&fee_makers)==0, 735705);
      let fees = transaction_fee::system_fees_collected();
      assert!(fees == 0, 7357008);

    }


  #[test(root = @ol_framework)]
  fun init_burn_tracker(root: &signer) {
    mock::genesis_n_vals(root, 1);
    let genesis_mint = 12345;
    mock::ol_initialize_coin_and_fund_vals(root, genesis_mint, true);

    let (prev_supply, prev_balance, burn_at_last_calc, cumu_burn) = ol_account::get_burn_tracker(@0x1000a);
    // everything initialized correctly
    assert!(prev_supply == 100000000, 7357001);
    assert!(prev_balance == 12345, 7357002);
    assert!(burn_at_last_calc == 0, 7357003);
    assert!(cumu_burn == 0, 7357004);

  }

  #[test(root = @ol_framework, alice=@0x1000a)]
  fun burn_tracker_increment(root: &signer, alice: &signer) {
    mock::genesis_n_vals(root, 2);
    let genesis_mint = 12345;
    mock::ol_initialize_coin_and_fund_vals(root, genesis_mint, true);

    // check init
    let (prev_supply, prev_balance, burn_at_last_calc, cumu_burn) =
    ol_account::get_burn_tracker(@0x1000a);
    // everything initialized correctly
    assert!(prev_supply == 100000000, 7357001);
    assert!(prev_balance == 12345, 7357002);
    assert!(burn_at_last_calc == 0, 7357003);
    assert!(cumu_burn == 0, 7357004);

    // simulate epoch boundary burn
    let all_fees = transaction_fee::test_root_withdraw_all(root);
    burn::epoch_burn_fees(root, &mut all_fees);
    coin::destroy_zero(all_fees); // destroy the hot potato

    // there should be no change at this point
    let (prev_supply_1, prev_balance_1, burn_at_last_calc, cumu_burn) =
    ol_account::get_burn_tracker(@0x1000a);
    assert!(prev_supply == prev_supply_1, 7357005);
    assert!(prev_balance == prev_balance_1, 7357006);

    ol_account::transfer(alice, @0x1000b, 100);

    // burn tracker will change on each transaction
    let (prev_supply_2, prev_balance_2, burn_at_last_calc_2, cumu_burn_2) =
    ol_account::get_burn_tracker(@0x1000a);

    // supply must go down
    assert!(prev_supply_2 < prev_supply, 7357007);
    // alice balance must go down after alice sent to bob
    assert!(prev_balance_2 < prev_balance, 7357008);
    // the last burn should be higher than 0 at init
    assert!(burn_at_last_calc_2 > burn_at_last_calc, 7357009);
    // the cumu burn should be higher
    assert!(cumu_burn_2 > cumu_burn, 7357010);

  }
}
