#[test_only]

/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_account {
  use std::vector;
  use ol_framework::libra_coin;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::ancestry;
  use diem_framework::coin;
  // use diem_std::debug::print;

  // scenario: testing trying send more funds than are unlocked
  #[test(root = @ol_framework, alice_sig = @0x1000a)]
  fun test_account_create(root: signer, alice_sig: signer) {
    let alice_addr = @0x1000a;
    let bob_addr = @0x1000b;

    mock::ol_test_genesis(&root);

    let mint_cap = libra_coin::extract_mint_cap(&root);
    ol_account::create_account(&root, alice_addr);
    ol_account::deposit_coins(alice_addr, coin::test_mint(100, &mint_cap));
    coin::destroy_mint_cap(mint_cap);

    let addr_tree = ancestry::get_tree(alice_addr);
    assert!(vector::length(&addr_tree) > 0, 7357001);
    assert!(vector::contains(&addr_tree, &@0x1), 7357002);


    let (a_balance, _) = ol_account::balance(alice_addr);
    assert!(a_balance == 100, 735703);

    ol_account::transfer(&alice_sig, bob_addr, 20);
    let addr_tree = ancestry::get_tree(bob_addr);
    assert!(vector::length(&addr_tree) > 1, 7357004);
    assert!(vector::contains(&addr_tree, &alice_addr), 7357005);

  }


  #[test(root = @ol_framework, alice_sig = @0x1000a)]
  #[expected_failure(abort_code = 131082, location = 0x1::ol_account)]
  fun test_account_reject_create(root: signer, alice_sig: signer) {
    let alice_addr = @0x1000a;
    let bob_addr = @0x1000b;

    mock::ol_test_genesis(&root);

    let alice_balance = 10000 * 1000000; // with scaling
    let bob_tx_too_much = 1100 * 1000000; // above limit

    let mint_cap = libra_coin::extract_mint_cap(&root);
    ol_account::create_account(&root, alice_addr);
    ol_account::deposit_coins(alice_addr, coin::test_mint(alice_balance, &mint_cap));
    coin::destroy_mint_cap(mint_cap);

    let addr_tree = ancestry::get_tree(alice_addr);
    assert!(vector::length(&addr_tree) > 0, 7357001);
    assert!(vector::contains(&addr_tree, &@0x1), 7357002);


    let (a_balance, _) = ol_account::balance(alice_addr);
    assert!(a_balance == alice_balance, 735703);

    ol_account::transfer(&alice_sig, bob_addr, bob_tx_too_much);
    let addr_tree = ancestry::get_tree(bob_addr);
    assert!(vector::length(&addr_tree) > 1, 7357004);
    assert!(vector::contains(&addr_tree, &alice_addr), 7357005);
  }
}
