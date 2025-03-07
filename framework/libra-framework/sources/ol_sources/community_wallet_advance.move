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
  use std::signer;
  use std::timestamp;
  use std::vector;
  use std::guid::{Self, GUID, ID};
  use diem_framework::account::{Self, GUIDCapability, WithdrawCapability};
  use diem_framework::coin::{Self, Coin};
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
  struct Credit has key {
    credit_available: u64,
    payment_history: vector<Payment>
  }

  struct Payment has key, store {
    timestamp_usecs: u64,
    coins: u64,
  }

  /// keep all loans in global state, so we can easily query
  struct EndowmentAdvanceRegistry has key {
    list: vector<LetterOfCredit>,
  }
  /// unit of loan
  struct LetterOfCredit has key, store {
    id: GUID,
    cw: address,
    amount: u64,
    due_timestamp_usecs: u64,
    repaid: bool,
  }

  /// Initialize global state for framework at genesis
  public fun initialize(framework_sig: &signer) {
    if (!exists<EndowmentAdvanceRegistry>(@diem_framework)) {
      move_to<EndowmentAdvanceRegistry>(framework_sig, EndowmentAdvanceRegistry{
        list: vector::empty<LetterOfCredit>()
      });
    }
  }

  /// Initialize the loan feature for a community wallet
  public fun init_user(dv_account: &signer) {
    if (!exists<Credit>(signer::address_of(dv_account))) {
      move_to<Credit>(dv_account, Credit{
        credit_available: 0,
        payment_history: vector::empty(),
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
  // TODO: unclear if this is needed for programmatic sending
  fun withdraw_funds(cap: &WithdrawCapability, amount: u64): Coin<LibraCoin> acquires Credit {
    assert!(amount> 0, error::invalid_argument(EAMOUNT_IS_ZERO));
    let payer = account::get_withdraw_cap_address(cap);
    let advance_funds = borrow_global_mut<Credit>(payer);
    let limit = advance_funds.credit_available;
    if (amount > limit) {
      error::invalid_argument(EINSUFFICIENT_FUNDS);
    };
    advance_funds.credit_available = limit - amount;
    ol_account::withdraw_with_capability(cap, amount)
  }

  public fun transfer_from_advance(cap: &WithdrawCapability, recipient: address, amount: u64) acquires Credit {
    assert!(amount> 0, error::invalid_argument(EAMOUNT_IS_ZERO));
    let payer = account::get_withdraw_cap_address(cap);
    let advance_funds = borrow_global_mut<Credit>(payer);
    let limit = advance_funds.credit_available;
    if (amount > limit) {
      error::invalid_argument(EINSUFFICIENT_FUNDS);
    };
    advance_funds.credit_available = limit - amount;
    ol_account::transfer_with_capability(cap, recipient, amount);
  }

  /// Management requests loan from actively managed community wallet
  /// belt and suspenders requires both capabilities.
  public fun request_advance(cap: &WithdrawCapability, guid_capability: &GUIDCapability, amount: u64) acquires EndowmentAdvanceRegistry, Credit {
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

    let advance_funds = borrow_global_mut<Credit>(account_address);

    advance_funds.credit_available = advance_funds.credit_available + amount;

    let loan_doc = LetterOfCredit {
      id: account::create_guid_with_capability(guid_capability),
      cw: account_address,
      amount: amount,
      due_timestamp_usecs: timestamp::now_seconds() + 31536000, // 1 year from now
      repaid: false
    };

    let state = borrow_global_mut<EndowmentAdvanceRegistry>(@diem_framework);
    vector::push_back(&mut state.list, loan_doc);
  }


  #[view]
  /// Check if any loan is overdue
  public fun is_loan_overdue(dv_account: address): bool acquires EndowmentAdvanceRegistry {
    let list = &borrow_global<EndowmentAdvanceRegistry>(@diem_framework).list;
    let current_time = timestamp::now_seconds();

    let i = 0;
    while (i < vector::length(list)) {
      let el = vector::borrow(list, i);
      if (el.cw == dv_account && !el.repaid && el.due_timestamp_usecs < current_time) {
        return true
      };
      i = i + 1;
    };
    false
  }

  /// Disable the community wallet if the loan is overdue
  // callable by anyone
  public entry fun maybe_deauthorize(dv_account: address) acquires EndowmentAdvanceRegistry {
    if (is_loan_overdue(dv_account)){
      // TODO: call donor_voice_reauthorize when it is merged
    }
  }

  /// Service the loan with new coins
  public fun service_loan_with_coin(dv_address: address, coins: Coin<LibraCoin>) acquires Credit {
    let state = borrow_global_mut<Credit>(dv_address);
    let payment_receipt = Payment {
      timestamp_usecs: timestamp::now_seconds(),
      coins: coin::value(&coins),
    };
    vector::push_back(&mut state.payment_history, payment_receipt);
    ol_account::deposit_coins(dv_address, coins);
  }

  /// finds the oldest unpaid letter of credit
  /// returns the id, and the balance amount
  fun find_oldest_loan_id(dv_account: address): (ID, u64) acquires EndowmentAdvanceRegistry {
    let ids = find_loans_by_address(dv_account);
    let oldest_loan_id = vector::borrow(&ids, 0);
    let oldest_loan_idx = loan_idx_by_guid(*oldest_loan_id);
    let oldest_loan_amount = 0;

    let oldest_due_timestamp_usecs = { // new scope to drop the borrow
      let oldest_loan = vector::borrow(&borrow_global<EndowmentAdvanceRegistry>(@diem_framework).list, oldest_loan_idx);
      oldest_loan.due_timestamp_usecs
    };

    let i = 1;
    while (i < vector::length(&ids)) {
      let loan_id = vector::borrow(&ids, i);
      let loan_idx = loan_idx_by_guid(*loan_id);
      let loan = vector::borrow(&borrow_global<EndowmentAdvanceRegistry>(@diem_framework).list, loan_idx);
      if (loan.due_timestamp_usecs < oldest_due_timestamp_usecs) {
      oldest_loan_id = loan_id;
      oldest_due_timestamp_usecs = loan.due_timestamp_usecs;
      oldest_loan_amount = loan.amount;
      };
      i = i + 1;
    };

    (*oldest_loan_id, oldest_loan_amount)
  }

  #[view]
  /// Total borrowed by account
  public fun total_credit_requested(account: address): u64 acquires EndowmentAdvanceRegistry {
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

  #[view]
  /// Calculate the total outstanding loans compared to the balance in Credit struct
  public fun total_outstanding_balance(account: address): u64 acquires EndowmentAdvanceRegistry, Credit {
    let advance_funds = borrow_global<Credit>(account);
    let total_credit_requested = total_credit_requested(account);
    advance_funds.credit_available - total_credit_requested
  }
}
