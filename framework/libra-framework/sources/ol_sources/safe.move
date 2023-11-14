///////////////////////////////////////////////////////////////////////////
// 0L Module
// MultiSig
// A payment tool for transfers which require n-of-m approvals
///////////////////////////////////////////////////////////////////////////


// The main design goals of this multisig implementation are:
// 0 . Leverages MultiSig library which allows for arbitrary transaction types to be handled by the multisig. This is a payments implementation.
// 1. should leverage the usual transaction flow and tools which users are familiar with to add funds to the account. The funds remain viewable by the usual tools for viewing account balances.
// 2. The authority over the address should not require collecting signatures offline: transactions should be submitted directly to the contract.
// 3. Funds are disbursed as usual: to a destination addresses, and not into any intermediate structures.
// 4. Does not pool funds into a custodian contract (like gnosis-type implementations)
// 5. Uses the shared security of the root address, and as such charge a fee for this benefit.

// Custody
// This multisig implementation does not custody funds in a central address (the MultiSig smart contract address does not pool funds).

// The multisig funds exist on a remote address, the address of the creator.
// This implies some safety features which need to be implemented to prevent the creator from having undue influence over the multisig after it has been created.

// No Segregation
// Once the account is created, and intantiated as a multisig, the funds remain in the ordinary data structure for coins. There is no intermediary data structure which holds the coins, neither on the account address, nor in the smart contract address.
// This means that all existing tools for sending transfers to this account will execute as usual. All tools and APIs which read balances will work as usual.

// 0L MultiSig module
// Third party multisig apps are possible, but either they will use a custodial model, use segrated structures on a sender account (where the signer may always have authority), or they will require the user to collect signatures offline.
// A third party multisig app could achieve the design goals above also by Leveraging the MultiSig contract. Achieving it requires tight coupling to the DiemAccount tools, and VM authority.

// Root Security
// This contract can simply be cloned, and a third party may offer a multisig service. Though we expect people to use the service with the highest level of guarantees, and least amount of effor to use.
// Using MultiSigPayment means leveraging "Root Security".
// This service has the highest level of security in the system, a shared Root Security. The account which creates this multisig (sponsor), immediately has their authorization key "bricked" such that it cannot issue any type of transaction to the account. The authorities are the only one that can issue transactions. Since this code is published to the 0x0 address, it cannot be changed, unles by protocol upgrade, thus it has the highest level of security.

// Fees
// Since this contract offers Root Security, this is a benefit provided collectively, and as such there is a fee for this service.

// The fee is paid to the root address, and is used to pay for the security from consensus (validator rewards). The fee is a percentage of the funds added to the multisig.

// Authorities
// What changes from a vanilla 0L Address that the "signer" for the account loses access to that account. And instead the funds are controlled by the Multisig logic. The implementation of this is that the account's AuthKey is rotated to a random number, and the signer for the account is removed, forcing the signer to lose control. As such the sender needs to THINK CAREFULLY about the initial set of authorities on this address.

module ol_framework::safe {
  use std::vector;
  use std::option::{Self, Option};
  use std::fixed_point32;
  use std::signer;
  use std::guid;
  use std::error;
  use diem_framework::account::WithdrawCapability;
  use diem_framework::coin;
  use ol_framework::ol_account;
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::multi_action;
  use ol_framework::system_addresses;
  use ol_framework::transaction_fee;

  // use diem_std::debug::print;

  friend ol_framework::epoch_boundary;

  /// a multi action safe is not initialized on this account
  const ESAFE_NOT_INITIALIZED: u64 = 1;

  /// Genesis starting fee for multisig service
  const STARTING_FEE: u64 = 00000027; // 1% per year, 0.0027% per epoch
  const PERCENT_SCALE: u64 = 1000000; // for 4 decimal precision percentages


  /// This is the data structure which is stored in the Action for the multisig.
  struct PaymentType has key, store, copy, drop {
    // The transaction to be executed
    destination: address,
    // amount
    amount: u64,
    // note
    note: vector<u8>,
  }

  /// This fucntion initiates governance for the multisig. It is called by the sponsor address, and is only callable once.
  /// init_gov fails gracefully if the governance is already initialized.
  /// init_type will throw errors if the type is already initialized.

  public fun init_payment_multisig(sponsor: &signer, init_signers: vector<address>, cfg_n_signers: u64) acquires RootMultiSigRegistry {
    multi_action::init_gov(sponsor, cfg_n_signers, &init_signers);
    multi_action::init_type<PaymentType>(sponsor, true);
    add_to_registry(signer::address_of(sponsor));
  }


  // Propose a transaction
  // Transactions should be easy, and have one obvious way to do it. There should be no other method for voting for a tx.
  // this function will catch a duplicate, and vote in its favor.
  // This causes a user interface issue, users need to know that you cannot have two open proposals for the same transaction.
  // It's optional to state how many epochs from today the transaction should expire. If the transaction is not approved by then, it will be rejected.
  // The default will be 14 days.
  // Only the first proposer can set the expiration time. It will be ignored when a duplicate is caught.


  public fun propose_payment(sig: &signer, multisig_addr: address, recipient: address, amount: u64, note: vector<u8>, duration_epochs: Option<u64>): guid::ID acquires RootMultiSigRegistry {
    assert!(is_in_registry(multisig_addr), error::invalid_state(ESAFE_NOT_INITIALIZED));
    let pay = new_payment(recipient, amount, *&note);
    let prop = multi_action::proposal_constructor(pay, duration_epochs);
    let id = multi_action::propose_new<PaymentType>(sig, multisig_addr, prop);
    vote_payment(sig, multisig_addr, &id);
    id
  }

  /// create a payment object, whcih can be send in a proposal.
  public fun new_payment(destination: address, amount: u64, note: vector<u8>): PaymentType {
    PaymentType {
      destination,
      amount,
      note,
    }
  }

  public fun vote_payment(sig: &signer, multisig_address: address, id: &guid::ID): bool {

    let (passed, cap_opt) = multi_action::vote_with_id<PaymentType>(sig, id, multisig_address);

    if (passed && option::is_some(&cap_opt)) {
      let cap = option::borrow(&cap_opt);
      let data = multi_action::extract_proposal_data(multisig_address, id);
      release_payment(&data, cap);

    };
    multi_action::maybe_restore_withdraw_cap(cap_opt); // don't need this and can't drop.

    passed
  }

  public fun is_payment_multisig(addr: address):bool {
    multi_action::has_action<PaymentType>(addr)
  }



  // Sending payment. Ordinarily an account can only transfer funds if the signer of that account is sending the transaction.
  // In Libra we have "withdrawal capability" tokens, which allow the holder of that token to authorize transactions. At the initilization of the multisig, the "withdrawal capability" was passed into the MultiSig datastructure.
  // Withdrawal capabilities are "hot potato" data. Meaning, they cannot ever be dropped and need to be moved to a final resting place, or returned to the struct that was housing it. That is what happens at the end of release_payment, it is only borrowed, and never leaves the data structure.

  fun release_payment(p: &PaymentType, cap: &WithdrawCapability) {
    let c = ol_account::withdraw_with_capability(
      cap,
      p.amount,
    );
    ol_account::deposit_coins(p.destination, c);
  }

  //////// ROOT SERVICE FEE BILLING ////////

  struct RootMultiSigRegistry has key {
    list: vector<address>,
    fee: u64, // percentage balance fee denomiated in 4 decimal precision 123456 = 12.3456%
  }

  public fun initialize(vm: &signer) {
   system_addresses::assert_ol(vm);
   if (!exists<RootMultiSigRegistry>(@ol_framework)) {
     move_to<RootMultiSigRegistry>(vm, RootMultiSigRegistry {
       list: vector::empty(),
       fee: STARTING_FEE,
     });
   };
  }

  fun add_to_registry(addr: address) acquires RootMultiSigRegistry {
    let reg = borrow_global_mut<RootMultiSigRegistry>(@ol_framework);
    if (!vector::contains(&reg.list, &addr)) {
      vector::push_back(&mut reg.list, addr);
    };
  }

  fun is_in_registry(addr: address):bool acquires RootMultiSigRegistry {
    let reg = borrow_global_mut<RootMultiSigRegistry>(@ol_framework);
    vector::contains(&reg.list, &addr)
  }


  /// charges the fee for shared security
  /// returns (security_bill_count, security_bill_amount, success) so that epoch_boundary can
  /// audit the result
  public(friend) fun root_security_fee_billing(vm: &signer): (u64, u64, bool) acquires RootMultiSigRegistry {
    system_addresses::assert_ol(vm);
    let reg = borrow_global<RootMultiSigRegistry>(@ol_framework);
    let security_bill_count = 0;
    let security_bill_amount = 0;
    let expected_fees = 0;
    let i = 0;
    while (i < vector::length(&reg.list)) {

      let multi_sig_addr = vector::borrow(&reg.list, i);

      let pct = fixed_point32::create_from_rational(reg.fee, PERCENT_SCALE);

      let fee = fixed_point32::multiply_u64(coin::balance<LibraCoin>(*multi_sig_addr), pct);
      expected_fees = expected_fees + fee;

      let coin_opt = ol_account::vm_withdraw_unlimited(vm, *multi_sig_addr, fee);
      if (option::is_some(&coin_opt)) {
        let c = option::extract(&mut coin_opt);
        security_bill_amount = security_bill_amount + coin::value(&c);
        security_bill_count = security_bill_count + 1;
        transaction_fee::vm_pay_fee(vm, *multi_sig_addr, c);
      };
      option::destroy_none(coin_opt);

      i = i + 1;
    };
    let success = security_bill_count == 1 && security_bill_amount == expected_fees;
    (security_bill_count, security_bill_amount, success)
  }
}
