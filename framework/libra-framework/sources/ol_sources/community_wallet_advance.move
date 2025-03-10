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
  use diem_framework::account::{Self, GUIDCapability};
  use diem_framework::coin::{Coin};
  use ol_framework::ol_account;
  use ol_framework::libra_coin::{Self, LibraCoin};

  friend ol_framework::donor_voice_txs;

  #[test_only]
  friend ol_framework::test_community_wallet;
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

  /// Trying to overpay the balance outstanding
  const EOVERPAYING: u64 = 9;

  /// math error when trying to update credit score
  const ELOG_MATH_ERR: u64 = 10;

  //////// CONSTANTS ////////
  /// How many basis points of the CW account balance
  /// is available to extend credit.
  /// 1000
  const BPS_BALANCE_CREDIT_LINE: u64 = 50; // 0.50%

  /// minimum yearly repayment of the outstanding balance
  const BPS_MINIMUM_PAY_OUTSTANDING: u64 = 500; // 5%

  /// State on the community wallet account,
  /// representing loaned unlocked coins
  /// This is just a tracker,
  /// coins themselves are kept in the default CoinStore struct

  struct CreditScore has key {
    // current balance outstanding
    balance_outstanding: u64,
    // last withdrawal amount
    last_withdrawal: u64,
    // credit utilization
    lifetime_withdrawals: u64,
    // timestamp of last usage
    last_withdrawal_usecs: u64,
    // last amount deposited
    last_deposit_amount: u64,
    // timestamp of deposit
    last_deposit_usecs: u64,
    // serviced the loan
    lifetime_deposits: u64,
  }


  /// Initialize the loan feature for a community wallet
  public fun initialize(dv_account: &signer) {
    if (!exists<CreditScore>(signer::address_of(dv_account))) {
      move_to<CreditScore>(dv_account, CreditScore{
        balance_outstanding: 0,
        last_withdrawal: 0,
        last_withdrawal_usecs: 0,
        lifetime_withdrawals: 0,
        last_deposit_amount: 0,
        last_deposit_usecs: timestamp::now_seconds(), // don't assume delinquent if not initialized
        lifetime_deposits: 0,
      });
    }
  }

  /// check if amount withdrawn will be below credit limit
  fun can_withdraw_amount(dv_account: address, amount: u64):bool acquires CreditScore {
    assert!(amount> 0, error::invalid_argument(EAMOUNT_IS_ZERO));
    assert!(!is_delinquent(dv_account), error::invalid_state(ELOAN_OVERDUE));
    let available = total_credit_available(dv_account);
    available > amount
  }
  /// Only can be called on epoch boundary as part of the donor_voice_txs.move authorization flow.
  /// Will withdraw funds and track the logger
  public(friend) fun transfer_credit(framework_sig: &signer, guid_cap: &GUIDCapability, recipient: address, amount: u64): u64 acquires CreditScore {
    let dv_account = account::get_guid_capability_address(guid_cap);
    can_withdraw_amount(dv_account, amount);
    log_withdrawal(dv_account, amount);
    ol_account::vm_transfer(framework_sig, dv_account, recipient, amount);
    // TODO: check if there is any possibility of partial amount sent
    amount
  }

  /// Service the loan with new coins
  /// If the balance outstanding is lower than the coin value,
  /// then only log the amount which is to be debited.
  /// This amount only affects logging, all the coins are sent anyways to the CoinStore
  public fun service_loan_with_coin(dv_address: address, coins: Coin<LibraCoin>) acquires CreditScore {
    let state = borrow_global_mut<CreditScore>(dv_address);
    let coin_value = libra_coin::value(&coins);
    let deposit_amount = coin_value;
    if (coin_value > state.balance_outstanding) {
      deposit_amount = state.balance_outstanding;
    };
    log_deposit(dv_address, deposit_amount);

    ol_account::deposit_coins(dv_address, coins);
  }

  fun log_withdrawal(dv_account: address, amount: u64) acquires CreditScore {
    let cs_state = borrow_global_mut<CreditScore>(dv_account);
    cs_state.last_withdrawal = amount;
    cs_state.last_withdrawal_usecs = timestamp::now_seconds();
    cs_state.balance_outstanding = cs_state.balance_outstanding + amount;
    cs_state.lifetime_withdrawals = cs_state.lifetime_withdrawals + amount;
    // TODO: get a specific error for this check
    // shouldn't be in a situation where deposits are greater than withdrawals
    assert!(cs_state.lifetime_withdrawals >= cs_state.lifetime_deposits, error::invalid_state(ELOG_MATH_ERR));
    // the lifetime withdrawn should be equal or greater to the current balance outstanding
    assert!(cs_state.lifetime_withdrawals >= cs_state.balance_outstanding, error::invalid_state(ELOG_MATH_ERR));
  }

  fun log_deposit(dv_account: address, amount: u64) acquires CreditScore {
    assert!(amount> 0, error::invalid_argument(EAMOUNT_IS_ZERO));

    let cs_state = borrow_global_mut<CreditScore>(dv_account);
    cs_state.last_deposit_usecs = timestamp::now_seconds();
    assert!(cs_state.balance_outstanding > amount, error::invalid_argument(EOVERPAYING));
    cs_state.balance_outstanding = cs_state.balance_outstanding - amount;
    cs_state.lifetime_deposits = cs_state.lifetime_deposits + amount;

    // shouldn't be in a situation where deposits are greater than withdrawals
    assert!(cs_state.lifetime_withdrawals >= cs_state.lifetime_deposits, error::invalid_state(ELOG_MATH_ERR));
  }

  #[view]
  /// Has the CW account made a payment in the last year
  // TODO: need to check amount of payment history in the last year, but we're not tracking individual payments.
  public fun is_delinquent(dv_account: address): bool acquires CreditScore {
    let cs_state = borrow_global<CreditScore>(dv_account);
    // never withdrawn
    if (cs_state.lifetime_withdrawals == 0 ||
      cs_state.last_withdrawal == 0
    ){ return false };

    let current_time = timestamp::now_seconds();

    let year_after_last_deposit = cs_state.last_deposit_usecs + 31536000;
    // still within one year window
    current_time < year_after_last_deposit
  }


  /// Disable the community wallet if the loan is overdue
  // callable by anyone
  public entry fun maybe_deauthorize(dv_account: address) acquires CreditScore {
    if (is_delinquent(dv_account)){
      // TODO: call donor_voice_reauthorize when it is merged
    }
  }


  #[view]
  /// Calculate the total outstanding loans compared to the balance in Credit struct
  public fun total_outstanding_balance(account: address): u64 acquires CreditScore{
    let cs_state = borrow_global<CreditScore>(account);
    cs_state.balance_outstanding
  }

  #[view]
  /// Checks if the current outstanding balance is below credit limit
  public fun total_credit_available(dv_account: address): u64 acquires CreditScore {
    let cs = borrow_global_mut<CreditScore>(dv_account);
    let usage = cs.balance_outstanding;

    let (_, total_balance) = ol_account::balance(dv_account);
    if (usage > total_balance) {
      return 0
    };

    let limit = (total_balance * BPS_BALANCE_CREDIT_LINE) / 10000;

    if (usage > limit ) {
      return 0
    };

    limit - usage
  }

}
