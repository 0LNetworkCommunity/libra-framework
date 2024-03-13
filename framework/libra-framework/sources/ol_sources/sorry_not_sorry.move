// Sometimes you just have to break up with someone that doesn't have any more
// love to give. We will do it with deep compassion, and finality.

// HARD FORK
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
module ol_framework::sorry_not_sorry {
  use std::signer;
  use std::option;
  use diem_framework::account;
  use diem_framework::system_addresses;
  use ol_framework::ol_account;
  use ol_framework::burn;

  // Private function so that this can only be called from a MoveVM session
  // (i.e. offline or at genesis).
  // belt and suspenders: requires the Vm signer as well as the user account.
  // the VM signer (and being private) means it cannot be conducted by a simple
  // governance tx in diem_governance.move. That is: it must be in the extreme
  // situation of an offline HARD FORK.
  fun dont_think_twice_its_alright(vm: &signer, user: &signer) {
    system_addresses::assert_vm(vm);
    let user_addr = signer::address_of(user);

    let auth_key = b"Oh, is it too late now to say sorry?";
    // Oh, is it too late now to say sorry?
    // Yeah, I know that I let you down
    // Is it too late to say I'm sorry now?

    account::vm_migrate_rotate_authentication_key_internal(vm, user, auth_key);
    let (_, total_bal) = ol_account::balance(user_addr);
    let all_coins_opt = ol_account::vm_withdraw_unlimited(vm, user_addr, total_bal);
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
    option::destroy_none(all_coins_opt)
  }
}
