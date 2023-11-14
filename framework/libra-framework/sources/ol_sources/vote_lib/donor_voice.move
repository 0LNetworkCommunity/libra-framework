
/// Donor Voice wallets is a service of the chain.
/// Any address can voluntarily turn their account into a Donor Voice account.

/// Definitions
/// Unless otherwise specified the assumption of the Donor Voice app is
/// that there is an Owner of the account and property.
/// The Owner can assign Proxy Authorities, who acting as custodians can issue
/// transactions on behalf of Owner
/// Depositors are called Donors, and the app gives depositors
/// visibility of the transactions, and also limited authority over
/// specific actions: alterting the Owner and Depositors from
/// unauthorized transaction.

/// The DonorVoice Account Lifecycle:
/// Proxy Authorities use a MultiSig to schedule ->
/// Once scheduled the Donors use a TurnoutTally to Veto ->
/// Epoch boundary: transaction executes when the VM reads the Schedule struct at the epoch boundary, and issues payment.

/// By creating a TxSchedule wallet you are providing certain restrictions and guarantees to the users that interact with this wallet.

/// 1. The wallet's contents is propoperty of the owner. The owner is free to issue transactions which change the state of the wallet, including transferring funds. There are however time, and veto policies.

/// 2. All transfers out of the account are timed. Meaning, they will execute automatically after a set period of time passes. The VM address triggers these events at each epoch boundary. The purpose of the delayed transfers is that the transaction can be paused for analysis, and eventually rejected by the donors of the wallet.

/// 3. Every pending transaction can be "vetoed". The vetos delay the finalizing of the transaction, to allow more time for analysis. Each veto adds one day/epoch to the transaction PER DAY THAT A VETO OCCURRS. That is, two vetos happening in the same day, only extend the vote by one day. If a sufficient number of Donors vote on the Veto, then the transaction will be rejected. Since TxSchedule has an expiration time, as does ParticipationVote, each time there is a veto, the deadlines for both are syncronized, based on the new TxSchedule expiration time.

/// 4. After three consecutive transaction rejections, the account will become frozen. The funds remain in the account but no operations are available until the Donors, un-freeze the account.

/// 5. Voting for all purposes are done on a pro-rata basis according to the amounts donated. Voting using ParticipationVote method, which in short, biases the threshold based on the turnout of the vote. TL;DR a low turnout of 12.5% would require 100% of the voters to veto, and lower thresholds for higher turnouts until 51%.

/// 6. The donors can vote to liquidate a frozen TxSchedule account. The result will depend on the configuration of the TxSchedule account from when it was initialized: the funds by default return to the end user who was the donor.

/// 7. Third party contracts can wrap the Donor Voice wallet. The outcomes of the votes can be returned to a handler in a third party contract For example, liquidiation of a frozen account is programmable: a handler can be coded to determine the outcome of the Donor Voice wallet. See in CommunityWallets the funds return to the InfrastructureEscrow side-account of the user.

module ol_framework::donor_voice {
    use std::vector;
    use std::signer;
    use std::error;
    use std::guid;
    use std::fixed_point32;
    use std::option::{Self, Option};
    use diem_framework::system_addresses;
    use diem_framework::coin;
    use ol_framework::epoch_helper;
    use ol_framework::ol_account;
    use ol_framework::receipts;
    use ol_framework::multi_action;
    use ol_framework::account::{Self, WithdrawCapability};
    use ol_framework::donor_voice_governance;
    use ol_framework::ballot;
    use ol_framework::cumulative_deposits;
    use ol_framework::transaction_fee;
    use ol_framework::match_index;

    // use diem_std::debug::print;

    friend ol_framework::community_wallet;

    /// Not initialized as a Donor Voice account.
    const ENOT_INIT_DONOR_VOICE: u64 = 1;
    /// User is not a donor and cannot vote on this account
    const ENOT_AUTHORIZED_TO_VOTE: u64 = 2;
    /// Could not find a pending transaction by this GUID
    const ENO_PEDNING_TRANSACTION_AT_UID: u64 = 3;
    /// No enum for this number
    const ENOT_VALID_STATE_ENUM: u64 = 4;
    /// No enum for this number
    const EMULTISIG_NOT_INIT: u64 = 5;
    /// No enum for this number
    const ENO_VETO_ID_FOUND: u64 = 6;

    const SCHEDULED: u8 = 1;
    const VETO: u8 = 2;
    const PAID: u8 = 3;

    /// number of epochs to wait before a transaction is executed
    /// Veto can happen in this time
    /// at the end of the third epoch from when multisig gets consensus
    const DEFAULT_PAYMENT_DURATION: u64 = 3;
    /// minimum amount of time to evaluate when one donor flags for veto.
    const DEFAULT_VETO_DURATION: u64 = 7;


    // root registry for the Donor Voice accounts
    struct Registry has key {
      list: vector<address>,
      liquidation_queue: vector<address>,
    }

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

    // struct Donors has key {
    //   list: vector<address>,
    // }


    // // A flag on the account that it wants to be considered a community walley
    // struct CommunityWallet has key { }


    //////// INIT REGISRTY OF DONOR VOICE ACCOUNTS  ////////

    // Donor Voice Accounts are a root security service. So the root account needs to keep a registry of all Donor Voice accounts, using this service.

    // Utility used at genesis (and on upgrade) to initialize the system state.
    public fun initialize(vm: &signer) {
      system_addresses::assert_ol(vm);

      if (!is_root_init()) {
        move_to<Registry>(vm, Registry {
          list: vector::empty<address>(),
          liquidation_queue: vector::empty<address>(),
        });
      };
    }

    public fun is_root_init():bool {
      exists<Registry>(@ol_framework)
    }


    public fun migrate_root_registry(vm: &signer, list: vector<address>) {
      system_addresses::assert_ol(vm);
      if (!is_root_init()) {
        move_to<Registry>(vm, Registry {
          list,
          liquidation_queue: vector::empty<address>(),
        });
      };
    }

    // can only be called by genesis
    public(friend) fun migrate_community_wallet_account(vm: &signer, dv_account:
    &signer) acquires Registry {
      system_addresses::assert_ol(vm);
      let liquidate_to_match_index = true;
      // skip setting up the multisig
      structs_init(dv_account, liquidate_to_match_index);
      add_to_registry(dv_account);
    }

    //////// DONOR VOICE INITIALIZATION ////////
    // There are three steps in initializing an account. These steps can be combined in a single transaction, or done in separate transactions. The "bricking" of the sponsor key should be done in a separate transaction, in case there are any errors in the initialization.

    // 1. The basic structs for a Donor Voice account need to be initialized, and the account needs to be added to the Registry at root.

    // 2. A MultiSig action structs need to be initialized.

    // 3. Once the MultiSig is initialized, the account needs to be bricked, before the MultiSig can be used.

    public fun make_donor_voice(sponsor: &signer, init_signers: vector<address>, cfg_n_signers: u64) acquires Registry {
      cumulative_deposits::init_cumulative_deposits(sponsor);

      // we are setting liquidation to match_index as false by default
      // the user can send another transacton to change this.
      let liquidate_to_match_index = false;
      structs_init(sponsor, liquidate_to_match_index);
      make_multi_action(sponsor, cfg_n_signers, init_signers);
      add_to_registry(sponsor);
    }

    fun structs_init(sig: &signer, liquidate_to_match_index: bool) {
      if (!exists<Registry>(@ol_framework)) return;

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

      donor_voice_governance::init_donor_governance(sig);
    }

    // add to root registry
    fun add_to_registry(sig: &signer) acquires Registry {
      if (!exists<Registry>(@ol_framework)) return;

      let addr = signer::address_of(sig);
      let list = get_root_registry();
      if (!vector::contains<address>(&list, &addr)) {
        let s = borrow_global_mut<Registry>(@ol_framework);
        vector::push_back(&mut s.list, addr);
      };
    }



    /// Like any MultiSig instance, a sponsor which is the original owner of the account, needs to initialize the account.
    /// The account must be "bricked" by the owner before MultiSig actions can be taken.
    /// Note, as with any multisig, the new_authorities cannot include the sponsor, since that account will no longer be able to sign transactions.
    public fun make_multi_action(sponsor: &signer, cfg_default_n_sigs: u64, new_authorities: vector<address>) {
      multi_action::init_gov(sponsor, cfg_default_n_sigs, &new_authorities);
      multi_action::init_type<Payment>(sponsor, true); // "true": We make this multisig instance hold the WithdrawCapability. Even though we don't need it for any account pay functions, we can use it to make sure the entire pipeline of private functions scheduling a payment are authorized. Belt and suspenders.
    }


    /// Check if the account is a Donor Voice account, and initialized properly.
    public fun is_donor_voice(multisig_address: address):bool {
      multi_action::is_multi_action(multisig_address) &&
      multi_action::has_action<Payment>(multisig_address) &&
      exists<Freeze>(multisig_address) &&
      exists<TxSchedule>(multisig_address)
    }

    // Getter for retrieving the list of TxSchedule wallets.
    public fun get_root_registry(): vector<address> acquires Registry{
      if (exists<Registry>(@ol_framework)) {
        let s = borrow_global<Registry>(@ol_framework);
        return *&s.list
      } else {
        return vector::empty<address>()
      }
    }

    ///////// MULTISIG ACTIONS TO SCHEDULE A TIMED TRANSFER /////////
    /// As in any MultiSig instance, the transaction which proposes the action (the scheduled transfer) must be signed by an authority on the MultiSig.
    /// The same function is the handler for the approval case of the MultiSig action.
    /// Since Donor Voice accounts are involved with sensitive assets, we have moved the WithdrawCapability to the MultiSig instance. Even though we don't need it for any account functions for paying, we use it to ensure no private functions related to assets can be called. Belt and suspenders.

    /// Returns the GUID of the transfer.
    public fun propose_payment(
      sender: &signer,
      multisig_address: address,
      payee: address,
      value: u64,
      description: vector<u8>
    ): guid::ID acquires TxSchedule {
      let tx = Payment {
        payee,
        value,
        description,
      };

      // TODO: get expiration
      let prop = multi_action::proposal_constructor(tx, option::none());

      let uid = multi_action::propose_new<Payment>(sender, multisig_address, prop);

      let (passed, withdraw_cap_opt) = multi_action::vote_with_id<Payment>(sender, &uid, multisig_address);

      let tx = multi_action::extract_proposal_data(multisig_address, &uid);

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
    // at that point the transaction leves the proposed queue, and is added
    // the rejected list.

    fun schedule(
      withdraw_capability: &WithdrawCapability, tx: Payment, uid: &guid::ID
    ) acquires TxSchedule {

      let multisig_address = account::get_withdraw_cap_address(withdraw_capability);
      let transfers = borrow_global_mut<TxSchedule>(multisig_address);

      let deadline = epoch_helper::get_current_epoch() + DEFAULT_PAYMENT_DURATION;

      let t = TimedTransfer {
        uid: *uid,
        deadline, // pays automatically at the end of seventh epoch. Unless there is a veto by a Donor. In that case a day is added for every day there is a veto. This deduplicates Vetos.
        tx,
        epoch_latest_veto_received: 0,
      };

      // let id = guid::id(&t.uid);
      vector::push_back<TimedTransfer>(&mut transfers.scheduled, t);
      // return id
    }

    /// saerch for a transction ID in the queues. Returns (is found, index, status enum)
    public fun find_schedule_by_id(state: &TxSchedule, uid: &guid::ID): (bool, u64, u8) { // (is_found, index, state)
      let (found, i) = schedule_status(state, uid, SCHEDULED);
      if (found) return (found, i, SCHEDULED);

      let (found, i) = schedule_status(state, uid, VETO);
      if (found) return (found, i, VETO);

      let (found, i) = schedule_status(state, uid, PAID);
      if (found) return (found, i, PAID);

      (false, 0, 0)
    }

    ///////// PROCESS PAYMENTS /////////
    /// The VM on epoch boundaries will execute the payments without the users
    /// needing to intervene.
    /// Returns (accounts_processed, amount_processed, success)
    // TODO: add to the return the tuple of sender/recipient that failed
    public fun process_donor_voice_accounts(
      vm: &signer,
      epoch: u64,
    ): (u64, u64, bool) acquires Registry, TxSchedule, Freeze {
      // while we are here let's liquidate any expired accounts.
      vm_liquidate(vm);

      let accounts_processed = 0;
      let amount_processed = 0;
      let expected_amount = 0;

      let list = get_root_registry();

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

    /// tries to settle any amounts that have been scheduled for payment
    /// for audit instrumentation returns how much was actually transferred
    /// and if that amount was equal to the expected amount transferred (amount_processed, expected_amount, succcess)
    fun maybe_pay_deadline(vm: &signer, state: &mut TxSchedule, epoch: u64): (u64, u64, bool) acquires Freeze {
      let expected_amount = 0;
      let amount_processed = 0;
      let i = 0;

      while (i < vector::length(&state.scheduled)) {

        let this_exp = *&vector::borrow(&state.scheduled, i).deadline;
        if (this_exp == epoch) {
          let t = vector::remove(&mut state.scheduled, i);
          let multisig_address = guid::id_creator_address(&t.uid);

          // Note the VM can do this without the WithdrawCapability
          expected_amount = expected_amount + t.tx.value;

          // if the account is a community wallet, then we assume
          // the transfers will be locked.
          let coin_opt = ol_account::vm_withdraw_unlimited(vm, multisig_address,
          t.tx.value);
          let amount_transferred = 0;
          // TBD: transfers from DV which are not CW
          // There's a circular dependency with CW which
          // prevents from making a switch case here.
          if (option::is_some(&coin_opt)) {
            let c = option::extract(&mut coin_opt);
            amount_transferred = coin::value(&c);
            ol_account::vm_deposit_coins_locked(vm, t.tx.payee, c);
          };
          option::destroy_none(coin_opt);

          amount_processed = amount_processed + amount_transferred;

          // update the records
          vector::push_back(&mut state.paid, t);

          // if theres a single transaction that gets approved, then the freeze consecutive rejection counter is reset
          reset_rejection_counter(vm, multisig_address)
        };

        i = i + 1;
      };

      (amount_processed, expected_amount, expected_amount == amount_processed)
    }

    public fun find_by_deadline(multisig_address: address, epoch: u64): vector<guid::ID> acquires TxSchedule {
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
    veto_uid: &guid::ID,
    tx_uid: &guid::ID,
  ) acquires TxSchedule, Freeze {
    let multisig_address = guid::id_creator_address(tx_uid);
    donor_voice_governance::assert_authorized(sender, multisig_address);

    let veto_is_approved = donor_voice_governance::veto_by_id(sender, veto_uid);
    if (option::is_none(&veto_is_approved)) return;

    let (_found, _idx, state) = get_schedule_state(multisig_address, tx_uid);
    assert!(state == 1, error::invalid_state(ENOT_VALID_STATE_ENUM)); // is scheduled

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
        donor_voice_governance::sync_ballot_and_tx_expiration(sender, veto_uid, tx_mut.deadline)
      }

    }
  }

  // private function. Once vetoed, the transaction is
  // removed from proposed list.
  fun reject(uid: &guid::ID)  acquires TxSchedule, Freeze {
    let multisig_address = guid::id_creator_address(uid);
    let (found, _idx, state) = get_schedule_state(multisig_address, uid);
    assert!(found, error::invalid_state(ENO_PEDNING_TRANSACTION_AT_UID));
    assert!(state == 1, error::invalid_state(ENOT_VALID_STATE_ENUM));

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
  public fun propose_veto(donor: &signer, uid_of_tx: &guid::ID): Option<guid::ID>  acquires TxSchedule {
    let multisig_address = guid::id_creator_address(uid_of_tx);
    donor_voice_governance::assert_authorized(donor, multisig_address);
    let state = borrow_global<TxSchedule>(multisig_address);
    // need to check if the tx is already schdules.

    let (found, _index, status) = find_schedule_by_id(state, uid_of_tx);
    if (found && status == SCHEDULED) {
      let epochs_duration = DEFAULT_VETO_DURATION;

      let uid = donor_voice_governance::propose_veto(&state.guid_capability, uid_of_tx,  epochs_duration);
      return option::some(uid)
    };
    option::none()
  }

  /// If there are approved transactions, then the consectutive rejection counter is reset.
  public fun reset_rejection_counter(vm: &signer, wallet: address) acquires Freeze {
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

    public fun get_pending_timed_transfer_mut(state: &mut TxSchedule, uid: &guid::ID): &mut TimedTransfer {
      let (found, i) = schedule_status(state, uid, SCHEDULED);

      assert!(found, error::invalid_argument(ENO_PEDNING_TRANSACTION_AT_UID));
      vector::borrow_mut<TimedTransfer>(&mut state.scheduled, i)
    }

    public fun schedule_status(state: &TxSchedule, uid: &guid::ID, state_enum: u8): (bool, u64) {
      let list = if (state_enum == SCHEDULED) { &state.scheduled }
      else if (state_enum == VETO) { &state.veto }
      else if (state_enum == PAID) { &state.paid }
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


    //////// LIQUIDATION ////////
    /// propose and vote on the liquidation of this wallet
    public fun propose_liquidation(donor: &signer, multisig_address: address)  acquires TxSchedule {
      donor_voice_governance::assert_authorized(donor, multisig_address);
      let state = borrow_global<TxSchedule>(multisig_address);
      let epochs_duration = 365; // liquidation vote can take a whole year
      donor_voice_governance::propose_liquidate(&state.guid_capability, epochs_duration);
    }

    /// Once a liquidation has been proposed, other donors can vote on it.
    fun liquidation_handler(donor: &signer, multisig_address: address) acquires Freeze, Registry {
      donor_voice_governance::assert_authorized(donor, multisig_address);
      let res = donor_voice_governance::vote_liquidation(donor, multisig_address);

      if (option::is_some(&res) && *option::borrow(&res)) {
        // The VM will call this function to liquidate the wallet.
        // the donors cannot do this because they cant get the withdrawal capability
        // from the multisig account.

        // first we freeze it so nothing can happen in the interim.
        let f = borrow_global_mut<Freeze>(multisig_address);
        f.is_frozen = true;
        let f = borrow_global_mut<Registry>(@ol_framework);
        vector::push_back(&mut f.liquidation_queue, multisig_address);
    }
  }

  #[view]
  // list of accounts that are pending liquidation after a successful vote to liquidate
  public fun get_liquidation_queue(): vector<address> acquires Registry{
    let f = borrow_global<Registry>(@ol_framework);
    *&f.liquidation_queue
  }

  /// The VM will call this function to liquidate all Donor Voice
  /// wallets in the queue.
   public(friend) fun vm_liquidate(vm: &signer) acquires Freeze, Registry {
      system_addresses::assert_ol(vm);
      let f = borrow_global_mut<Registry>(@ol_framework);
      let len = vector::length(&f.liquidation_queue);

      let i = 0;
      while (i < len) {
        let multisig_address = vector::swap_remove(&mut f.liquidation_queue, i);

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

   #[test_only]
   public fun test_helper_vm_liquidate(vm: &signer) acquires Registry, Freeze {
    vm_liquidate(vm);
   }

    /// get the proportion of donoations of all donors to account.
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

    //////// GETTERS ////////
    public fun get_tx_params(t: &TimedTransfer): (address, u64, vector<u8>, u64) {
      (t.tx.payee, t.tx.value, *&t.tx.description, t.deadline)
    }


    /// Check the status of proposals in the MultiSig Workflow
    /// NOTE: These are payments that have not yet been scheduled.
    public fun get_multisig_proposal_state(directed_address: address, uid: &guid::ID): (bool, u64, u8, bool) { // (is_found, index, state)

      multi_action::get_proposal_status_by_id<Payment>(directed_address, uid)
    }

    /// Get the status of a SCHEDULED payment which as already passed the multisig stage.
    public fun get_schedule_state(directed_address: address, uid: &guid::ID): (bool, u64, u8) acquires TxSchedule { // (is_found, index, state)
      let state = borrow_global<TxSchedule>(directed_address);
      find_schedule_by_id(state, uid)
    }

    public fun is_scheduled(directed_address: address, uid: &guid::ID): bool acquires TxSchedule {
      let (_, _, state) = get_schedule_state(directed_address, uid);
      state == ballot::get_pending_enum()
    }

    public fun is_paid(directed_address: address, uid: &guid::ID): bool acquires TxSchedule {
      let (_, _, state) = get_schedule_state(directed_address, uid);
      state == ballot::get_approved_enum()
    }

    public fun is_veto(directed_address: address, uid: &guid::ID): bool acquires TxSchedule {
      let (_, _, state) = get_schedule_state(directed_address, uid);
      state == ballot::get_rejected_enum()
    }

    // getter to check if wallet is frozen
    // used in account before attempting a transfer.
    public fun is_account_frozen(addr: address): bool acquires Freeze{
      let f = borrow_global<Freeze>(addr);
      f.is_frozen
    }

    public fun is_liquidate_to_match_index(addr: address): bool acquires Freeze{
      let f = borrow_global<Freeze>(addr);
      f.liquidate_to_match_index
    }


    //////// TRANSACTION SCRIPTS ////////

    // public fun init_donor_voice(sponsor: &signer, init_signers: vector<address>, cfg_n_signers: u64) acquires Registry {
    //   // let init_signers = vector::singleton(signer_one);
    //   // vector::push_back(&mut init_signers, signer_two);
    //   // vector::push_back(&mut init_signers, signer_three);

    //   cumulative_deposits::init_cumulative_deposits(sponsor);

    //   // we are setting liquidation to infra escrow as false by default
    //   // the user can send another transacton to change this.
    //   let liquidate_to_match_index = false;
    //   set_donor_voice(sponsor, liquidate_to_match_index);
    //   make_multi_action(sponsor, cfg_n_signers, init_signers);
    //   add_to_registry(sponsor);
    // }

    /// option to set the liquidation destination to infrastructure escrow
    /// must be done before the multisig is finalized and the sponsor cannot control the account.
    public fun set_liquidate_to_match_index(sponsor: &signer, liquidate_to_match_index: bool) acquires Freeze {
      let f = borrow_global_mut<Freeze>(signer::address_of(sponsor));
      f.liquidate_to_match_index = liquidate_to_match_index;
    }

    // /// the sponsor must finalize the initialization, this is a separate step so that the user can optionally check everything is in order before bricking the account key.
    // public fun finalize_init(sponsor: &signer) acquires Registry {
    //   let multisig_address = signer::address_of(sponsor);
    //   assert!(multi_action::is_multi_action(multisig_address), error::invalid_state(EMULTISIG_NOT_INIT));

    //   assert!(multi_action::has_action<Payment>(multisig_address), error::invalid_state(EMULTISIG_NOT_INIT));

    //   assert!(exists<Freeze>(multisig_address), error::invalid_state(ENOT_INIT_DONOR_VOICE));

    //   assert!(exists<TxSchedule>(multisig_address), error::invalid_state(ENOT_INIT_DONOR_VOICE));

    //   // multi_action::finalize_and_brick(sponsor);
    //   assert!(is_donor_voice(multisig_address), error::invalid_state(ENOT_INIT_DONOR_VOICE));

    //   // only add to registry if INIT is successful.
    //   add_to_registry(sponsor);
    // }



    //////// TX HELPER ////////

    // transaction helper to wrap Donor Voice init
    public entry fun make_donor_voice_tx(sponsor: &signer, init_signers: vector<address>, cfg_n_signers: u64) acquires Registry {
      make_donor_voice(sponsor, init_signers, cfg_n_signers);
    }

    public entry fun propose_payment_tx(
      auth: signer,
      multisig_address: address,
      payee: address,
      value: u64,
      description: vector<u8>
    )  acquires TxSchedule {
      propose_payment(&auth, multisig_address, payee, value, description);
    }


    public entry fun propose_veto_tx(donor: &signer, multisig_address: address, id: u64) acquires TxSchedule, Freeze{
      let tx_uid = guid::create_id(multisig_address, id);
      let opt_uid_of_gov_prop = propose_veto(donor, &tx_uid);
      if (option::is_some(&opt_uid_of_gov_prop)) { // check successful proposal
        let veto_uid = option::borrow(&opt_uid_of_gov_prop);
        veto_handler(donor, veto_uid, &tx_uid);
      }
    }

    /// Entry functiont to vote the veto.
    public entry fun vote_veto_tx(donor: &signer, multisig_address: address, id: u64)  acquires TxSchedule, Freeze {
      let tx_uid = guid::create_id(multisig_address, id);
      let (found, veto_uid) = donor_voice_governance::find_tx_veto_id(tx_uid);
      assert!(found, error::invalid_argument(ENO_VETO_ID_FOUND));
      veto_handler(donor, &veto_uid, &tx_uid);
    }

    public entry fun propose_liquidate_tx(donor: &signer, multisig_address: address)  acquires TxSchedule {
      propose_liquidation(donor, multisig_address);
    }

    public entry fun vote_liquidation_tx(donor: &signer, multisig_address: address) acquires Freeze, Registry {
      liquidation_handler(donor, multisig_address);
    }
}
