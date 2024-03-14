// LAST GOODBYE
// Sometimes you just have to break up with someone that doesn't have any more
// love to give. We will do it with deep compassion, and finality.

//// First time hard-forkers, read this first ////
// By using the code below, I'm assuming you know what you are doing.
// That is, you know that in a hard fork you are starting a new chain.
// This is not a governance action on an existing chain.
// You are leaving a chain behind and starting a new one.
// Keep this close to your heart:
// The dominant strategy for others is to not join you.
//////////////////////////////////////////////////

// Removing Accounts in Hard Forks in Diem
// Forking a chain and removing accounts is not as simple as for example on a
// UTXO chain like bitcoin, where dropping the accounts would also remove
// the coins.
// On modern blockchains the coins are connected through different modules and
// there is state that needs to be gracefully removed in a cascade.
// With MoveVm however there is no way of eliminating accounts from within the
// VM, the account is a primitive of the database.
// So forking accounts would be a two step procedure: execute some MoveVm
// transactions to gracefully sunset accounts. Then subsequently run some
// database operation (in Rust-land), to remove the address tree.
// Commit Note: luckily we've never encountered a need for this tool,
// but unfortunately it is an in-demand feature.

/// We just have to face it, this time
/// (This time, this time) we're through (we're through)
/// This time we're through, we're really through
/// Breaking up is never easy, I know, but I have (I have to go) to go
/// (This time I have to go, this time I know) knowing me, knowing you
/// It's the best I can do
module ol_framework::last_goodbye {
  use std::signer;
  use std::option;
  use std::vector;
  use diem_framework::account;
  use diem_framework::system_addresses;
  use ol_framework::burn;
  use ol_framework::libra_coin::LibraCoin;
  use diem_framework::coin;
  #[test_only]
  use ol_framework::ol_account;

  // use std::debug::print;

  // Private function so that this can only be called from a MoveVM session
  // (i.e. offline or at genesis).
  // belt and suspenders: requires the Vm signer as well as the user account.
  // the VM signer (and being private) means it cannot be conducted by a simple
  // governance tx in diem_governance.move. That is: it must be in the extreme
  // situation of an offline HARD FORK.
  fun dont_think_twice_its_alright(vm: &signer, user: &signer) {
    system_addresses::assert_vm(vm);
    let user_addr = signer::address_of(user);
    if (!account::exists_at(user_addr)) {
      return
    };

    // do all the necessary coin accounting prior to removing the account.
    let total_bal = coin::balance<LibraCoin>(user_addr);

    let all_coins_opt = coin::vm_withdraw<LibraCoin>(vm, user_addr,
    total_bal);

    // It ain't no use to sit and wonder why, babe
    // If'n you don't know by now
    // And it ain't no use to sit and wonder why, babe
    // It'll never do somehow
    // When your rooster crows at the break of dawn
    // Look out your window and I'll be gone
    // You're the reason I'm a-traveling on
    // But don't think twice, it's all right
    if (option::is_some(&all_coins_opt)) {
      let good_capital = option::extract(&mut all_coins_opt);
      burn::burn_and_track(good_capital);
    };

    option::destroy_none(all_coins_opt);

    let auth_key = b"Oh, is it too late now to say sorry?";
    vector::trim(&mut auth_key, 32);

    // Oh, is it too late now to say sorry?
    // Yeah, I know that I let you down
    // Is it too late to say I'm sorry now?


    // this will brick the account permanently
    // now the offline db tools can safely remove the key from db.
    account::rotate_authentication_key_internal(user, auth_key);

    // This is our last goodbye
    // I hate to feel the love between us die
    // But it's over
    // Just hear this and then I'll go
    // You gave me more to live for
    // More than you'll ever know
  }

  #[test(vm = @0x0, alice = @0x1000a)]
    fun k_bai(vm: &signer, alice: &signer) {
      use diem_framework::account;

      let a_addr = signer::address_of(alice);
      account::create_account_for_test(a_addr);
      assert!(account::exists_at(a_addr), 7357); // Confirm Alice's account exists

      let auth_orig = account::get_authentication_key(a_addr);

      dont_think_twice_its_alright(vm, alice);
      assert!(account::exists_at(a_addr), 7357); // Ensure the account still exists after operation

      // Verify if the authentication key changed as expected
      let auth_changed = account::get_authentication_key(a_addr);
      assert!(auth_changed != auth_orig, 7358); // Confirm the authentication key was altered
    }

    #[test(vm = @0x0, framework = @0x1, alice = @0x1000a)]
    fun k_bai_balance(vm: &signer, framework: &signer, alice: &signer) {
      use diem_framework::account;
      use ol_framework::mock;

      mock::ol_test_genesis(framework);

      let a_addr = signer::address_of(alice);
      ol_account::create_account(vm, a_addr);
      assert!(account::exists_at(a_addr), 735701); // Verify Alice's account was successfully created

      let auth_orig = account::get_authentication_key(a_addr);

      mock::ol_mint_to(framework, a_addr, 303030);

      let (_, total_bal_pre) = ol_account::balance(a_addr);
      assert!(total_bal_pre == 303030, 735702); // Ensure the balance matches expected before operation

      dont_think_twice_its_alright(vm, alice);
      assert!(account::exists_at(a_addr), 735703); // Confirm the account exists post-operation

      let (_, total_bal_post) = ol_account::balance(a_addr);
      assert!(total_bal_post == 0, 735704); // Verify the balance is zero after operation

      // New Assert: Check if the authentication key changed post-operation
      let auth_changed = account::get_authentication_key(a_addr);
      assert!(auth_changed != auth_orig, 735705); // Confirm the authentication key was altered
    }

  #[test(vm = @0x0, framework = @0x1, alice = @0x1000a)]
  fun k_bai_global_balance(vm: &signer, framework: &signer, alice: &signer) {
    use diem_framework::account;
    use ol_framework::mock;
    use ol_framework::libra_coin;

    // Initialize the framework and set initial conditions
    mock::genesis_n_vals(framework, 1);
    mock::ol_initialize_coin_and_fund_vals(framework, 10000, true);

    // Check the initial global supply
    let existing_supply_pre = libra_coin::supply();
    assert!(existing_supply_pre > 0, 735701); // Ensure there's an initial supply

    let a_addr = signer::address_of(alice);
    assert!(account::exists_at(a_addr), 735702); // Verify Alice's account exists

    // Retrieve and assert Alice's initial balance
    let (_, total_bal_pre) = ol_account::balance(a_addr);
    assert!(total_bal_pre == 10000, 735703); // Expected balance before operation

    // Perform the account modification operation
    dont_think_twice_its_alright(vm, alice);
    assert!(account::exists_at(a_addr), 735704); // Account should still exist

    // Assert the authentication key was changed
    let auth_orig_post = account::get_authentication_key(a_addr);
    assert!(auth_orig_post != b"Oh, is it too late now to say sorry?", 735705);

    // Assert Alice's balance is now 0 after the operation
    let (_, total_bal_post) = ol_account::balance(a_addr);
    assert!(total_bal_post == 0, 735706); // Balance should be zero

    // Check the global supply after the operation
    let existing_supply_post = libra_coin::supply();
    assert!(existing_supply_post < existing_supply_pre, 735707); // Global supply should decrease
  }

  #[test(vm = @0x0,framework = @0x1, alice = @0x1000a)]
  fun k_bai_effect_on_global_burns(vm: &signer, framework: &signer, alice: &signer) {
    use ol_framework::mock;
    use ol_framework::burn;
    use ol_framework::libra_coin;
    use std::signer;

    // Initialize the testing environment and mint coins to Alice's account.
    mock::genesis_n_vals(framework, 1);
    let mint_amount = 10000;
    mock::ol_initialize_coin_and_fund_vals(framework, mint_amount, true);
    
    let alice_addr = signer::address_of(alice);

    // Capture the global supply and burn counters before operation.
    let global_supply_pre = libra_coin::supply();
    let (burned_pre, recycled_pre) = burn::get_lifetime_tracker();

    dont_think_twice_its_alright(vm, alice);

    // Capture the global supply and burn counters after operation.
    let global_supply_post = libra_coin::supply();
    let (burned_post, recycled_post) = burn::get_lifetime_tracker();


    // Verify burn
    assert!(burned_post > burned_pre, 7357001);
    
    // Assert that recycled coins count remains the same, indicating no recycling occurred.
    assert!(recycled_pre == recycled_post, 7357005);

    // Assert the global supply of coins has decreased if the coins were burned,
    assert!(global_supply_post < global_supply_pre, 7357004);

    // Verify Alice's account balance is zero after the operation.
    let (_, alice_balance_post) = ol_account::balance(alice_addr);
    assert!(alice_balance_post == 0, 7357003);
  }




}
