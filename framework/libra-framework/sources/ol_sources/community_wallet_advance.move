// Community wallet advance module
// Ordinarily community wallets can only send
// funds to slow wallets. This means they don't have
// liquid and unlocked tokens. This was intended to
// prevent sybil attacks and other forms of abuse.
// However, there are cases where a community wallet's
// program may need liquid funds.

// This experimental feature would allow a community wallet
// to unlock tokens for sending to ordinary wallets, but only as a loan.
// It needs to be repaid within a certain time frame, or the community wallet
// will be disabled (and require re-authorization by the donors).
// Additionally, new donations to the community wallet, first go into servicing
// the loan.
// Last, there's a yearly limit of X% of the balance which the community wallet
// can loan out.

// In the design below, the Loan information is stored in the framework 0x1 account, however the coins are kept in the account state, and must be authorized with a withdraw capability.

module ol_framework::community_wallet_advance {
  use std::error;
  // use std::signer;
  use std::vector;
  use std::guid::{Self, GUID, ID};
  use diem_framework::account::{Self, WithdrawCapability};
  use diem_framework::coin::{Coin};
  use ol_framework::ol_account;
  use ol_framework::libra_coin::{LibraCoin};

  /// Error code indicating that the account does not have enough funds to complete the transaction.
  const EINSUFFICIENT_FUNDS: u64 = 1;

  /// Error code indicating that the specified loan could not be found.
  const ELOAN_NOT_FOUND: u64 = 2;

  /// Error code indicating that the loan has already been repaid.
  const ELOAN_ALREADY_REPAID: u64 = 3;

  /// Error code indicating that the loan is overdue.
  const ELOAN_OVERDUE: u64 = 4;

  /// Error code indicating that the yearly limit for loans has been exceeded.
  const ECREDIT_LIMIT_EXCEEDED: u64 = 5;

  /// The new request would exceed the credit limit
  const ENEW_BALANCE_WOULD_EXCEED_CREDIT_LIMIT: u64 = 6;

  /// Error code indicating that the operation is not authorized.
  const EUNAUTHORIZED: u64 = 7;

  /// Trying to transfer a zero amount
  const EAMOUNT_IS_ZERO: u64 = 8;

  /// State on the community wallet account,
  /// representing loaned unlocked coins
  /// This is just a tracker,
  /// coins themselves are kept in the default CoinStore struct
  /// so that balances can easily be calculated
  struct AdvanceFunds has key {
    coins_available: u64
  }

  /// For Global state, so we can easily query
  /// all CW advance loans
  struct EndowmentAdvanceRegistry has key {
    list: vector<AdvanceLoan>,
  }
  /// unit of loan
  struct AdvanceLoan has key, store {
    id: GUID,
    cw: address,
    amount: u64,
    due_date: u64,
    repaid: bool,
  }

  /// Initialize global state for framework at genesis
  public fun initialize(framework_sig: &signer) {
    if (!exists<EndowmentAdvanceRegistry>(@diem_framework)) {
      move_to<EndowmentAdvanceRegistry>(framework_sig, EndowmentAdvanceRegistry{
        list: vector::empty<AdvanceLoan>()
      });
    }
  }

  /// find the list of loans taken by an address, and return the vector of GUID for each
  public fun find_loans_by_address(dv_account: address): vector<ID> acquires EndowmentAdvanceRegistry {
    let list = &borrow_global<EndowmentAdvanceRegistry>(@diem_framework).list;
    let ids = vector::empty<ID>();

    let i = 0;
    while (i < vector::length(list)) {
      let el = vector::borrow(list, i);
      if (el.cw == dv_account) {
        vector::push_back(&mut ids, guid::id(&el.id));
      };
      i = i + 1;
    };

    if (vector::is_empty(&ids)) {
      error::invalid_argument(ELOAN_NOT_FOUND);
    };
    ids
  }

  // /// Find the loans taken by an address
  // public fun borrow_mut_loans_by_address(dv_account: address): &mut vector<AdvanceLoan> {
  //   let registry = borrow_global_mut<EndowmentAdvanceRegistry>(@diem_framework);
  //   let loans = vector::empty<AdvanceLoan>();


  //   vector::filter(&mut registry.list, |loan| {
  //     if (loan.cw == dv_account) {
  //     vector::push_back(&mut loans, loan);
  //     }
  //   });
  //   if (vector::is_empty(&loans)) {
  //     error::invalid_argument(E_LOAN_NOT_FOUND);
  //   };
  //   loans
  // }

  /// Find idx of loan by GUID
  fun loan_idx_by_guid(id: ID): u64 acquires EndowmentAdvanceRegistry {
    let list = &mut borrow_global_mut<EndowmentAdvanceRegistry>(@diem_framework).list;

    let i = 0;
    while (i < vector::length(list)) {
      let el = vector::borrow_mut(list, i);
      if (guid::id(&el.id) == id) {
        return i
      };
      i = i + 1;
    };
    // nothing found
    assert!(false, error::invalid_argument(ELOAN_NOT_FOUND));
    i // noop
  }


  /// Withdraw funds from advance funds
  public fun withdraw_funds(cap: &WithdrawCapability, amount: u64): Coin<LibraCoin> acquires AdvanceFunds {
    assert!(amount> 0, error::invalid_argument(EAMOUNT_IS_ZERO));
    let payer = account::get_withdraw_cap_address(cap);
    let advance_funds = borrow_global_mut<AdvanceFunds>(payer);
    let limit = advance_funds.coins_available;
    if (amount > limit) {
      error::invalid_argument(EINSUFFICIENT_FUNDS);
    };
    advance_funds.coins_available = limit - amount;
    ol_account::withdraw_with_capability(cap, amount)
  }


  // /// Initialize the loan feature for a community wallet
  // public fun initialize(account: &signer) {
  //   // Implementation goes here
  // }

  /// Management requests loan from actively managed community wallet
  public fun request_advance(cap: &WithdrawCapability, amount: u64) acquires EndowmentAdvanceRegistry, AdvanceFunds {
    let account_address = account::get_withdraw_cap_address(cap);
    let (_, total_balance) = ol_account::balance(account_address);

    let total_loaned = total_outstanding_balance(account_address);

    let credit_limit = total_balance / 100; // 1% of the balance

    if (total_loaned > credit_limit) {
      error::invalid_argument(ECREDIT_LIMIT_EXCEEDED);
    };

    if (total_loaned + amount > credit_limit) {
      error::invalid_argument(ENEW_BALANCE_WOULD_EXCEED_CREDIT_LIMIT);
    };

    let advance_funds = borrow_global_mut<AdvanceFunds>(account_address);

    advance_funds.coins_available = advance_funds.coins_available + amount;

  }

  // public fun send_from_unlocked(account: &signer, amount: u64) {
  //   // Implementation goes here
  // }

  // /// Repay a loan to the community wallet
  // public fun repay_loan(cap: &account::GUIDCapability, coin: Coin<LibraCoin>) {
  //   // Implementation goes here
  // }

  // #[view]
  // /// Check if a loan is overdue
  // public fun is_loan_overdue(dv_account: address): bool {
  //   // Implementation goes here
  //   false
  // }

  // /// Disable the community wallet if the loan is overdue
  // fun deauthorize(cap: &account::GUIDCapability) {
  //   // TODO: link to donor_voice_reauth.move when merged
  //   // Implementation goes here
  // }

  // /// Service the loan with new donations
  // public fun maybe_service_loan_with_donations(account: &signer, donation_amount: u64) {
  //   // Implementation goes here
  // }

  /// Total borrowed
  public fun total_borrowed(account: address): u64 acquires EndowmentAdvanceRegistry {
    let loans = find_loans_by_address(account);
    let total = 0;
    let i = 0;
    while (i < vector::length(&loans)) {
      let loan_id = vector::borrow(&loans, i);
      let loan_idx = loan_idx_by_guid(*loan_id);
      let loan = vector::borrow(&borrow_global<EndowmentAdvanceRegistry>(@diem_framework).list, loan_idx);
      total = total + loan.amount;
      i = i + 1;
    };
    total
  }

  /// Calculate the total outstanding loans compared to the balance in AdvanceFunds struct
  public fun total_outstanding_balance(account: address): u64 acquires EndowmentAdvanceRegistry, AdvanceFunds {
    let advance_funds = borrow_global<AdvanceFunds>(account);
    let total_borrowed = total_borrowed(account);
    advance_funds.coins_available - total_borrowed
  }
}
