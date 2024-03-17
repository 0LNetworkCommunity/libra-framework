#[test_only]
module ol_framework::test_rewards {
  use diem_framework::coin;
  use diem_framework::stake;
  use ol_framework::burn;
  use ol_framework::libra_coin;
  use ol_framework::mock;
  use ol_framework::rewards;

  // use diem_std::debug::print;

  #[test(root = @ol_framework)]
  fun test_pay_reward_single(root: signer) {

    let alice = @0x1000a;

    mock::genesis_n_vals(&root, 1);
    let mint_cap = libra_coin::extract_mint_cap(&root);

    let uid_before = stake::get_reward_event_guid(alice);

    let new_coin = coin::test_mint(10000, &mint_cap);

    coin::destroy_mint_cap(mint_cap);

    rewards::test_helper_pay_reward(&root, alice, new_coin, 1);
    let uid_after = stake::get_reward_event_guid(alice);
    assert!(uid_after > uid_before, 7357001);

    let b = libra_coin::balance(alice);
    assert!(b == 10000, 7357002);
  }

  #[test(root=@ol_framework)]
  fun test_pay_reward_all(root: &signer) {

    let alice = @0x1000a;
    let dave = @0x1000d;

    let vals = mock::genesis_n_vals(root, 4);
    let mint_cap = libra_coin::extract_mint_cap(root);
    let uid_before = stake::get_reward_event_guid(dave);
    assert!(uid_before == 0, 7357000);
    let new_coin = coin::test_mint(10000, &mint_cap);

    coin::destroy_mint_cap(mint_cap);

    rewards::test_process_recipients(root, vals, 1000, &mut new_coin, 1);

    let uid_after = stake::get_reward_event_guid(dave);
    assert!(uid_after > uid_before, 7357001);

    let b = libra_coin::balance(alice);
    assert!(b == 1000, 7357002);

    let b = libra_coin::balance(dave);
    assert!(b == 1000, 7357002);

    burn::burn_and_track(new_coin);
  }
}
