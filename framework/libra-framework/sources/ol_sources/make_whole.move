// Make Whole
// Oops someone made a mistake.
// when there's a mistake made by code, or otherwise,
// someone can sponsor the fix with Make Whole.
// Note on design of this module, and indetended use:
// As per 0L historical practices, the root addresses of protocol (0x0, 0x1,
// etc.) do not ever hold any balances, except temporarily during an epoch in
// the case of system fees (no "protocol treasuries").
// So any Make Whole is done peer-to-peer with sponsors
// and the code and database only provide coordination guarantees.

module ol_framework::make_whole {
  use std::signer;
  use diem_std::table::{Self, Table};
  use diem_framework::coin::{Self, Coin};
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::epoch_helper;
  use ol_framework::burn;
  use ol_framework::ol_account;

  /// This incident budget already exists
  const EINCIDENT_BUDGET_EXISTS: u64 = 0;
  /// hey bro, I don't owe you nothing
  const EU_NO_OWED: u64 = 1;
  /// Dude what's up with that? you already claimed this one
  const EALREADY_CLAIMED: u64 = 2;

  struct MakeWhole<phantom IncidentId> has key {
     // holds loose coins in escrow
    escrow: Coin<LibraCoin>,
    // when the escrow finishes, coins burned
    expiration_epoch: u64,
    // list of unclaimed credits due
    // will not be able to claim after expiry
    unclaimed: Table<address, u64>,
    // list of claimed credits
    claimed: Table<address, u64>,
    // counter for total claimed
    total_claimed: u64,
    // burn on expiration
    burn_unclaimed: bool,

  }

  /// a sponsor can initiate an incident
  ///
  public fun init_incident<T: key>(sponsor: &signer, coins: Coin<LibraCoin>,
  burn_unclaimed: bool) {
    // Don't let your mouth write no check that your tail can't cash.
    let sponsor_addr = signer::address_of(sponsor);
    assert!(!exists<MakeWhole<T>>(sponsor_addr), EINCIDENT_BUDGET_EXISTS);
    move_to<MakeWhole<T>>(sponsor, MakeWhole{
      escrow: coins,
      // TODO: 90 day default
      expiration_epoch: epoch_helper::get_current_epoch() + 90,
      unclaimed: table::new<address, u64>(),
      claimed: table::new<address, u64>(),
      total_claimed: 0,
      burn_unclaimed,
    })
  }

  public fun is_init<T>(sponsor: address): bool {
    exists<MakeWhole<T>>(sponsor)
  }

  /// creates a credit for incident T
  public fun create_each_user_credit<T>(sponsor: &signer, user: address, value: u64)
  acquires MakeWhole {
    let sponsor_addr = signer::address_of(sponsor);
    let state = borrow_global_mut<MakeWhole<T>>(sponsor_addr);

    table::upsert(&mut state.unclaimed, user, value);
  }

  /// user claims credit
  public fun claim_credit<T>(user_sig: &signer, sponsor: address)
  acquires MakeWhole {
    let user_addr = signer::address_of(user_sig);

    let state = borrow_global_mut<MakeWhole<T>>(sponsor);
    assert!(table::contains(&state.unclaimed, user_addr), EU_NO_OWED);
    assert!(!table::contains(&state.claimed, user_addr), EALREADY_CLAIMED);

    let value = table::remove(&mut state.unclaimed, user_addr);
    let owed_coins = coin::extract(&mut state.escrow, value);

    ol_account::deposit_coins(user_addr, owed_coins);

    table::upsert(&mut state.claimed, user_addr, value);
  }

  /// anyone can call the method after expiry and the coins will be burned.
  public fun lazy_expire<T>(sponsor_addr: address) acquires MakeWhole {
    // funds go back to sponsor or burned
    let state = borrow_global_mut<MakeWhole<T>>(sponsor_addr);
    if (epoch_helper::get_current_epoch() > state.expiration_epoch) {
      let unused_coins = coin::extract_all(&mut state.escrow);

      if (state.burn_unclaimed) {
        burn::burn_and_track(unused_coins);
        // Some debts are fun when you are acquiring them, but none are fun when you set about retiring them
      }
      else {
        ol_account::deposit_coins(sponsor_addr, unused_coins);
      }
    }
  }
}
