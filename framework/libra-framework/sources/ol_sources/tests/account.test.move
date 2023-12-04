#[test_only]

/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_account {
  use ol_framework::libra_coin;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use diem_framework::coin;
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



    let (a_balance, _) = ol_account::balance(alice_addr);
    assert!(a_balance == 100, 735701);

    ol_account::transfer(&alice_sig, bob_addr, 20);
  }
}
