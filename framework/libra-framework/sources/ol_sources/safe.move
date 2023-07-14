///////////////////////////////////////////////////////////////////////////
// 0L Module
// Governance
// A payment tool for transfers which require n-of-m approvals
///////////////////////////////////////////////////////////////////////////


// Governance is a module which allows for a group of authorities to approve an a generic "Action". The Action type can be defined by an external contract, and the Governance module will only check if the Action has been approved by the required number of authorities.
// similarly any handler for the Action can be executed by an external contract, and the Governance module will only check if the Action has been approved by the required number of authorities.
// Each Action has a separate data structure for tabulating the votes in approval of the Action. But there is shared state between the Actions, that being Governance, which contains the constraints for each Action that are checked on each vote (n_sigs, expiration, signers, etc)
// The Actions are triggered "lazily", that is: the last authorized sender of a proposal/vote, is the one to trigger the action.
// Theere is no offline signature aggregation. The authorities over the address should not require collecting signatures offline: proposal should be submitted directly to this contract.

// With this design, the multisig can be used for different actions. A MultiSigPayment contract is an example of a Root Service which the chain provides, which leverages the Governance module to provide a payment service which requires n-of-m approvals.

//V7 NOTE: from V6 we are refactoring so the the account first needs to be created as a "resource account". It's a minor change given that V6 had a similar construct of a "signerless account", Previously in ol this meant to "Brick" the authkey after the WithdrawCapability was stored in a common struct. Vendor had independenly made the same design using Signer Capability.

module ol_framework::safe {
  use std::vector;
  use std::option::{Self, Option};
  use std::signer;
  use std::error;
  use std::guid;
  use aptos_framework::account::{Self, WithdrawCapability};
  use aptos_framework::resource_account;
  use ol_framework::ballot::{Self, BallotTracker};
  use aptos_framework::reconfiguration;
  // use DiemFramework::Debug::print;

  const EGOV_NOT_INITIALIZED: u64 = 440000;
  /// The owner of this account can't be an authority, since it will subsequently be bricked. The signer of this account is no longer useful. The account is now controlled by the Governance logic.
  const ESIGNER_CANT_BE_AUTHORITY: u64 = 440001;
  /// signer not authorized to approve a transaction.
  const ENOT_AUTHORIZED: u64 = 440002;
  /// There are no pending transactions to search
  const EPENDING_EMPTY: u64 = 440003;
  /// Not enough signers configured
  const ENO_SIGNERS: u64 = 440004;
  /// The multisig setup  is not finalized, the sponsor needs to brick their authkey. The account setup sponsor needs to be verifiably locked out before operations can begin.
  const ENOT_FINALIZED_NOT_BRICK: u64 = 440005;
  /// Already registered this action type
  const EACTION_ALREADY_EXISTS: u64 = 440006;
  /// Action not found
  const EACTION_NOT_FOUND: u64 = 440007;
  /// Proposal is expired
  const EPROPOSAL_EXPIRED: u64 = 440008;
  /// Proposal is expired
  const EDUPLICATE_PROPOSAL: u64 = 440009;
  /// Proposal is expired
  const EPROPOSAL_NOT_FOUND: u64 = 440010;
  /// Proposal voting is closed
  const EVOTING_CLOSED: u64 = 440011;


  /// default setting for a proposal to expire
  const DEFAULT_EPOCHS_EXPIRE: u64 = 14;

  /// A Governance account is an account which requires multiple votes from Authorities to  send a transaction.
  /// A multisig can be used to get agreement on different types of Actions, such as a payment transaction where the handler code for the transaction is an a separate contract. See for example MultiSigPayment.
  /// Governance struct holds the metadata for all the instances of Actions on this account.
  /// Every action has the same set of authorities and governance.
  /// This is intentional, since privilege escalation can happen if each action has a different set of governance, but access to funds and other state.
  /// If the organization wishes to have Actions with different governance, then a separate Account is necessary.


  /// DANGER
  // Governance optionally holds a WithdrawCapability, which is used to withdraw funds from the account. All actions share the same WithdrawCapability.
  /// The WithdrawCapability can be used to withdraw funds from the account.
  /// Ordinarily only the signer/owner of this address can use it.
  /// We are bricking the signer, and as such the withdraw capability is now controlled by the Governance logic.
  /// Core Devs: This is a major attack vector. The WithdrawCapability should NEVER be returned to a public caller, UNLESS it is within the vote and approve flow.

  /// Note, the WithdrawCApability is moved to this shared structure, and as such the signer of the account is bricked. The signer who was the original owner of this account ("sponsor") can no longer issue transactions to this account, and as such the WithdrawCapability would be inaccessible. So on initialization we extract the WithdrawCapability into the Governance governance struct.

  //TODO: feature: signers is a hashmap and each can have a different weight
  struct Governance has key {
    cfg_duration_epochs: u64,
    cfg_default_n_sigs: u64,
    signers: vector<address>,
    withdraw_capability: Option<WithdrawCapability>, // for the calling function to be able to do asset moving operations.
    guid_capability: account::GUIDCapability, // this is needed to create GUIDs for the Ballot.
  }

  struct Action<ProposalData> has key, store {
    can_withdraw: bool,
    vote: BallotTracker<Proposal<ProposalData>>,
  }

  // All proposals share some common fields
  // and each proposal can add type-specific parameters
  // The handler for such specific parameters needs to included in code by an external contract.
  // Governance, will only say if it passed or not.
  // Note: The underlying Ballot deals with the GUID generation
  struct Proposal<ProposalData> has store, drop {
    // id: u64,
    // The transaction to be executed
    proposal_data: ProposalData,
    // The votes received
    votes: vector<address>,
    // approved
    approved: bool,
    // The expiration time for the transaction
    expiration_epoch: u64,
  }


  public fun proposal_constructor<ProposalData: store + drop>(proposal_data: ProposalData, duration_epochs: Option<u64>): Proposal<ProposalData> {

    let duration_epochs = if (option::is_some(&duration_epochs)) {
      *option::borrow(&duration_epochs)
    } else {
      DEFAULT_EPOCHS_EXPIRE
    };

    Proposal<ProposalData> {
      // id: 0,
      proposal_data,
      votes: vector::empty<address>(),
      approved: false,
      expiration_epoch: reconfiguration::get_current_epoch() + duration_epochs,
    }
  }


  fun assert_authorized(sig: &signer, multisig_address: address) acquires Governance {
        // cannot start manipulating contract until it is finalized
    assert!(is_finalized(multisig_address), error::invalid_argument(ENOT_FINALIZED_NOT_BRICK));

    assert!(exists<Governance>(multisig_address), error::invalid_argument(ENOT_AUTHORIZED));

    // check sender is authorized
    let sender_addr = signer::address_of(sig);
    assert!(is_authority(multisig_address, sender_addr), error::invalid_argument(ENOT_AUTHORIZED));
  }


  // Initialize the governance structs for this account.
  // Governance contains the constraints for each Action that are checked on each vote (n_sigs, expiration, signers, etc)
  // Also, an initial Action of type PropGovSigners is created, which is used to govern the signers and threshold for this account.
  public fun init_gov(sig: &signer, cfg_default_n_sigs: u64, m_seed_authorities: &vector<address>) {
    assert!(cfg_default_n_sigs > 0, error::invalid_argument(ENO_SIGNERS));

    let multisig_address = signer::address_of(sig);
    // User footgun. The signer of this account is bricked, and as such the signer can no longer be an authority.
    assert!(!vector::contains(m_seed_authorities, &multisig_address), error::invalid_argument(ESIGNER_CANT_BE_AUTHORITY));

    if (!exists<Governance>(multisig_address)) {
        move_to(sig, Governance {
        cfg_duration_epochs: DEFAULT_EPOCHS_EXPIRE,
        cfg_default_n_sigs,
        signers: *m_seed_authorities,
        // counter: 0,
        withdraw_capability: option::none(),
        guid_capability: account::create_guid_capability(sig),
      });
    };

    if (!exists<Action<PropGovSigners>>(multisig_address)) {
      move_to(sig, Action<PropGovSigners> {
        can_withdraw: false,
        // pending: vector::empty(),
        // approved: vector::empty(),
        // rejected: vector::empty(),
        vote: ballot::new_tracker<Proposal<PropGovSigners>>(),
      });
    }
  }

  //////// Helper functions to check initialization //////////
  /// Is the Multisig Governance initialized?
  public fun is_init(multisig_address: address): bool {
    exists<Governance>(multisig_address) &&
    exists<Action<PropGovSigners>>(multisig_address)
  }

  /// Has a multisig struct for a given action been created?
  public fun has_action<ProposalData: store>(addr: address):bool {
    exists<Action<ProposalData>>(addr)
  }

  /// An initial "sponsor" who is the signer of the initialization account calls this function.
  // This function creates the data structures, but also IMPORTANTLY it rotates the AuthKey of the account to a system-wide unusuable key (b"brick_all_your_base_are_belong_to_us").
  public fun init_type<ProposalData: store + drop >(
    sig: &signer,
    can_withdraw: bool,
   ) acquires Governance {
    let multisig_address = signer::address_of(sig);
    // TODO: there is no way of creating a new Action by multisig. The "signer" would need to be spoofed, which account does only in specific and scary situations (e.g. vm_create_account_migration)

    assert!(is_init(multisig_address), error::invalid_argument(EGOV_NOT_INITIALIZED));

    assert!(!exists<Action<ProposalData>>(multisig_address), error::invalid_argument(EACTION_ALREADY_EXISTS));
    // make sure the signer's address is not in the list of authorities.
    // This account's signer will now be useless.



    // maybe the withdraw cap was never extracted in previous set up.
    // but we won't extract it if none of the Actions require it.
    if (can_withdraw) {
      maybe_extract_withdraw_cap(sig);
    };

    move_to(sig, Action<ProposalData> {
        can_withdraw,
        // pending: vector::empty(),
        // approved: vector::empty(),
        // rejected: vector::empty(),
        vote: ballot::new_tracker<Proposal<ProposalData>>(),
      });
  }


  fun maybe_extract_withdraw_cap(sig: &signer) acquires Governance {
    let multisig_address = signer::address_of(sig);
    assert!(exists<Governance>(multisig_address), error::invalid_argument(ENOT_AUTHORIZED));

    let ms = borrow_global_mut<Governance>(multisig_address);
    if (option::is_some(&ms.withdraw_capability)) {
      return
    } else {
      let cap = account::extract_withdraw_capability(sig);
      option::fill(&mut ms.withdraw_capability, cap);
    }
  }

  public fun maybe_restore_withdraw_cap(sig: &signer, multisig_addr: address, w: Option<WithdrawCapability>) acquires Governance {
    assert_authorized(sig, multisig_addr);
    assert!(exists<Governance>(multisig_addr), error::invalid_argument(ENOT_AUTHORIZED));
    if (option::is_some(&w)) {
      let ms = borrow_global_mut<Governance>(multisig_addr);
      let cap = option::extract(&mut w);
      option::fill(&mut ms.withdraw_capability, cap);
    };
    option::destroy_none(w);

  }

  // /// Once the "sponsor" which is setting up the multisig has created all the multisig types (payment, generic, gov), they need to brick this account so that the signer for this address is rendered useless, and it is a true multisig.
  // public fun finalize_and_brick(sig: &signer) {
  //   account::brick_this(sig, b"yes I know what I'm doing");
  //   assert!(is_finalized(signer::address_of(sig)), error::invalid_state(ENOT_FINALIZED_NOT_BRICK));
  // }

  public fun is_finalized(addr: address): bool {
    resource_account::is_resource_account(addr)
  }


  // Propose an Action
  // Transactions should be easy, and have one obvious way to do it. There should be no other method for voting for a tx.
  // this function will catch a duplicate, and vote in its favor.
  // This causes a user interface issue, users need to know that you cannot have two open proposals for the same transaction.
  // It's optional to state how many epochs from today the transaction should expire. If the transaction is not approved by then, it will be rejected.
  // The default will be 14 days.
  // Only the first proposer can set the expiration time. It will be ignored when a duplicate is caught.

  public fun propose_new<ProposalData: store + drop>(
    sig: &signer,
    multisig_address: address,
    proposal_data: Proposal<ProposalData>,
  ): guid::ID acquires Governance, Action {
    // print(&20);
    assert_authorized(sig, multisig_address);
// print(&21);
    let ms = borrow_global_mut<Governance>(multisig_address);
    let action = borrow_global_mut<Action<ProposalData>>(multisig_address);
    // print(&22);
    // go through all proposals and clean up expired ones.
    lazy_cleanup_expired(action);
// print(&23);
    // does this proposal already exist in the pending list?
    let (found, guid, _idx, status_enum, _is_complete) = search_proposals_for_guid<ProposalData>(&action.vote, &proposal_data);
    // print(&found);
    // print(&status_enum);
    // print(&24);
    if (found && status_enum == ballot::get_pending_enum()) {
      // print(&2401);
      // this exact proposal is already pending, so we we will just return the guid of the existing proposal.
      // we'll let the caller decide what to do (we wont vote by default)
      return guid
    };

    let guid = account::create_guid_with_capability(&ms.guid_capability);

// print(&25);
    let ballot = ballot::propose_ballot(&mut action.vote, guid, proposal_data);
// print(&26);
    let id = ballot::get_ballot_id(ballot);
// print(&27);
    id
  }


  public fun vote_with_data<ProposalData: store + drop>(sig: &signer, proposal: &Proposal<ProposalData>, multisig_address: address): (bool, Option<WithdrawCapability>) acquires Governance, Action {
    assert_authorized(sig, multisig_address);

    let action = borrow_global_mut<Action<ProposalData>>(multisig_address);
    // let ms = borrow_global_mut<Governance>(multisig_address);
    // go through all proposals and clean up expired ones.
    // lazy_cleanup_expired(action);

    // does this proposal already exist in the pending list?
    let (found, uid, _idx, _status_enum, _is_complete) = search_proposals_for_guid<ProposalData>(&action.vote, proposal);

    assert!(found, error::invalid_argument(EPROPOSAL_NOT_FOUND));

    vote_impl<ProposalData>(sig, multisig_address, &uid)

  }


  /// helper function to vote with ID only
  public fun vote_with_id<ProposalData: store + drop>(sig: &signer, id: &guid::ID, multisig_address: address): (bool, Option<WithdrawCapability>) acquires Governance, Action {
    assert_authorized(sig, multisig_address);

    // let action = borrow_global_mut<Action<ProposalData>>(multisig_address);
    vote_impl<ProposalData>(sig, multisig_address, id)

  }

  fun vote_impl<ProposalData: store + drop>(
    sig: &signer,
    // ms: &mut Governance,
    multisig_address: address,
    id: &guid::ID
  ): (bool, Option<WithdrawCapability>) acquires Governance, Action {

    // print(&60);
    assert_authorized(sig, multisig_address); // belt and suspenders
    let ms = borrow_global_mut<Governance>(multisig_address);
    let action = borrow_global_mut<Action<ProposalData>>(multisig_address);
    // print(&61);
    // always run this to cleanup all missing ballots
    lazy_cleanup_expired(action);

    // if (check_expired(&action.vote)) return (false, option::none());
    // print(&62);

    // does this proposal already exist in the pending list?
    let (found, _idx, status_enum, is_complete) = ballot::find_anywhere<Proposal<ProposalData>>(&action.vote, id);
    // print(&63);
    assert!(found, error::invalid_argument(EPROPOSAL_NOT_FOUND));
    assert!(status_enum == ballot::get_pending_enum(), error::invalid_argument(EVOTING_CLOSED));
     assert!(!is_complete, error::invalid_argument(EVOTING_CLOSED));

    // print(&64);
    let b = ballot::get_ballot_by_id_mut(&mut action.vote, id);
    let t = ballot::get_type_struct_mut(b);
    // print(&65);
    vector::push_back(&mut t.votes, signer::address_of(sig));
    // print(&66);
    let passed = tally(t, *&ms.cfg_default_n_sigs);
    // print(&67);

    if (passed) {
      ballot::complete_ballot(b);
    };

    // get the withdrawal capability, we're not allowed copy, but we can
    // extract and fill, and then replace it. See account for an example.
    let withdraw_cap = if (
      passed &&
      option::is_some(&ms.withdraw_capability) &&
      action.can_withdraw
    ) {
      let c = option::extract(&mut ms.withdraw_capability);
      option::some(c)
    } else {
      option::none()
    };

    // print(&withdraw_cap);
    // print(&68);

    (passed, withdraw_cap)
  }


  fun tally<ProposalData: store + drop>(prop: &mut Proposal<ProposalData>, n: u64): bool {
    // print(&40001);

    // print(&prop.votes);

    if (vector::length(&prop.votes) >= n) {
      prop.approved = true;
      // print(&40002);

      return true
    };

    false
  }



  fun find_expired<ProposalData: store + drop>(a: & Action<ProposalData>): vector<guid::ID>{
    // print(&40);
    let epoch = reconfiguration::get_current_epoch();
    let b_vec = ballot::get_list_ballots_by_enum(&a.vote, ballot::get_pending_enum());
    let id_vec = vector::empty();
    // print(&41);
    let i = 0;
    while (i < vector::length(b_vec)) {
      // print(&4101);
      let b = vector::borrow(b_vec, i);
      let t = ballot::get_type_struct<Proposal<ProposalData>>(b);


      if (epoch > t.expiration_epoch) {
        // print(&4010101);
        let id = ballot::get_ballot_id(b);
        // print(&4010102);
        vector::push_back(&mut id_vec, id);

      };
      i = i + 1;
    };

    id_vec
  }

  fun lazy_cleanup_expired<ProposalData: store + drop>(a: &mut Action<ProposalData>) {
    let expired_vec = find_expired(a);
    // print(&expired_vec);
    let len = vector::length(&expired_vec);
    // print(&len);
    let i = 0;
    while (i < len) {
      let id = vector::borrow(&expired_vec, i);
      // lets check the status just in case.
       ballot::move_ballot(&mut a.vote, id, ballot::get_pending_enum(), ballot::get_rejected_enum());
      i = i + 1;
    };
  }

  fun check_expired<ProposalData: store>(prop: &Proposal<ProposalData>): bool {
    let epoch_now = reconfiguration::get_current_epoch();
    epoch_now > prop.expiration_epoch
  }

  public fun is_authority(multisig_addr: address, addr: address): bool acquires Governance {
    let m = borrow_global<Governance>(multisig_addr);
    vector::contains(&m.signers, &addr)
  }

  /// This function is used to copy the data from the proposal that is in the multisig.
  /// Note that this is the only way to get the data out of the multisig, and it is the only function to use the `copy` trait. If you have a workflow that needs copying, then the data struct for the action payload will need to use the `copy` trait.
    public fun extract_proposal_data<ProposalData: store + copy + drop>(multisig_address: address, uid: &guid::ID): ProposalData acquires Action {
      let a = borrow_global<Action<ProposalData>>(multisig_address);
      let b = ballot::get_ballot_by_id(&a.vote, uid);
      let t = ballot::get_type_struct<Proposal<ProposalData>>(b);

      let Proposal<ProposalData> {
          proposal_data: existing_data,
          expiration_epoch: _,
          votes: _,
          approved: _,
      } = t;

      *existing_data
    }


    /// returns a tuple of (is_found: bool, index: u64, status_enum: u8, is_complete: bool)
    public fun search_proposals_for_guid<ProposalData: drop + store> (
      tracker: &BallotTracker<Proposal<ProposalData>>,
      data: &Proposal<ProposalData>,
    ): (bool, guid::ID, u64, u8, bool)  {
     // looking in pending

     let (found, guid, idx) = find_index_of_ballot_by_data(tracker, data, ballot::get_pending_enum());
     if (found) {
      let b = ballot::get_ballot_by_id(tracker, &guid);
      let complete = ballot::is_completed<Proposal<ProposalData>>(b);
       return (true, guid, idx, ballot::get_pending_enum(), complete)
     };

    let (found, guid, idx) = find_index_of_ballot_by_data(tracker, data, ballot::get_approved_enum());
     if (found) {
      let b = ballot::get_ballot_by_id(tracker, &guid);
      let complete = ballot::is_completed<Proposal<ProposalData>>(b);
       return (true, guid, idx, ballot::get_approved_enum(), complete)
     };

    let (found, guid, idx) = find_index_of_ballot_by_data(tracker, data, ballot::get_rejected_enum());
     if (found) {
      let b = ballot::get_ballot_by_id(tracker, &guid);
      let complete = ballot::is_completed<Proposal<ProposalData>>(b);
       return (true, guid, idx, ballot::get_rejected_enum(), complete)
     };

      (false, guid::create_id(@0x0, 0), 0, 0, false)
    }

    public fun find_index_of_ballot_by_data<ProposalData: drop + store> (
      tracker: &BallotTracker<Proposal<ProposalData>>,
      incoming_proposal: &Proposal<ProposalData>,
      status_enum: u8,
    ): (bool, guid::ID, u64) {
      let Proposal<ProposalData> {
          proposal_data: incoming_data,
          expiration_epoch: _,
          votes: _,
          approved: _,
      } = incoming_proposal;

     let list = ballot::get_list_ballots_by_enum<Proposal<ProposalData>>(tracker, status_enum);

      let i = 0;
      while (i < vector::length(list)) {
        let b = vector::borrow(list, i);
        let t = ballot::get_type_struct<Proposal<ProposalData>>(b);

        // strip the votes and approved fields for comparison
        let Proposal<ProposalData> {
            proposal_data: existing_data,
            expiration_epoch: _,
            votes: _,
            approved: _,
        } = t;

        if (existing_data == incoming_data) {
          let uid = ballot::get_ballot_id(b);
          return (true, uid, i)
        };
        i = i + 1;
      };

      (false, guid::create_id(@0x0, 0), 0)
    }

  public fun get_proposal_status_by_id<ProposalData: drop + store>(multisig_address: address, uid: &guid::ID): (bool, u64, u8, bool) acquires Action { // found, index, status_enum, is_complete
    let a = borrow_global<Action<ProposalData>>(multisig_address);
    ballot::find_anywhere(&a.vote, uid)
  }


  ////////  GOVERNANCE  ////////
  // Governance of the multisig happens through an instance of Action<PropGovSigners>. This action has no special privileges, and is just a normal proposal type.
  // The entry point and handler for governance exists on this contract for simplicity. However, there's no reason it couldn't be called from an external contract.


  /// Tis is a ProposalData type for governance. This Proposal adds or removes a list of addresses as authorities. The handlers are located in this contract.
  struct PropGovSigners has key, store, copy, drop {
    add_remove: bool, // true = add, false = remove
    addresses: vector<address>,
    n_of_m: Option<u64>, // Optionally change the n of m threshold. To only change the n_of_m threshold, an empty list of addresses is required.
  }

  // Proposing a governance change of adding or removing signer, or changing the n-of-m of the authorities. Note that proposing will deduplicate in the event that two authorities miscommunicate and send the same proposal, in that case for UX purposes the second proposal becomes a vote.
  public fun propose_governance(sig: &signer, multisig_address: address, addresses: vector<address>, add_remove: bool, n_of_m: Option<u64>, duration_epochs: Option<u64> ): guid::ID acquires Governance, Action {
    assert_authorized(sig, multisig_address); // Duplicated with propose(), belt and suspenders
    let data = PropGovSigners {
      addresses,
      add_remove,
      n_of_m,
    };

    let prop = proposal_constructor<PropGovSigners>(data, duration_epochs);
    let id = propose_new<PropGovSigners>(sig, multisig_address, prop);
    vote_governance(sig, multisig_address, &id);

    id
  }

  /// This function can be called directly. Or the user can call propose_governance() with same parameters, which will deduplicate the proposal and instead vote. Voting is always a positive vote. There is no negative (reject) vote.
  public fun vote_governance(sig: &signer, multisig_address: address, id: &guid::ID): bool acquires Governance, Action {
    assert_authorized(sig, multisig_address);


    let (passed, cap_opt) = {
      // let ms = borrow_global_mut<Governance>(multisig_address);
      // let action = borrow_global_mut<Action<PropGovSigners>>(multisig_address);
      vote_impl<PropGovSigners>(sig, multisig_address, id)
    };
    maybe_restore_withdraw_cap(sig, multisig_address, cap_opt); // don't need this and can't drop.

    if (passed) {
      let ms = borrow_global_mut<Governance>(multisig_address);
      let data = extract_proposal_data<PropGovSigners>(multisig_address, id);
      maybe_update_authorities(ms, data.add_remove, &data.addresses);
      maybe_update_threshold(ms, &data.n_of_m);
    };
    passed
  }

  /// Updates the authorities of the multisig. This is a helper function for governance.
  // must be called with the withdraw capability and signer. belt and suspenders
  fun maybe_update_authorities(ms: &mut Governance, add_remove: bool, addresses: &vector<address>) {

      if (vector::is_empty(addresses)) {
        // The address field may be empty if the multisig is only changing the threshold
        return
      };

      if (add_remove) {
        vector::append(&mut ms.signers, *addresses);
      } else {

        // remove the signers
        let i = 0;
        while (i < vector::length(addresses)) {
          let addr = vector::borrow(addresses, i);
          let (found, idx) = vector::index_of(&ms.signers, addr);
          if (found) {
            vector::swap_remove(&mut ms.signers, idx);
          };
          i = i + 1;
        };
      };
  }

  fun maybe_update_threshold(ms: &mut Governance, n_of_m_opt: &Option<u64>) {
    if (option::is_some(n_of_m_opt)) {
      ms.cfg_default_n_sigs = *option::borrow(n_of_m_opt);
    };
  }

  //////// GETTERS ////////
  #[view]
  public fun get_authorities(multisig_address: address): vector<address> acquires Governance {
    let m = borrow_global<Governance>(multisig_address);
    *&m.signers
  }

  #[view]
  public fun get_threshold(multisig_address: address): (u64, u64) acquires Governance {
    let m = borrow_global<Governance>(multisig_address);
    (*&m.cfg_default_n_sigs, vector::length(&m.signers))
  }

}