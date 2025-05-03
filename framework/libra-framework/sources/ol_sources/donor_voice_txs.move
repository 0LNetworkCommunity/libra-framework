/// Donor Voice wallets is a service of the chain.
/// Any address can voluntarily turn their account into a Donor Voice account.

/// Definitions
/// Unless otherwise specified the assumption of the Donor Voice app is
/// that there is an Owner of the account and property.
/// The Owner can assign Proxy Authorities, who acting as custodians can issue
/// transactions on behalf of Owner
/// Depositors are called Donors, and the app gives depositors
/// visibility of the transactions, and also limited authority over
/// specific actions: alerting the Owner and Depositors from
/// unauthorized transaction.

/// The DonorVoice Account Lifecycle:
/// Proxy Authorities use a MultiSig to schedule ->
/// Once scheduled the Donors use a TurnoutTally to Veto ->
/// Epoch boundary: transaction executes when the VM reads the Schedule struct at the epoch boundary, and issues payment.

/// By creating a TxSchedule wallet you are providing certain restrictions and guarantees to the users that interact with this wallet.

/// 1. The wallet's contents is property of the owner. The owner is free to issue transactions which change the state of the wallet, including transferring funds. There are however time, and veto policies.

/// 2. All transfers out of the account are timed. Meaning, they will execute automatically after a set period of time passes. The VM address triggers these events at each epoch boundary. The purpose of the delayed transfers is that the transaction can be paused for analysis, and eventually rejected by the donors of the wallet.

/// 3. Every pending transaction can be "vetoed". Each veto delays the finalizing of the transaction, to allow more time for analysis. Each veto adds one day/epoch to the transaction PER DAY THAT A VETO OCCURS. That is, two vetoes happening in the same day, only extend the vote by one day. If a sufficient number of Donors vote on the Veto, then the transaction will be rejected. Since TxSchedule has an expiration time, as does ParticipationVote, each time there is a veto, the deadlines for both are synchronized, based on the new TxSchedule expiration time.

/// 4. After three consecutive transaction rejections, the account will become frozen. The funds remain in the account but no operations are available until the Donors, un-freeze the account.

/// 5. Voting for all purposes are done on a pro-rata basis according to the amounts donated. Voting using ParticipationVote method, which in short, biases the threshold based on the turnout of the vote. TL;DR a low turnout of 12.5% would require 100% of the voters to veto, and lower thresholds for higher turnouts until 51%.

/// 6. The donors can vote to liquidate a frozen TxSchedule account. The result will depend on the configuration of the TxSchedule account from when it was initialized: the funds by default return to the end user who was the donor.

/// 7. Third party contracts can wrap the Donor Voice wallet. The outcomes of the votes can be returned to a handler in a third party contract For example, liquidation of a frozen account is programmable: a handler can be coded to determine the outcome of the Donor Voice wallet. See in CommunityWallets the funds return to the InfrastructureEscrow side-account of the user.

module ol_framework::donor_voice_txs {
    use std::vector;
    use std::signer;
    use std::error;
    use std::guid;
    use std::fixed_point32;
    use std::option::{Self, Option};
    use diem_framework::system_addresses;
    use diem_framework::coin;
    use ol_framework::ballot;
    use ol_framework::community_wallet_advance;
    use ol_framework::epoch_helper;
    use ol_framework::ol_account;
    use ol_framework::ol_features_constants;
    use ol_framework::receipts;
    use ol_framework::multi_action;
    use ol_framework::account::{Self, WithdrawCapability};
    use ol_framework::donor_voice_governance;
    use ol_framework::donor_voice_reauth;
    use ol_framework::cumulative_deposits;
    use ol_framework::reauthorization;
    use ol_framework::transaction_fee;
    use ol_framework::match_index;
    use ol_framework::donor_voice;
    use ol_framework::slow_wallet;

    friend ol_framework::community_wallet_init;
    friend ol_framework::epoch_boundary;

    #[test_only]
    friend ol_framework::test_donor_voice;
    #[test_only]
    friend ol_framework::test_community_wallet;

    /// Not initialized as a Donor Voice account.
    const ENOT_INIT_DONOR_VOICE: u64 = 1;
    /// User is not a donor and cannot vote on this account
    const ENOT_AUTHORIZED_TO_VOTE: u64 = 2;
    /// Could not find a pending transaction by this GUID
    const ENO_PENDING_TRANSACTION_AT_UID: u64 = 3;
    /// Unknown state enum used
    const ENOT_VALID_STATE_ENUM: u64 = 4;
    /// Multisig not initialized
    const EMULTISIG_NOT_INIT: u64 = 5;
    /// No enum for this number
    const ENO_VETO_ID_FOUND: u64 = 6;
    /// No enum for this number
    const EPAYEE_NOT_SLOW: u64 = 7;
    /// Governance mode: chain has restricted Donor Voice transactions while upgrades are executed.
    const EGOVERNANCE_MODE: u64 = 8;
    /// Can't initialize accounts with high balance
    const ECANT_INIT_WITH_HIGH_BALANCE: u64 = 9;
    /// This is a slow wallet, probably meant to send to unrestricted account
    const EUNAUTHORIZED_SLOW_WALLET: u64 = 10;
    /// Unlocked coin transfers exceed the limit
    const EUNLOCKED_COIN_LIMIT_EXCEEDED: u64 = 11;

    /// number of epochs to wait before a transaction is executed
    /// Veto can happen in this time
    /// at the end of the third epoch from when multisig gets consensus
    const DEFAULT_PAYMENT_DURATION: u64 = 3;
    /// minimum amount of time to evaluate when one donor flags for veto.
    const DEFAULT_VETO_DURATION: u64 = 7;
    /// liquidation vote can take a whole year
    const LIQUIDATION_TALLY_DAYS: u64 = 365;
    /// maximum starting balance for a donor voice account
    // one million coins with scaling factor
    const MAXIMUM_STARTING_BALANCE: u64 = 1_000_000 * 1_000_000;


    // Timed transfer schedule pipeline
    struct TxSchedule has key {
      scheduled: vector<TimedTransfer>,
      veto: vector<TimedTransfer>,
      paid: vector<TimedTransfer>,
      guid_capability: account::GUIDCapability, // we need this for the MultiSig
    }

    /// This is the basic payment information.
    /// This is used initially in a MultiSig, for the Authorities
    /// initially to schedule.
    struct Payment has copy, drop, store {
      payee: address,
      value: u64,
      description: vector<u8>,
    }


    struct TimedTransfer has drop, store {
      uid: guid::ID, // copy of ID generated by MultiSig for the transaction
      deadline: u64, // what epoch does the transaction execute
      tx: Payment, // The transaction properties
      epoch_latest_veto_received: u64, // This is to check if we need to extend the deadline
    }

    // account's freeze policy which donor's have a "voice" in
    struct Freeze has key {
      is_frozen: bool,
      consecutive_rejections: u64,
      unfreeze_votes: vector<address>,
      liquidate_to_match_index: bool,
    }

    // can only be called by genesis
    public(friend) fun migrate_community_wallet_account(framework: &signer, dv_account:
    &signer) {
      system_addresses::assert_ol(framework);
      let liquidate_to_match_index = true;
      // skip setting up the multisig
      structs_init(dv_account, liquidate_to_match_index);
      donor_voice::add(dv_account);
    }

    //////// DONOR VOICE INITIALIZATION ////////
    // There are three steps in initializing an account. These steps can be combined in a single transaction, or done in separate transactions. The "bricking" of the sponsor key should be done in a separate transaction, in case there are any errors in the initialization.

    // 1. The basic structs for a Donor Voice account need to be initialized, and the account needs to be added to the Registry at root.

    // 2. A MultiSig action structs need to be initialized.

    // 3. Once the MultiSig is initialized, the account needs to be caged, before the MultiSig can be used.

    public(friend) fun make_donor_voice(sponsor: &signer)  {
      assert!(!ol_features_constants::is_governance_mode_enabled(), error::invalid_state(EGOVERNANCE_MODE));

      // check users are not bypassing the human verification
      // for level 8
      let addr = signer::address_of(sponsor);
      reauthorization::assert_v8_authorized(addr);

      // check if the account has a meaningful balance
      // donor voice accounts build slowly through decentralized
      // donations, and large initialization removes transaction
      // metadata for donor governance
      let (_, balance) = ol_account::balance(addr);
      assert!(balance < MAXIMUM_STARTING_BALANCE, error::invalid_argument(ECANT_INIT_WITH_HIGH_BALANCE));


      // will not create if already exists (for migration)
      cumulative_deposits::init_cumulative_deposits(sponsor);

      // we are setting liquidation to match_index as false by default
      // the user can send another transaction to change this.
      let liquidate_to_match_index = false;
      structs_init(sponsor, liquidate_to_match_index);
      make_multi_action(sponsor);
      donor_voice::add(sponsor);
    }

    fun structs_init(sig: &signer, liquidate_to_match_index: bool) {
      if (!donor_voice::is_root_init()) return;

      // exit gracefully in migration cases
      // if Freeze exists everything else is likely created
      if (exists<Freeze>(signer::address_of(sig))) return;

      move_to<Freeze>(
        sig,
        Freeze {
          is_frozen: false,
          consecutive_rejections: 0,
          unfreeze_votes: vector::empty<address>(),
          liquidate_to_match_index,
        }
      );

      let guid_capability = account::create_guid_capability(sig);
      move_to(sig, TxSchedule {
          scheduled: vector::empty(),
          veto: vector::empty(),
          paid: vector::empty(),
          guid_capability,
        });

      // Commit note: this should now failover gracefully
      donor_voice_governance::maybe_init_dv_governance(sig);
      community_wallet_advance::initialize(sig);
    }

    /// Like any MultiSig instance, a sponsor which is the original owner of the account, needs to initialize the account.
    /// The account must be "bricked" by the owner before MultiSig actions can be taken.
    /// Note, as with any multisig, the new_authorities cannot include the sponsor, since that account will no longer be able to sign transactions.
    fun make_multi_action(sponsor: &signer) {
      multi_action::init_gov(sponsor);
      multi_action::init_type<Payment>(sponsor, true); // "true": We make this multisig instance hold the WithdrawCapability. Even though we don't need it for any account pay functions, we can use it to make sure the entire pipeline of private functions scheduling a payment are authorized. Belt and suspenders.
    }


    ///////// MULTISIG ACTIONS TO SCHEDULE A TIMED TRANSFER /////////
    /// As in any MultiSig instance, the transaction which proposes the action (the scheduled transfer) must be signed by an authority on the MultiSig.
    /// The same function is the handler for the approval case of the MultiSig action.
    /// Since Donor Voice accounts are involved with sensitive assets, we have moved the WithdrawCapability to the MultiSig instance. Even though we don't need it for any account functions for paying, we use it to ensure no private functions related to assets can be called. Belt and suspenders.

    /// To request an advance, the payee must be and ORDINARY WALLET
    /// and the limits must not have been exceeded.

    /// Returns the GUID of the transfer.
    fun propose_payment(
      sender: &signer,
      multisig_address: address,
      payee: address,
      value: u64,
      description: vector<u8>,
    ): guid::ID acquires TxSchedule {
      // check the CW has been reauthorized
      donor_voice_reauth::assert_authorized(multisig_address);
      // check the multisig admin is initialized
      reauthorization::assert_v8_authorized(signer::address_of(sender));
      // check the payee is initialized
      reauthorization::assert_v8_authorized(payee);


      assert!(!ol_features_constants::is_governance_mode_enabled(), error::invalid_state(EGOVERNANCE_MODE));


      // Commit note: don't abort on not slow wallet, in cases of unlocked advance
      // we place this check on the calling _tx entry function

      let tx = Payment {
        payee,
        value,
        description,
      };

      // TODO: get expiration
      let prop = multi_action::proposal_constructor(tx, option::none());

      let uid = multi_action::propose_new<Payment>(sender, multisig_address, prop);

      let (passed, withdraw_cap_opt) = multi_action::vote_with_id<Payment>(sender, &uid, multisig_address);

      let tx: Payment = multi_action::extract_proposal_data(multisig_address, &uid);

      if (passed && option::is_some(&withdraw_cap_opt)) {
        schedule(option::borrow(&withdraw_cap_opt), tx, &uid);
      };

      multi_action::maybe_restore_withdraw_cap(withdraw_cap_opt);

      uid

    }

    /// Private function which handles the logic of adding a new timed transfer
    /// DANGER upstream functions need to check the sender is authorized.
    // TODO: perhaps require the WithdrawCapability

    // The TxSchedule wallet signer can propose a timed transaction.
    // the timed transaction defaults to occurring in the 3rd following epoch.
    // TODO: Increase this time?
    // the transaction will automatically occur at the epoch boundary,
    // unless a veto vote by the validator set is successful.
    // at that point the transaction leaves the proposed queue, and is added
    // the rejected list.

    fun schedule(
      withdraw_capability: &WithdrawCapability, tx: Payment, uid: &guid::ID
    ) acquires TxSchedule {
      let multisig_address = account::get_withdraw_cap_address(withdraw_capability);
      let transfers = borrow_global_mut<TxSchedule>(multisig_address);

      let deadline = epoch_helper::get_current_epoch() + DEFAULT_PAYMENT_DURATION;

      let t = TimedTransfer {
        uid: *uid,
        deadline, // pays automatically at the end of seventh epoch. Unless there is a veto by a Donor. In that case a day is added for every day there is a veto. This deduplicates vetoes.
        tx,
        epoch_latest_veto_received: 0,
      };

      vector::push_back<TimedTransfer>(&mut transfers.scheduled, t);
    }

    /// search for a transaction ID in the queues. Returns (is found, index, status enum)
    fun find_schedule_by_id(state: &TxSchedule, uid: &guid::ID): (bool, u64, u8) { // (is_found, index, state)
      let (found, i) = schedule_status(state, uid, scheduled_enum());
      if (found) return (found, i, scheduled_enum());

      let (found, i) = schedule_status(state, uid, veto_enum());
      if (found) return (found, i, veto_enum());

      let (found, i) = schedule_status(state, uid, paid_enum());
      if (found) return (found, i, paid_enum());

      (false, 0, 0)
    }

    ///////// PROCESS PAYMENTS /////////
    /// The VM on epoch boundaries will execute the payments without the users
    /// needing to intervene.
    /// Returns (accounts_processed, amount_processed, success)
    // TODO: add to the return the tuple of sender/recipient that failed
    public(friend) fun process_donor_voice_accounts(
        vm: &signer,
        epoch: u64,
    ): (u64, u64, bool) acquires TxSchedule, Freeze {
      // while we are here let's liquidate any expired accounts.
      vm_liquidate(vm);

      let accounts_processed = 0;
      let amount_processed = 0;
      let expected_amount = 0;

      let list = donor_voice::get_root_registry();

      let i = 0;

      while (i < vector::length(&list)) {
        let multisig_address = vector::borrow(&list, i);
        if (exists<TxSchedule>(*multisig_address)) {
          let state = borrow_global_mut<TxSchedule>(*multisig_address);
          let (processed, expected, _success) = maybe_pay_deadline(vm, state, epoch);
          amount_processed = amount_processed + processed;
          expected_amount = expected_amount + expected;
          accounts_processed = accounts_processed + 1;
        };
        i = i + 1;
      };

      let success = vector::length(&list) == accounts_processed && amount_processed == expected_amount;
      (accounts_processed, amount_processed, success)
    }

    fun filter_scheduled_due(state: &mut TxSchedule, epoch:
    u64): vector<TimedTransfer> {
      // move the future payments to the front of the list,
      // so we can split off the due payments from the second half.
      let list = &mut state.scheduled;
      let split_point = vector::stable_partition<TimedTransfer>(list, |e| {
        let e: &TimedTransfer = e;
        e.deadline > epoch
      });
      vector::trim(&mut state.scheduled, split_point)
    }


    /// Tries to settle any amounts that have been scheduled for payment
    /// for audit instrumentation returns how much was actually transferred
    /// and if that amount was equal to the expected amount transferred (amount_processed, expected_amount, success)
    fun maybe_pay_deadline(vm: &signer, state: &mut TxSchedule, epoch: u64): (u64, u64, bool) acquires Freeze {
      let expected_amount = 0;
      let amount_processed = 0;
      let i = 0;

      let due_list = filter_scheduled_due(state, epoch);

      // find all Txs scheduled prior to this epoch.
      let len = vector::length(&due_list);
      while (i < len) {
        let t: TimedTransfer = vector::pop_back(&mut due_list);
        expected_amount = expected_amount + t.tx.value;
        let multisig_address = guid::id_creator_address(&t.uid);


        // perform the same evaluation as in th _tx. Is the recipient receiving unlocked coins within the policy of the community wallet
        let is_slow = slow_wallet::is_slow(t.tx.payee);


        let this_transfer_value =if (!is_slow) {
          let can_withdraw = community_wallet_advance::can_withdraw_amount(multisig_address, t.tx.value);

          if (can_withdraw) {
              community_wallet_advance::transfer_credit_vm(vm, &state.guid_capability, t.tx.payee, t.tx.value);
              t.tx.value
            } else {
              // should not try to make any payments
              continue
          }
        } else {
          handle_slow_wallet_payment(vm, &t)
        };

        // check success
        if  (this_transfer_value > 0) {
          // if there's a single transaction that gets approved, then the consecutive rejection counter (for freezing the account) is reset
          reset_rejection_counter(vm, multisig_address);

          // update the records (hot potato, can't copy or drop)
          vector::push_back(&mut state.paid, t);

        } else {
          // if it could not be paid because of low balance,
          // place it back on the scheduled list
          vector::push_back(&mut state.scheduled, t);
        };

        amount_processed = amount_processed + this_transfer_value;

        i = i + 1;
      };

      (amount_processed, expected_amount, expected_amount == amount_processed)
    }

    /// Default payment mode, sends fund to a slow wallet
    /// @returns: u64
    /// -  the total amount able to transfer (including account limits)
    fun handle_slow_wallet_payment(vm: &signer, t: &TimedTransfer): u64 {
        // make a slow payment
        let multisig_address = guid::id_creator_address(&t.uid);

        // if the account is a community wallet, then we assume
        // the transfers will be locked.
        let coin_opt = ol_account::vm_withdraw_unlimited(vm, multisig_address,
        t.tx.value);
        let amount_transferred = 0;
        if (option::is_some(&coin_opt)) {
          let c = option::extract(&mut coin_opt);
          amount_transferred = coin::value(&c);
          ol_account::vm_deposit_coins_locked(vm, t.tx.payee, c);
        };
        option::destroy_none(coin_opt);

        amount_transferred
    }


  //////// GOVERNANCE HANDLERS ////////

  // Governance logic is defined in donor_voice_governance.move
  // Below are functions to handle the cases for rejecting and freezing accounts based on governance outcomes.

  //////// VETO ////////
  // A validator casts a vote to veto a proposed/pending transaction
  // by a TxSchedule wallet.
  // The validator identifies the transaction by a unique id.
  // Tallies are computed on the fly, such that if a veto happens,
  // the community which is faster than waiting for epoch boundaries.
  // NOTE: veto and tx both have UIDs but they are separate
  fun veto_handler(
    sender: &signer,
    ballot_uid: &guid::ID,
    tx_uid: &guid::ID,
  ) acquires TxSchedule, Freeze {
    let multisig_address = guid::id_creator_address(tx_uid);
    let tx_id_num = guid::id_creation_num(tx_uid);

    donor_voice_governance::assert_is_voter(sender, multisig_address);

    // NOTE: we are voting with the BALLOT ID
    let veto_is_approved = donor_voice_governance::veto_by_ballot_id(sender, ballot_uid);
    if (option::is_none(&veto_is_approved)) return;


    // check the TX is no longer scheduled
    assert!(is_scheduled(multisig_address, tx_id_num), error::invalid_state(ENO_PENDING_TRANSACTION_AT_UID));

    if (*option::borrow(&veto_is_approved)) {
      // if the veto passes, freeze the account
      reject(tx_uid);

      maybe_freeze(multisig_address);
    } else {
      // per the TxSchedule policy we need to slow
      // down the payments further if there are rejections.
      // Add another day for each veto
      let state = borrow_global_mut<TxSchedule>(multisig_address);
      let tx_mut = get_pending_timed_transfer_mut(state, tx_uid);
      if (tx_mut.epoch_latest_veto_received < epoch_helper::get_current_epoch()) {
        tx_mut.deadline = tx_mut.deadline + 1;

        // check that the expiration of the payment
        // is the same as the end of the veto ballot
        // This is because the ballot expiration can be
        // extended based on the threshold of votes.
        donor_voice_governance::sync_ballot_and_tx_expiration(sender, ballot_uid, tx_mut.deadline)
      }

    }
  }

  // private function. Once vetoed, the transaction is
  // removed from proposed list.
  fun reject(uid: &guid::ID)  acquires TxSchedule, Freeze {
    let multisig_address = guid::id_creator_address(uid);
    let id_num = guid::id_creation_num(uid);
    assert!(is_scheduled(multisig_address, id_num), error::invalid_state(ENO_PENDING_TRANSACTION_AT_UID));

    let c = borrow_global_mut<TxSchedule>(multisig_address);

    let len = vector::length(&c.scheduled);
    let i = 0;
    while (i < len) {
      let t = vector::borrow<TimedTransfer>(&c.scheduled, i);
      if (&t.uid == uid) {
        // remove from proposed list
        let t = vector::remove<TimedTransfer>(&mut c.scheduled, i);
        vector::push_back(&mut c.veto, t);
        // increment consecutive rejections counter
        let f = borrow_global_mut<Freeze>(multisig_address);
        f.consecutive_rejections = f.consecutive_rejections + 1;

      };

      i = i + 1;
    };

  }

  /// propose and vote on the veto of a specific transaction.
  /// The transaction must first have been scheduled, otherwise this proposal will abort.
  fun propose_veto(donor: &signer, uid_of_tx: &guid::ID): Option<guid::ID>  acquires TxSchedule {
    let multisig_address = guid::id_creator_address(uid_of_tx);
    donor_voice_governance::assert_is_voter(donor, multisig_address);
    let state = borrow_global<TxSchedule>(multisig_address);
    // need to check if the tx is already scheduled.

    let (found, _index, status) = find_schedule_by_id(state, uid_of_tx);
    if (found && status == scheduled_enum()) {
      let epochs_duration = DEFAULT_VETO_DURATION;

      let uid = donor_voice_governance::propose_veto(&state.guid_capability, uid_of_tx, epochs_duration);
      return option::some(uid)
    };
    option::none()
  }


  #[test_only]
  public(friend) fun test_propose_veto(donor: &signer, uid_of_tx: &guid::ID):
  Option<guid::ID>  acquires TxSchedule {
    propose_veto(donor, uid_of_tx)
  }

  /// If there are approved transactions, then the consecutive rejection counter is reset.
  fun reset_rejection_counter(vm: &signer, wallet: address) acquires Freeze {
    system_addresses::assert_ol(vm);
    borrow_global_mut<Freeze>(wallet).consecutive_rejections = 0;
  }

  /// TxSchedule wallets get frozen if 3 consecutive attempts to transfer are rejected.
  fun maybe_freeze(wallet: address) acquires Freeze {
    if (borrow_global<Freeze>(wallet).consecutive_rejections > 2) {
      let f = borrow_global_mut<Freeze>(wallet);
      f.is_frozen = true;
    }
  }

  fun get_pending_timed_transfer_mut(state: &mut TxSchedule, uid: &guid::ID): &mut TimedTransfer {
    let (found, i) = schedule_status(state, uid, scheduled_enum());

    assert!(found, error::invalid_argument(ENO_PENDING_TRANSACTION_AT_UID));
    vector::borrow_mut<TimedTransfer>(&mut state.scheduled, i)
  }

  fun schedule_status(state: &TxSchedule, uid: &guid::ID, state_enum: u8): (bool, u64) {
    let list = if (state_enum == scheduled_enum()) { &state.scheduled }
    else if (state_enum == veto_enum()) { &state.veto }
    else if (state_enum == paid_enum()) { &state.paid }
    else {
      assert!(false, error::invalid_argument(ENOT_VALID_STATE_ENUM));
      &state.scheduled  // dummy
    };

    let len = vector::length(list);
    let i = 0;
    while (i < len) {
      let t = vector::borrow<TimedTransfer>(list, i);
      if (&t.uid == uid) {
        return (true, i)
      };

      i = i + 1;
    };
    (false, 0)
  }


  #[test_only]
  public fun test_propose_liquidation(donor: &signer, multisig_address: address)
  acquires TxSchedule {
    propose_liquidation(donor, multisig_address);

  }
  //////// LIQUIDATION ////////
  /// propose and vote on the liquidation of this wallet
  fun propose_liquidation(donor: &signer, multisig_address: address)  acquires TxSchedule {
    donor_voice_governance::assert_is_voter(donor, multisig_address);
    let state = borrow_global<TxSchedule>(multisig_address);
    donor_voice_governance::propose_liquidate(&state.guid_capability, LIQUIDATION_TALLY_DAYS);
  }

  /// Once a liquidation has been proposed, other donors can vote on it.
  fun liquidation_handler(donor: &signer, multisig_address: address) acquires Freeze {
      donor_voice_governance::assert_is_voter(donor, multisig_address);
      let res = donor_voice_governance::vote_liquidation(donor, multisig_address);

      if (option::is_some(&res) && *option::borrow(&res)) {
        // The VM will call this function to liquidate the wallet.
        // the donors cannot do this because they cant get the withdrawal capability
        // from the multisig account.

        // first we freeze it so nothing can happen in the interim.
        let f = borrow_global_mut<Freeze>(multisig_address);
        f.is_frozen = true;
        // let f = borrow_global_mut<Registry>(@ol_framework);
        donor_voice::add_to_liquidation_queue(multisig_address);
    }
  }

  /// The VM will call this function to liquidate all Donor Voice
  /// wallets in the queue.
  public(friend) fun vm_liquidate(vm: &signer) acquires Freeze {
    system_addresses::assert_ol(vm);
    let liq = donor_voice::get_liquidation_queue();
    let len = vector::length(&liq);

    let i = 0;
    while (i < len) {
      let multisig_address = vector::swap_remove(&mut liq, i);

      // if this account was tagged a community wallet, then the
      // funds get split pro-rata at the current split of the
      // burn recycle algorithm.
      // Easiest way to do this is to send it to transaction fee account
      // so it can be split up by the burn recycle algorithm.
      // and trying to call Burn, here will create a circular dependency.


      if (!is_liquidate_to_match_index(multisig_address)) {
        // otherwise the default case is that donors get their funds back.
        let (pro_rata_addresses, pro_rata_amounts) = get_pro_rata(multisig_address);
        let k = 0;
        let len = vector::length(&pro_rata_addresses);
        // then we split the funds and send it back to the user's wallet
        while (k < len) {
            let addr = vector::borrow(&pro_rata_addresses, k);
            let amount = vector::borrow(&pro_rata_amounts, k);
            // in the case of community wallets where it gets
            // liquidated to the matching index
            let coin_opt = ol_account::vm_withdraw_unlimited(vm, multisig_address, *amount);
            if (option::is_some(&coin_opt)) {
              let c = option::extract(&mut coin_opt);

              if (is_liquidate_to_match_index(multisig_address)) {
                match_index::match_and_recycle(vm, &mut c);
                option::fill(&mut coin_opt, c);
              } else {
                // in the ordinary case, where it goes back to the donors
                ol_account::vm_deposit_coins_locked(vm, *addr, c);
              };
            };
            option::destroy_none(coin_opt);
            k = k + 1;
        };
      };

      // there may be some superman 3 funds left from rounding issues
      let (_, balance) = ol_account::balance(multisig_address);
      let coin_opt = ol_account::vm_withdraw_unlimited(vm, multisig_address, balance);
      if (option::is_some(&coin_opt)) {
        let c = option::extract(&mut coin_opt);
        transaction_fee::vm_pay_fee(vm, @ol_framework, c);
      };
      option::destroy_none(coin_opt);

      i = i + 1;
    }
  }

  /// option to set the liquidation destination to infrastructure escrow
  /// must be done before the multisig is finalized and the sponsor cannot control the account.
  public(friend) fun set_liquidate_to_match_index(sponsor: &signer, liquidate_to_match_index: bool) acquires Freeze {
    let f = borrow_global_mut<Freeze>(signer::address_of(sponsor));
    f.liquidate_to_match_index = liquidate_to_match_index;
  }


  public fun get_tx_params(t: &TimedTransfer): (address, u64, vector<u8>, u64) {
    (t.tx.payee, t.tx.value, *&t.tx.description, t.deadline)
  }

    /// Get the state of a specific transaction in the multisig workflow
  /// Returns (found, idx, status_enum, completed)
  public fun get_multisig_proposal_state(multisig_address: address, uid: &guid::ID): (bool, u64, u8, bool) {
    multi_action::get_proposal_status_by_id<Payment>(multisig_address, uid)
  }

  /// Get the state of a scheduled transaction
  /// Returns (found, idx, state_enum)
  public fun get_schedule_state(multisig_address: address, uid: &guid::ID): (bool, u64, u8) acquires TxSchedule {
    assert!(exists<TxSchedule>(multisig_address), error::not_found(ENOT_INIT_DONOR_VOICE));
    let state = borrow_global<TxSchedule>(multisig_address);
    find_schedule_by_id(state, uid)
  }

  // Mapping the status enums to the multi-action (ballot) naming.

  // multisig approval process enums
  #[view]
  public fun voting_enum(): u8 {
    ballot::get_pending_enum() // 1
  }
  #[view]
  public fun approved_enum(): u8 {
    ballot::get_approved_enum() // 2
  }
  #[view]
  public fun rejected_enum(): u8 {
    ballot::get_rejected_enum() // 3
  }

  // payment process enums

  #[view]
  public fun scheduled_enum(): u8 {
    4
  }
  #[view]
  public fun paid_enum(): u8 {
    5
  }
  #[view]
  public fun veto_enum(): u8 {
    6
  }


  //////// VIEWS ////////


  #[view]
  /// Check if the account is a Donor Voice account, and initialized properly.
  public fun is_donor_voice(multisig_address: address):bool {
    multi_action::is_multi_action(multisig_address) &&
    multi_action::has_action<Payment>(multisig_address) &&
    exists<Freeze>(multisig_address) &&
    exists<TxSchedule>(multisig_address)
  }

  #[view]
  /// Get the complete status of a transaction in both the multisig workflow and scheduling process
  ///
  /// This function provides a comprehensive view of a transaction's status throughout its lifecycle,
  /// including both its state in the multisig approval process and its state in the payment scheduling system.
  /// It's particularly useful for applications that need to track transactions across all stages
  /// of the donor voice workflow.
  ///
  /// # Parameters
  /// * `multisig_address`: The address of the donor voice account
  /// * `tx_id`: The transaction ID number to query
  ///
  /// # Returns
  /// A tuple containing:
  /// * `proposal_found`: Whether the transaction was found in the multisig proposals
  /// * `proposal_idx`: The index of the transaction in the multisig proposal list
  /// * `proposal_status`: The status of the proposal in the multisig workflow:
  ///   - `1`: Pending approval (voting)
  ///   - `2`: Approved
  ///   - `3`: Rejected
  /// * `proposal_completed`: Whether the multisig proposal has been completed
  /// * `schedule_found`: Whether the transaction was found in the scheduling system
  /// * `schedule_idx`: The index of the transaction in the scheduling list
  /// * `schedule_status`: The status of the transaction in the scheduling system:
  ///   - `4`: Scheduled for execution
  ///   - `5`: Paid/executed
  ///   - `6`: Vetoed
  ///
  /// # Example Usage
  /// ```
  /// // Get full status of transaction with ID 42
  /// let (proposal_found, proposal_idx, proposal_status, proposal_completed,
  ///     schedule_found, schedule_idx, schedule_status) =
  ///     donor_voice_txs::get_tx_status(donor_voice_address, 42);
  ///
  /// // Check if the transaction is approved by the multisig and scheduled for payment
  /// if (proposal_found && proposal_status == 2 && proposal_completed &&
  ///     schedule_found && schedule_status == 4) {
  ///     // Transaction is approved and scheduled for payment
  /// }
  /// ```
  public fun get_tx_status(multisig_address: address, tx_id: u64): (bool, u64, u8, bool, bool, u64, u8) acquires TxSchedule {
    let uid = guid::create_id(multisig_address, tx_id);
    let (proposal_found, proposal_idx, proposal_status, proposal_completed) = get_multisig_proposal_state(multisig_address, &uid);

    let (schedule_found, schedule_idx, schedule_status) = get_schedule_state(multisig_address, &uid);
    (proposal_found, proposal_idx, proposal_status, proposal_completed, schedule_found, schedule_idx, schedule_status)
  }

  #[view]
  /// Check if a transaction proposal is currently in voting stage in the multisig workflow
  /// Uses a transaction ID number instead of a GUID ID reference
  public fun is_voting(donor_voice_address: address, tx_id: u64): bool  {
    let uid = guid::create_id(donor_voice_address, tx_id);
    let (found, _, status, _) = get_multisig_proposal_state(donor_voice_address,
    &uid);
    found && (status == voting_enum())
  }

  #[view]
  /// Check if a transaction proposal has been approved in the multisig workflow
  /// Uses a transaction ID number instead of a GUID ID reference
  public fun is_approved(donor_voice_address: address, tx_id: u64): bool {
    let uid = guid::create_id(donor_voice_address, tx_id);
    let (found, _, status, _) =
    get_multisig_proposal_state(donor_voice_address, &uid);
     found && (status == approved_enum())
  }

  #[view]
  /// Check if a transaction proposal has been rejected in the multisig workflow
  /// Uses a transaction ID number instead of a GUID ID reference
  public fun is_rejected(donor_voice_address: address, tx_id: u64): bool {
    let uid = guid::create_id(donor_voice_address, tx_id);
    let (found, _, status, _) =
    get_multisig_proposal_state(donor_voice_address, &uid);
    found && (status == rejected_enum())
  }

  #[view]
  /// Check if a transaction has been scheduled for execution
  /// Uses a transaction ID number instead of a GUID ID reference
  public fun is_scheduled(donor_voice_address: address, tx_id: u64): bool acquires TxSchedule {
    let uid = guid::create_id(donor_voice_address, tx_id);
    let (_, _, state) = get_schedule_state(donor_voice_address, &uid);
    state == scheduled_enum()
  }

  #[view]
  /// Check if a transaction has been paid/executed successfully
  /// Uses a transaction ID number instead of a GUID ID reference
  public fun is_paid(donor_voice_address: address, tx_id: u64): bool acquires TxSchedule {
    let uid = guid::create_id(donor_voice_address, tx_id);
    let (_, _, state) = get_schedule_state(donor_voice_address, &uid);
    state == paid_enum()
  }

  #[view]
  /// Check if a transaction has been vetoed
  /// Uses a transaction ID number instead of a GUID ID reference
  public fun is_veto(directed_address: address, tx_id: u64): bool acquires TxSchedule {
    let uid = guid::create_id(directed_address, tx_id);
    let (_, _, state) = get_schedule_state(directed_address, &uid);
    state == veto_enum()
  }


  #[view]
  // getter to check if wallet is frozen
  // used in account before attempting a transfer.
  public fun is_account_frozen(addr: address): bool acquires Freeze{
    let f = borrow_global<Freeze>(addr);
    f.is_frozen
  }

  #[view]
  public fun is_liquidate_to_match_index(addr: address): bool acquires Freeze{
    let f = borrow_global<Freeze>(addr);
    f.liquidate_to_match_index
  }


  #[view]
  /// get the proportion of donations of all donors to account.
  public fun get_pro_rata(multisig_address: address): (vector<address>, vector<u64>) {
    // get total fees
    let (_, current_balance) = ol_account::balance(multisig_address);
    let all_time_donations = cumulative_deposits::get_cumulative_deposits(multisig_address);
    let donors = cumulative_deposits::get_depositors(multisig_address);
    let pro_rata_addresses = vector::empty<address>();
    let pro_rata_amounts = vector::empty<u64>();

    let i = 0;
    let len = vector::length(&donors);
    while (i < len) {
      let donor = vector::borrow(&donors, i);
      let (_, _, my_total_donations) = receipts::read_receipt(*donor, multisig_address);

      // proportionally how much has this donor contributed to the all time cumulative donations.
      let ratio = fixed_point32::create_from_rational(my_total_donations, all_time_donations);
      // of the current remaining balance, how much would be the share attributed to that donor.
      let pro_rata = fixed_point32::multiply_u64(current_balance, ratio);

      vector::push_back(&mut pro_rata_addresses, *donor);
      vector::push_back(&mut pro_rata_amounts, pro_rata);
      i = i + 1;
    };

      (pro_rata_addresses, pro_rata_amounts)
  }

  #[view]
  /// get the total aggregate balance in all donor voice accounts.
  public fun get_dv_supply(): u64 {
    let list = donor_voice::get_root_registry();
    let sum = 0;
    vector::for_each(list, |addr| {
      let (_, balance) =  ol_account::balance(addr);
      sum = sum + balance;
    });
    sum
  }

  #[view]
  /// Get all transaction IDs for a specific status across multisig workflow and scheduling
  ///
  /// This function retrieves all transaction IDs that are currently in the specified state
  /// from either the multisig proposal process (pending, approved, rejected) or
  /// the scheduling system (scheduled, paid, vetoed).
  ///
  /// # Parameters
  /// * `donor_voice_address`: The address of the donor voice account to query
  /// * `status_enum`: The status to filter transactions by:
  ///   - `1`: Pending approval (voting in multisig)
  ///   - `2`: Approved (in multisig)
  ///   - `3`: Rejected (in multisig)
  ///   - `4`: Scheduled for execution
  ///   - `5`: Paid/executed
  ///   - `6`: Vetoed
  ///
  /// # Returns
  /// * `vector<u64>`: A vector containing all transaction IDs matching the specified status
  ///
  /// # Aborts
  /// * `ENOT_VALID_STATE_ENUM`: If the provided status_enum is not valid
  public fun list_by_status(donor_voice_address: address, status_enum: u8): vector<u64> acquires TxSchedule {
    // For status enums 1-3, we need to check proposals in multisig workflow
    if (status_enum == voting_enum() || status_enum == approved_enum() || status_enum == rejected_enum()) {
      return multi_action::get_proposals_by_status<Payment>(donor_voice_address, status_enum)
    };

    // For status enums 4-6, we check the scheduling system
    let state = borrow_global<TxSchedule>(donor_voice_address);

    // Select the right list based on status_enum
    let tx_list = if (status_enum == scheduled_enum()) {
      &state.scheduled
    } else if (status_enum == paid_enum()) {
      &state.paid
    } else if (status_enum == veto_enum()) {
      &state.veto
    } else {
      assert!(false, error::invalid_argument(ENOT_VALID_STATE_ENUM));
      &state.scheduled  // dummy return that never executes due to the assert
    };

    let ids = vector::empty<u64>();
    let i = 0;
    let len = vector::length(tx_list);

    while (i < len) {
      let t = vector::borrow(tx_list, i);
      vector::push_back(&mut ids, guid::id_creation_num(&t.uid));
      i = i + 1;
    };

    ids
  }

  // // Helper function to list proposals by status in the multisig system
  // fun list_multisig_proposals_by_status(multisig_address: address, status_enum: u8): vector<u64> {
  //   // Check if we have access to the multi_action module for the Payment type
  //   if (!multi_action::has_action<Payment>(multisig_address)) {
  //     return vector::empty<u64>()
  //   };

  //   // Use the new get_proposals_by_status function to fetch IDs directly
  //   multi_action::get_proposals_by_status<Payment>(multisig_address, status_enum)
  // }

  //////// TRANSACTIONS ////////

  // /// A signer of the multisig can propose a payment
  // }

  public entry fun propose_payment_tx(
    auth: signer,
    multisig_address: address,
    payee: address,
    value: u64,
    description: vector<u8>,
    advance_unlocked: bool,
  )  acquires TxSchedule {
    donor_voice_reauth::assert_authorized(multisig_address);

    let pay_is_slow = slow_wallet::is_slow(payee);
    // confirm the user's intention here
    if (advance_unlocked) {
      // check the amount attempting to be used, must be
      // than the Advance credit allowed
      assert!(!pay_is_slow, error::invalid_argument(EUNAUTHORIZED_SLOW_WALLET));
      let can_withdraw = community_wallet_advance::can_withdraw_amount(multisig_address, value);

      assert!(can_withdraw, error::invalid_argument(EUNLOCKED_COIN_LIMIT_EXCEEDED));
    } else {
      // if the user is not using advance unlocked, then
      // check the recipient is a slow wallet
      assert!(pay_is_slow, error::invalid_argument(EPAYEE_NOT_SLOW));
    };

    propose_payment(&auth, multisig_address, payee, value, description);
  }


  // VETO TXs
  /// A donor of the program can propose a veto to a scheduled transaction
  /// by the tx_id.
  /// Public entry function required for txs cli.
  public entry fun propose_veto_tx(donor: &signer, multisig_address: address, tx_id: u64) acquires TxSchedule, Freeze{
    let guid = guid::create_id(multisig_address, tx_id);
    // one thing is the id of the scheduled payment
    // another ID is that of the veto proposal
    let opt_uid_of_gov_prop = propose_veto(donor, &guid);
    if (option::is_some(&opt_uid_of_gov_prop)) { // check successful proposal
      let veto_uid = option::borrow(&opt_uid_of_gov_prop);
      veto_handler(donor, veto_uid, &guid);
    }
  }

  // /// A signer of the multisig can propose a payment
  // /// Public entry function required for txs cli
  // public entry fun propose_advance_tx(
  //   auth: signer,
  //   multisig_address: address,
  //   payee: address,
  //   value: u64,
  //   description: vector<u8>,
  // ) {
  //   donor_voice_reauth::assert_authorized(multisig_address);
  //   propose_advance(&auth, multisig_address, payee, value, description);
  // }

  // REAUTH TXs

  /// After proposed, subsequent donors can vote to reauth an account
  /// Public entry function required for txs cli.
  public entry fun vote_reauth_tx(donor: &signer, multisig_address: address) acquires TxSchedule {
    donor_voice_governance::assert_is_voter(donor, multisig_address);
    let state = borrow_global<TxSchedule>(multisig_address);
    let guid_cap = &state.guid_capability;
    // TODO: add approve/reject options
    donor_voice_governance::vote_reauthorize(donor, guid_cap, true);
  }

  public entry fun maybe_close_reauth(multisig_address: address) {
    donor_voice_governance::garbage_gov<donor_voice_governance::Reauth>(multisig_address);
  }

  // LIQUIDATE TXS

  /// A donor can propose the liquidation of a Donor Voice account
  /// Public entry function required for txs cli.
  public entry fun propose_liquidate_tx(donor: &signer, multisig_address: address)  acquires TxSchedule {
    propose_liquidation(donor, multisig_address);
  }
  /// After proposed, subsequent voters call this to vote liquidation
  public entry fun vote_liquidation_tx(donor: &signer, multisig_address: address) acquires Freeze {
    liquidation_handler(donor, multisig_address);
  }

  //////// TEST HELPERS ////////

  #[test_only]
  /// NOTE: this is needed because tests requires the the GUID of the transfer.
  public(friend) fun test_propose_payment(
    sender: &signer,
    multisig_address: address,
    payee: address,
    value: u64,
    description: vector<u8>,
  ): guid::ID acquires TxSchedule {

    propose_payment(
    sender,
    multisig_address,
    payee,
    value,
    description)
  }

  #[test_only]
  // TODO: flagged for deprecation. Propose_veto_tx handles both proposing and
  // voting
  /// After proposed, subsequent veto voters call this to vote on a tx veto
  public fun vote_veto_tx(donor: &signer, multisig_address: address, tx_id_num: u64)  acquires TxSchedule, Freeze {
    let tx_uid = guid::create_id(multisig_address, tx_id_num);
    let (found, ballot_uid) = donor_voice_governance::find_tx_veto_id(tx_uid);
    assert!(found, error::invalid_argument(ENO_VETO_ID_FOUND));
    veto_handler(donor, &ballot_uid, &tx_uid);
  }

  #[test_only]
  public(friend) fun find_by_deadline(multisig_address: address, epoch: u64): vector<guid::ID> acquires TxSchedule {
    let state = borrow_global_mut<TxSchedule>(multisig_address);
    let i = 0;
    let list = vector::empty<guid::ID>();

    while (i < vector::length(&state.scheduled)) {

      let prop = vector::borrow(&state.scheduled, i);
      if (prop.deadline == epoch) {
        vector::push_back(&mut list, *&prop.uid);
      };

      i = i + 1;
    };

    list
  }

  #[test_only]
  public fun test_helper_vm_liquidate(vm: &signer) acquires Freeze {
    vm_liquidate(vm);
  }

  #[test_only]
  public fun test_helper_make_donor_voice(vm: &signer, sig: &signer, initial_authorities: vector<address>) {
    use ol_framework::testnet;
    testnet::assert_testnet(vm);
    make_donor_voice(sig);
    multi_action::propose_offer_internal(sig, initial_authorities, option::none());
    donor_voice_reauth::test_set_authorized(vm, signer::address_of(sig));
  }
}
