// Oop someone made a mistake.
// when there's a mistake made by code, or otherwise
// someone can sponsor the fix
// with make whole.
// As per 0L practices, the root addresses of protocol  0x0, 0x1
// do not ever hold any balances, except temporarily as
// transaction fees.

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

  struct Incident has key {
    // name of the event that needs to be remedied
    // TODO: use string
    incident_name: vector<u8>,
    // sponsor:
    sponsor: address,
  }

  struct MakeWhole<phantom Incident> has key {
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

  public fun init_incident<T: key>(sponsor: &signer, coins: Coin<LibraCoin>,
  burn_unclaimed: bool) {
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

  /// creates a credit for incident T
  public fun create_each_user_credit<T>(sponsor: &signer, user: address, value: u64)
  acquires MakeWhole {
    let sponsor_addr = signer::address_of(sponsor);
    let state = borrow_global_mut<MakeWhole<T>>(sponsor_addr);

    table::upsert(&mut state.unclaimed, user, value);
  }

  /// anyone can call the method after expiry and the coins will be burned.
  public fun lazy_expire<T>(sponsor_addr: address) acquires MakeWhole {
    // funds go back to sponsor or burned
    let state = borrow_global_mut<MakeWhole<T>>(sponsor_addr);
    if (epoch_helper::get_current_epoch() > state.expiration_epoch) {
      let unused_coins = coin::extract_all(&mut state.escrow);

      if (state.burn_unclaimed) {
        burn::burn_and_track(unused_coins);
      }
      else {
        ol_account::deposit_coins(sponsor_addr, unused_coins);
      }
    }
  }
}
