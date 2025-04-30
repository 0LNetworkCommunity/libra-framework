/// Donor Voice account governance. See documentation at Donor Voice.move
/// This module includes controllers for initializing tallies for
/// 1. Donors Vetoing specific transactions.
/// 2. Donors periodically re-authorizing the mandate of the account owners program.
/// 3. Donors voting on liquidation of the account, regardless of the activity of the account.
///
/// Each governance vote (veto, reauth, liquidate) has view functions to:
/// - Check if a proposal exists
/// - Get the current vote tally
/// - Get the deadline (duration) of the vote

module ol_framework::donor_voice_governance {
    use std::error;
    use std::signer;
    use std::guid;
    use std::option::{Self, Option};
    use ol_framework::receipts;
    use ol_framework::turnout_tally::{Self, TurnoutTally};
    use ol_framework::ballot::{Self, Ballot, BallotTracker};
    use ol_framework::cumulative_deposits;
    use ol_framework::reauthorization;
    use diem_framework::account;
    use diem_framework::comparator;
    use ol_framework::epoch_helper;
    use ol_framework::donor_voice_reauth;
    use std::vector;

    friend ol_framework::donor_voice;
    friend ol_framework::donor_voice_txs;
    friend ol_framework::donor_voice_migration;

    //// ERROR CODES
    /// Is not a donor to this account
    const ENOT_A_DONOR: u64 = 1;
    /// No ballot found under that GUID
    const ENO_BALLOT_FOUND: u64 = 2;
    /// A proposal already exists with this data
    const EDUPLICATE_PROPOSAL: u64 = 3;
    /// No pending ballot found
    const ENO_PENDING_BALLOT_FOUND: u64 = 4;
    /// Reauthorization not required, the account is in good standing.
    const EREAUTH_NOT_REQUIRED: u64 = 5;

    /////// CONSTANTS ////////
    /// Tally expires after number of epochs.
    /// Note the CW needs to restart the tally if it expires
    /// It does not remain open
    const REAUTH_TALLY_EXPIRES: u64 = 30;

    /// Constants for governance types
    const GOV_TYPE_REAUTH: u8 = 1;
    const GOV_TYPE_LIQUIDATE: u8 = 2;
    const GOV_TYPE_VETO: u8 = 3;

    /// Data struct to store all the governance Ballots for vetoes
    /// allows for a generic type of Governance action, using the Participation Vote Poll type to keep track of ballots
    struct Governance<T> has key {
      tracker: BallotTracker<T>,
    }

    /// T type for veto
    struct Veto has drop, store {
      guid: guid::ID,
    }

    /// T type for liquidation
    struct Liquidate has drop, store {}

    /// T type for authorization
    struct Reauth has drop, store {}

    public(friend) fun maybe_init_dv_governance(dv_signer: &signer) {
      let addr = signer::address_of(dv_signer);

      if (!exists<Governance<TurnoutTally<Veto>>>(addr)) {
        let veto = Governance<TurnoutTally<Veto>> {
            tracker: ballot::new_tracker()
        };
        move_to(dv_signer, veto);
      };

      if (!exists<Governance<TurnoutTally<Liquidate>>>(addr)) {
        let liquidate = Governance<TurnoutTally<Liquidate>> {
            tracker: ballot::new_tracker()
        };
        move_to(dv_signer, liquidate);
      };

      if (!exists<Governance<TurnoutTally<Reauth>>>(addr)) {
        let reauth = Governance<TurnoutTally<Reauth>> {
            tracker: ballot::new_tracker()
        };
        move_to(dv_signer, reauth);
      };

      donor_voice_reauth::maybe_init(dv_signer);
    }

    /// return the index on the pending ballots list for the data searched
    fun pending_ballot_idx_by_data<T: drop + store>(dv_account: address, search_data: &T): (bool, u64) acquires Governance {
      let state = borrow_global_mut<Governance<TurnoutTally<T>>>(dv_account);
      let pending = ballot::get_list_ballots_by_enum(&state.tracker, ballot::get_pending_enum());

      let is_found = false;
      let count_found = 0;
      let idx = 0;

      let i = 0;
      while (i < vector::length(pending)) {
        let a_ballot = vector::borrow(pending, i);
        let turnout_tally = ballot::get_type_struct(a_ballot);
        let proposal_data = turnout_tally::get_tally_data(turnout_tally);
        if (search_data == proposal_data) {
          is_found = true;
          count_found = count_found + 1;
          idx = i;
        };
        i = i + 1;
      };

      assert!(count_found < 2, error::invalid_argument(EDUPLICATE_PROPOSAL));

      (is_found, idx)
    }

    /// get a mutable ballot from the pending list by index
    fun pending_ballot_mut_at_index<T: drop + store>(state: &mut Governance<TurnoutTally<T>>, index: u64): &mut Ballot<TurnoutTally<T>> {
      let pending = ballot::get_list_ballots_by_enum_mut(&mut state.tracker, ballot::get_pending_enum());
      vector::borrow_mut(pending, index)
    }

    /// a private function to propose a ballot for a veto. This is called by a verified donor.
    fun propose_gov<T: drop + store>(cap: &account::GUIDCapability, proposal: T, epochs_duration: u64): guid::ID acquires Governance {
      let dv_account = account::get_guid_capability_address(cap);
      let gov_state = borrow_global_mut<Governance<TurnoutTally<T>>>(dv_account);

      assert!(no_pending_proposal(&gov_state.tracker, &proposal), error::invalid_argument(EDUPLICATE_PROPOSAL));

      // what's the maximum universe of valid votes.
      let max_votes_enrollment = get_enrollment(dv_account);

      // enforce a minimum deadline. Liquidation deadlines are one year, Veto should be minimum 3.
      if (epochs_duration < 3) {
        epochs_duration = 3;
      };

      let deadline = epoch_helper::get_current_epoch() + epochs_duration;
      let max_extensions = 0; // infinite

      let t = turnout_tally::new_tally_struct(
        proposal,
        max_votes_enrollment,
        deadline,
        max_extensions
      );

      // set an ID for the proposal
      let guid = account::create_guid_with_capability(cap);
      let id = guid::id(&guid);

      ballot::propose_ballot(&mut gov_state.tracker, guid, t);
      id
    }

    /// Vote for a governance proposal of generic type.
    fun vote_gov<T: drop + store>(donor: &signer, multisig_address: address, ballot_id: u64, in_favor: bool): Option<bool> acquires Governance {
      assert_is_voter(donor, multisig_address);

      let state = borrow_global_mut<Governance<TurnoutTally<T>>>(multisig_address);

      // Create ballot GUID from the ID number
      let ballot_guid = guid::create_id(multisig_address, ballot_id);

      // Find the ballot with the given GUID
      let (found, idx, status_enum, _) = ballot::find_anywhere(&state.tracker, &ballot_guid);

      if (!found) {
        return option::none<bool>()
      };

      // Get the ballot and vote on it
      let ballot_mut = ballot::get_ballot_mut(&mut state.tracker, idx, status_enum);
      let user_weight = get_user_donations(multisig_address, signer::address_of(donor));

      turnout_tally::vote(donor, ballot::get_type_struct_mut(ballot_mut), &ballot_guid, in_favor, user_weight)
    }

    /// A generic governance tally function that returns detailed metrics about a governance vote.
    ///
    /// This function looks up a specific ballot by its GUID and returns comprehensive information
    /// about its current state, including voting percentages, thresholds, deadlines, and status.
    /// It's used by the governance system to determine the outcome of votes across different
    /// governance types (reauthorization, liquidation, veto).
    ///
    /// # Arguments
    ///
    /// * `dv_account` - The Donor Voice account address where the governance ballot is stored
    /// * `ballot_guid` - The unique GUID identifying the specific ballot to retrieve
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of governance action being tallied (Reauth, Liquidate, Veto)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `approval_pct` - The percentage (0-100) of votes that approved the proposal
    /// * `turnout_pct` - The percentage (0-100) of eligible voters who participated
    /// * `required_threshold` - The minimum approval percentage required for the proposal to pass
    /// * `epoch_deadline` - The epoch number when voting ends
    /// * `minimum_turnout` - The minimum percentage of eligible voters that must participate for a valid result
    /// * `approved` - Whether the proposal has reached the approval threshold
    /// * `is_complete` - Whether the voting process has been marked as complete
    /// * `status_enum` - The current status of the ballot (pending, approved, rejected)
    /// * `ballot_completed` - Whether the ballot itself has been marked as completed
    ///
    /// # Errors
    ///
    /// * `ENO_BALLOT_FOUND` - If no ballot with the specified GUID exists in any state
    fun tally_gov<T: drop + store>(dv_account: address, ballot_guid: guid::ID): (u64, u64, u64, u64, u64, bool, bool, u8, bool) acquires Governance {
      let state = borrow_global<Governance<TurnoutTally<T>>>(dv_account);

      // Search for the ballot using the provided GUID
      let (found, idx, status_enum, _) = ballot::find_anywhere(&state.tracker, &ballot_guid);
      assert!(found, error::invalid_argument(ENO_BALLOT_FOUND));

      // Get the ballot and its tally data
      let ballot = ballot::get_ballot(&state.tracker, idx, status_enum);
      let tally = ballot::get_type_struct(ballot);

      // Extract all the relevant metrics from the tally
      let approval_pct = turnout_tally::get_current_ballot_approval(tally);
      let turnout_pct = turnout_tally::get_current_ballot_participation(tally);
      let required_threshold = turnout_tally::get_current_threshold_required(tally);
      let epoch_deadline = turnout_tally::get_expiration_epoch(tally);
      let minimum_turnout = turnout_tally::get_minimum_turnout(tally);

      let (is_closed, approved) = turnout_tally::get_result(tally);
      // TODO there's some nomenclature issues
      // A ballot also keeps track of completion.
      let ballot_completed = ballot::is_completed(ballot);


      (approval_pct, turnout_pct, required_threshold, epoch_deadline, minimum_turnout, approved, is_closed, status_enum, ballot_completed)
    }

    /// Maybe close the poll and move the pending ballot to the approved or rejected list.
    /// This function checks if a specific ballot can be finalized, and if so,
    /// moves it from the pending list to either the approved or rejected list.
    ///
    /// # Arguments
    ///
    /// * `multisig_address` - The address of the Donor Voice account where the governance is stored
    /// * `ballot_id` - The specific ballot ID to check and potentially complete
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of governance action (Reauth, Liquidate, Veto)
    ///
    /// # Returns
    ///
    /// An option containing the result of the vote if completed:
    /// * `Some(true)` - The vote passed
    /// * `Some(false)` - The vote failed
    /// * `None` - The vote is still ongoing or no ballot exists
    fun maybe_tally_and_complete<T: drop + store>(multisig_address: address, ballot_id: u64): Option<bool> acquires Governance {
      let state = borrow_global_mut<Governance<TurnoutTally<T>>>(multisig_address);
      let ballot_guid = guid::create_id(multisig_address, ballot_id);

      // Find the ballot with the given ID in the pending list
      let (found, idx, status_enum, _) = ballot::find_anywhere(&state.tracker, &ballot_guid);

      if (!found || status_enum != ballot::get_pending_enum()) {
        return option::none<bool>()
      };

      let ballot_list = ballot::get_list_ballots_by_enum_mut(&mut state.tracker, status_enum);
      let this_ballot = vector::borrow_mut(ballot_list, idx);
      let tally_state = ballot::get_type_struct_mut(this_ballot);

      let result = turnout_tally::save_tally(tally_state);

      if (option::is_some(&result)) {
        let result_enum = if (*option::borrow(&result)) {
          ballot::get_approved_enum()
        } else {
          ballot::get_rejected_enum()
        };

        ballot::complete_and_move(&mut state.tracker, &ballot_guid, result_enum);
      };

      result
    }

    /// Returns the deadline (in epochs) for a ballot of the specified type.
    ///
    /// # Arguments
    ///
    /// * `dv_account` - The address of the Donor Voice account where the governance is stored
    /// * `data` - The data identifying the specific ballot
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of governance action (Reauth, Liquidate, Veto)
    ///
    /// # Returns
    ///
    /// The epoch number when the ballot expires
    ///
    /// # Errors
    ///
    /// * `ENO_PENDING_BALLOT_FOUND` - If no ballot is found with the given data
    fun deadline_gov<T: drop + store>(dv_account: address, data: &T): u64 acquires Governance {
      let (found, index) = pending_ballot_idx_by_data(dv_account, data);
      assert!(found, error::invalid_argument(ENO_PENDING_BALLOT_FOUND));
      let state = borrow_global_mut<Governance<TurnoutTally<T>>>(dv_account);
      let turnout_tally = pending_ballot_mut_at_index(state, index);
      let ballot_data = ballot::get_type_struct(turnout_tally);

      turnout_tally::get_expiration_epoch(ballot_data)
    }

    fun assert_pass_gov<T: drop + store>(dv_account: address, id: u64) acquires
    Governance {
      let ballot_guid = guid::create_id(dv_account, id);
      let state = borrow_global<Governance<TurnoutTally<T>>>(dv_account);

      // Search for the ballot using the provided GUID
      let (found, idx, status_enum, _) = ballot::find_anywhere(&state.tracker, &ballot_guid);

      assert!(found, error::invalid_argument(ENO_BALLOT_FOUND));

      // Get the ballot and its tally data
      let ballot = ballot::get_ballot(&state.tracker, idx, status_enum);
      let tally = ballot::get_type_struct(ballot);

      turnout_tally::assert_pass(tally);
    }

    /// Check if a proposal has already been made for this transaction.
    fun no_pending_proposal<T: drop + store>(tracker: &BallotTracker<TurnoutTally<T>>, data: &T): bool {
      // NOTE: Ballot.move does not check for duplicates. We need to check here.
      let list_pending = ballot::get_list_ballots_by_enum(tracker, ballot::get_pending_enum());

      let len = vector::length(list_pending);
      let i = 0;

      while (i < len) {
        let ballot = vector::borrow(list_pending, i);
        let ballot_data = ballot::get_type_struct(ballot);

        if (turnout_tally::get_tally_data(ballot_data) == data) return false;

        i = i + 1;
      };
      true
    }

    /// For a Donor Voice account get the total number of votes enrolled from reading the Cumulative tracker.
    fun get_enrollment(dv_account: address): u64 {
      cumulative_deposits::get_cumulative_deposits(dv_account)
    }

    //////// GOV ACTIONS ////////

    /// Reauth tally only.
    public(friend) fun vote_reauthorize(donor: &signer, guid_capability: &account::GUIDCapability, approve: bool) acquires Governance {
      let multisig_address = account::get_guid_capability_address(guid_capability);
      if (is_reauth_proposed(multisig_address)) {
        // Get the ballot ID if available
        let ballot_id = get_proposed_reauth_ballot(multisig_address);

        let res = vote_gov<Reauth>(donor, multisig_address, ballot_id, approve);

        // if tally closes and reauth is true
        if (option::is_some(&res) && *option::borrow(&res)) {
          donor_voice_reauth::reauthorize_now(guid_capability);
        };

        maybe_tally_and_complete<Reauth>(multisig_address, ballot_id);
      } else {
        // go ahead and propose it
        maybe_propose_reauth(guid_capability);
      }
    }

    /// Liquidation voting only. The handler for liquidation exists in donor_voice_tx
    public(friend) fun vote_liquidation(donor: &signer, multisig_address: address): Option<bool> acquires Governance {
      // Check if liquidation is already proposed and get the first ballot ID
      if (!is_liquidation_proposed(multisig_address)) {
        return option::none<bool>()
      };

      // Get the ballot ID - there should only be one pending liquidation ballot at a time
      let (pending_ids, _, _) = get_liquidation_ballots(multisig_address);
      assert!(!vector::is_empty(&pending_ids), error::invalid_argument(ENO_PENDING_BALLOT_FOUND));
      let ballot_id = *vector::borrow(&pending_ids, 0);

      vote_gov<Liquidate>(donor, multisig_address, ballot_id, true)
    }

    //////// TX HELPERS /////////

    // Check if the user is a donor to the multisig address.
    public(friend) fun assert_is_voter(sig: &signer, dv_account: address) {
      let user = signer::address_of(sig);
      assert!(check_is_donor(dv_account, user), error::permission_denied(ENOT_A_DONOR));
    }

    /// only Donor Voice can call this. The veto and liquidate handlers need
    /// to be located there. So users should not call functions here.
    public(friend) fun propose_veto(
      cap: &account::GUIDCapability,
      guid: &guid::ID, // Id of initiated transaction.
      epochs_duration: u64
    ): guid::ID acquires Governance {
      let data = Veto { guid: *guid };
      propose_gov<Veto>(cap, data, epochs_duration)
    }

    public(friend) fun propose_liquidate(
      cap: &account::GUIDCapability,
      epochs_duration: u64
    ): guid::ID acquires Governance {
      let data = Liquidate {};
      propose_gov<Liquidate>(cap, data, epochs_duration)
    }

    /// reauthorization cannot be proposed arbitrarily by a donor
    /// it must be flagged by a policy issue (e.g. doesn't pay back advances)
    /// must be inactive for a year
    /// the reauthorization window/mandate of 5 years expired.
    public(friend) fun maybe_propose_reauth(
      cap: &account::GUIDCapability,
    ): guid::ID acquires Governance {
      let data = Reauth {};
      let dv_address = account::get_guid_capability_address(cap);
      assert!(!donor_voice_reauth::is_authorized(dv_address), error::invalid_argument(EREAUTH_NOT_REQUIRED));
      propose_gov<Reauth>(cap, data, REAUTH_TALLY_EXPIRES)
    }


    // NOTE: Veto has a different workflow than the other two. The veto is proposed by a transaction, and the transaction ID is passed to the proposal.
    // there can be multiple vetoes in pending state in parallel, so we need to check if there is a veto for this transaction uuid.

    // with a known transaction uid, scan all the pending vetoes to see if there is a veto for that transaction, and what the index is.
    // NOTE: what is being returned is a different ID, that of the proposal to veto
    public(friend) fun find_tx_veto_id(tx_id: guid::ID): (bool, guid::ID) acquires Governance {
      let dv_account = guid::id_creator_address(&tx_id);
      let state = borrow_global_mut<Governance<TurnoutTally<Veto>>>(dv_account);

      let pending = ballot::get_list_ballots_by_enum(&state.tracker, ballot::get_pending_enum());
      let i = 0;
      while (i < vector::length(pending)) {
        let a_ballot = vector::borrow(pending, i);
        let turnout_tally = ballot::get_type_struct(a_ballot);
        let proposed_veto = turnout_tally::get_tally_data(turnout_tally);

        let res = comparator::compare(&proposed_veto.guid, &tx_id);

        if (comparator::is_equal(&res)) {
          return (true, ballot::get_ballot_id(a_ballot))
        };
        i = i + 1;
      };

      (false, guid::create_id(@0x1, 0))
    }

    /// Public script transaction to propose a veto, or vote on it if it already exists.
    /// should only be called by the Donor Voice.move so that the handlers can be called on "pass" conditions.
    public(friend) fun veto_by_ballot_id(user: &signer, ballot_guid: &guid::ID): Option<bool> acquires Governance {
      let dv_account = guid::id_creator_address(ballot_guid);
      assert_is_voter(user, dv_account);
      let ballot_id = guid::id_creation_num(ballot_guid);
      // Use the ID we already have instead of searching for it

      vote_gov<Veto>(user, dv_account, ballot_id, true);
      maybe_tally_and_complete<Veto>(dv_account, ballot_id)
    }

    /// The veto process for a donor voice transactions
    /// might get out of sync with the payment instruction
    /// expiration. This function will sync the two.
    public(friend) fun sync_ballot_and_tx_expiration(user: &signer, ballot_guid: &guid::ID, epoch_deadline: u64) acquires Governance {
      let dv_account = guid::id_creator_address(ballot_guid);
      assert_is_voter(user, dv_account);

      let state = borrow_global_mut<Governance<TurnoutTally<Veto>>>(dv_account);

      let ballot = ballot::get_ballot_by_id_mut(&mut state.tracker, ballot_guid);
      let tally_state = ballot::get_type_struct_mut(ballot);

      turnout_tally::extend_deadline(tally_state, epoch_deadline);
    }

    /// Helper function to get all pending ballot IDs for a specific governance type
    fun get_ballot_ids<T: drop + store>(dv_account: address, status_enum: u8 ): vector<u64> acquires Governance {
      let state = borrow_global<Governance<TurnoutTally<T>>>(dv_account);
      let pending_list = ballot::get_list_ballots_by_enum(&state.tracker, status_enum);

      let result = vector::empty<u64>();
      let i = 0;
      let len = vector::length(pending_list);

      while (i < len) {
        let ballot = vector::borrow(pending_list, i);
        let guid = ballot::get_ballot_id(ballot);
        // Get the ID number directly from the ID
        let id_num = guid::id_creation_num(&guid);
        vector::push_back(&mut result, id_num);
        i = i + 1;
      };

      result
    }

    #[view]
    /// gets all the veto ballots in each state
    /// returns pending, approved, rejected
    public fun get_all_ballots_of_type<T: drop + store>(dv_account: address): (vector<u64>, vector<u64>, vector<u64>)  acquires Governance {
      let pending_list = get_ballot_ids<T>(dv_account, ballot::get_pending_enum());
      let approved_list = get_ballot_ids<T>(dv_account, ballot::get_approved_enum());
      let rejected_list = get_ballot_ids<T>(dv_account, ballot::get_rejected_enum());

      (pending_list, approved_list, rejected_list)
    }

    //////// VIEWS ////////

    #[view]
    /// view function to check that a user account is a Donor for a Donor Voice account.
    public fun check_is_donor(dv_account: address, user: address): bool {
      reauthorization::assert_v8_authorized(user);

      // Only as high as I reach can I grow
      // Only as far as I seek can I go
      // Only as deep as I look can I see
      // Only as much as I dream can I be

      get_user_donations(dv_account, user) > 0
    }

    #[view]
    /// For an individual donor, get the amount of votes that they can cast, based on their cumulative donations to the Donor Voice account.
    public fun get_user_donations(dv_account: address, user: address): u64 {
      let (_, _, cumulative_donations) = receipts::read_receipt(user, dv_account);
      cumulative_donations
    }


    //////// VETO VIEWS ////////

    #[view]
    /// gets all the veto ballots in each state
    /// returns pending, approved, rejected
    public fun get_veto_ballots(dv_account: address): (vector<u64>, vector<u64>, vector<u64>)  acquires Governance {
      get_all_ballots_of_type<Veto>(dv_account)
    }

    #[view]
    // for a transaction UID return if the address and proposal ID have any vetoes. guid::ID is destructured for view functions
    public fun tx_has_veto_pending(dv_account: address, tx_id: u64): bool acquires Governance {
      let tx_uid = guid::create_id(dv_account, tx_id);
      let (found, _) = find_tx_veto_id(tx_uid);
      found
    }

    #[view]
    // returns the deadline (in epochs) for a veto vote
    public fun get_veto_deadline(dv_account: address, tx_id: u64): u64 acquires Governance {
      deadline_gov<Veto>(dv_account, &Veto { guid: guid::create_id(dv_account, tx_id) })
    }

    #[view]
    /// Returns the details of a veto vote for the specified transaction.
    ///
    /// # Arguments
    ///
    /// * `dv_account` - The Donor Voice account address
    /// * `ballot_id` - The ballot ID number
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `approval_pct` - The percentage (0-100) of votes that approved the proposal
    /// * `turnout_pct` - The percentage (0-100) of eligible voters who participated
    /// * `required_threshold` - The minimum approval percentage required to pass
    /// * `epoch_deadline` - The epoch by which voting must be completed
    /// * `minimum_turnout` - The minimum percentage of eligible voters that must participate
    /// * `approved` - Whether the proposal has reached the approval threshold
    /// * `is_complete` - Whether the voting process has been marked as complete
    /// * `status_enum` - The current status of the ballot (pending, approved, rejected)
    /// * `ballot_completed` - Whether the ballot itself has been marked as completed
    ///
    /// # Errors
    ///
    /// * `ENO_BALLOT_FOUND` - If no veto ballot exists for the given ballot ID
    public fun get_veto_tally(dv_account: address, ballot_id: u64): (u64, u64, u64, u64, u64, bool, bool, u8, bool) acquires Governance {
      let guid = guid::create_id(dv_account, ballot_id);
      tally_gov<Veto>(dv_account, guid)
    }

    public fun get_veto_tally_by_tx_id(dv_account: address, tx_id: u64): (u64, u64, u64, u64, u64, bool, bool, u8, bool) acquires Governance {
      let tx_guid = guid::create_id(dv_account, tx_id);
      // First find the ballot GUID associated with this transaction
      let (found, ballot_guid) = find_tx_veto_id(tx_guid);
      assert!(found, error::invalid_argument(ENO_BALLOT_FOUND));
      get_veto_tally(dv_account, guid::id_creation_num(&ballot_guid))
    }

    /////// REAUTH VIEWS ////////

    #[view]
    /// gets all the veto ballots in each state
    /// returns pending, approved, rejected
    public fun get_reauth_ballots(dv_account: address): (vector<u64>, vector<u64>, vector<u64>)  acquires Governance {
      get_all_ballots_of_type<Reauth>(dv_account)
    }

    #[view]
    /// gets a current pending reauth ballot, or aborts
    public fun get_proposed_reauth_ballot(dv_account: address): u64 acquires Governance {
      let state = borrow_global<Governance<TurnoutTally<Reauth>>>(dv_account);
      let list_pending = ballot::get_list_ballots_by_enum(&state.tracker, ballot::get_pending_enum());
      // there should never be more than one pending reauth ballot
      assert!(vector::length(list_pending) == 1, error::invalid_argument(ENO_PENDING_BALLOT_FOUND));

      let ballot = vector::borrow(list_pending, 0);
      let guid = ballot::get_ballot_id(ballot);

      guid::id_creation_num(&guid)
    }

    #[view]
    /// gets a current pending reauth ballot, or aborts
    public fun is_reauth_proposed(dv_account: address): bool acquires Governance {
      let state = borrow_global<Governance<TurnoutTally<Reauth>>>(dv_account);
      let list_pending = ballot::get_list_ballots_by_enum(&state.tracker, ballot::get_pending_enum());
      vector::length(list_pending) > 0
    }

    #[view]
    /// Returns the details of a vote result as a tuple.
    ///
    /// # Arguments
    ///
    /// * `dv_account` - The Donor Voice account address
    /// * `id` - The ID number of the reauthorization ballot to tally
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `approval_pct` - The percentage (0-100) of votes that approved the proposal
    /// * `turnout_pct` - The percentage (0-100) of eligible voters who participated
    /// * `required_threshold` - The minimum approval percentage required to pass
    /// * `epoch_deadline` - The epoch by which voting ends
    /// * `minimum_turnout` - The minimum percentage of eligible voters that must participate
    /// * `approved` - Whether the proposal has reached the approval threshold
    /// * `is_complete` - Whether the voting process has been marked as complete
    /// * `status_enum` - The current status of the ballot (pending, approved, rejected)
    /// * `ballot_completed` - Whether the ballot itself has been marked as completed
    ///
    /// # Errors
    ///
    /// * `ENO_BALLOT_FOUND` - If no reauth ballot exists with the given ID
    public fun get_reauth_tally(dv_account: address, ballot_id: u64): (u64, u64, u64, u64, u64, bool, bool, u8, bool) acquires Governance {
      let ballot_guid = guid::create_id(dv_account, ballot_id);
      tally_gov<Reauth>(dv_account, ballot_guid)
    }

    #[view]
    /// returns the deadline (in epochs) for a reauthorization vote
    public fun get_reauth_expiry(dv_account: address): u64 acquires Governance {
      deadline_gov<Reauth>(dv_account, &Reauth {})
    }

    #[view]
    /// should produce a useful error if cannot pass
    public fun reauth_would_pass(dv_account: address, ballot_id: u64): bool acquires Governance {
      assert_pass_gov<Reauth>(dv_account, ballot_id);
      true
    }

    /////// LIQ VIEWS ////////

    #[view]
    /// gets all the veto ballots in each state
    /// returns pending, approved, rejected
    public fun get_liquidation_ballots(dv_account: address): (vector<u64>, vector<u64>, vector<u64>)  acquires Governance {
      get_all_ballots_of_type<Liquidate>(dv_account)
    }

    #[view]
    public fun is_liquidation_proposed(dv_account: address): bool acquires Governance {
      let state = borrow_global<Governance<TurnoutTally<Liquidate>>>(dv_account);
      let list_pending = ballot::get_list_ballots_by_enum(&state.tracker, ballot::get_pending_enum());
      vector::length(list_pending) > 0
    }

    #[view]
    /// Show the status of a liquidation tally.
    ///
    /// # Arguments
    ///
    /// * `dv_account` - The Donor Voice account address
    /// * `id` - The ID number of the liquidation ballot to tally
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `approval_pct` - The percentage (0-100) of votes that approved the proposal
    /// * `turnout_pct` - The percentage (0-100) of eligible voters who participated
    /// * `required_threshold` - The minimum approval percentage required to pass
    /// * `epoch_deadline` - The epoch by which voting ends
    /// * `minimum_turnout` - The minimum percentage of eligible voters that must participate
    /// * `approved` - Whether the proposal has reached the approval threshold
    /// * `is_complete` - Whether the voting process has been marked as complete
    /// * `status_enum` - The current status of the ballot (pending, approved, rejected)
    /// * `ballot_completed` - Whether the ballot itself has been marked as completed
    ///
    /// # Errors
    ///
    /// * `ENO_BALLOT_FOUND` - If no liquidation ballot exists with the given ID
    public fun get_liquidation_tally(dv_account: address, id: u64): (u64, u64, u64, u64, u64, bool, bool, u8, bool) acquires Governance {
      let ballot_guid = guid::create_id(dv_account, id);
      tally_gov<Liquidate>(dv_account, ballot_guid)
    }

    #[view]
    // returns the deadline (in epochs) for a liquidation vote
    public fun get_liquidation_deadline(dv_account: address): u64 acquires Governance {
      deadline_gov<Liquidate>(dv_account, &Liquidate {})
    }
}
