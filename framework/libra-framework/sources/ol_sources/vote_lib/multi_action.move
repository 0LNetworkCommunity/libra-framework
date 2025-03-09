///////////////////////////////////////////////////////////////////////////
// 0L Module
// MultiAction
// A payment tool for transfers which require n-of-m approvals
///////////////////////////////////////////////////////////////////////////


// MultiAction is part of the Vote tooling (aka DAO tooling). It's a module which allows for a group of authorities to approve a generic action, and on a successful vote return account capabilities to a calling third party contract. The Action type can be defined by a third party smart contract, and the MultiAction module will only check if the Action has been approved by the required number of authorities.

// The goal is for MultiAction to be composable. For example an 0L "root service" called Safe, uses MultiAction to create a simple abstraction for a payment multisig. DonorDirected is similar to Safe, and has more rules for paying out (timed transfers).

// This is a type of multisig that can be programmable by other on-chain contracts. Previously in V6 we used to call this MultiSig. However platform vendor introduced a new (and great) multisig feature. Both of these can coexist as they have different purposes.

// The module MultiAction allows the sponsor to propose authorities for a future multisig account, which must be claimed by each designated authority. Once enough claims are received, the sponsor can proceed to set up the multisig account. After the account becomes multisig, any changes to authorities must be voted on.

// With vendor::MultiSig, the action needs to be constructed offline using a script. There are advantages to this. Anything that can be written into a script can be made to execute by the authorities. However the code is not inspectable, it's stored as bytecode. MultiAction, on the other hand requires that all the logic be written in a deployed contract. The execution of the action depends on the third-party published contract. MultiAction will only return the Capabilities necessary for the smart contract to do what it needs, e.g. a simple passed/rejected result, an optional WithdrawCapability, and and optional (dangerous) SignerCapability (tbd as of V7).

// Similarly any handler for the Action can be executed by an external contract, and the Governance module will only check if the Action has been approved by the required number of authorities.
// Each Action has a separate data structure for tabulating the votes in approval of the Action. But there is shared state between the Actions, that being Governance, which contains the constraints for each Action that are checked on each vote (n_sigs, expiration, signers, etc)
// The Actions are triggered "lazily", that is: the last authorized sender of a proposal/vote, is the one to trigger the action.
// Theere is no offline signature aggregation. The authorities over the address should not require collecting signatures offline: proposal should be submitted directly to this contract.

// With this design, the multisig can be used for different actions. The safe.move contract is an example of a Root Service which the chain provides, which leverages the Governance module to provide a payment service which requires n-of-m approvals.

// V7 NOTE: from V6 we are refactoring so the the account first needs to be created as a "resource account". It's a minor change given that V6 had a similar construct of a "signerless account", Previously in 0L this meant to "Brick" the authkey after the WithdrawCapability was stored in a common struct. Vendor had independenly made the same design using Signer Capability.

module ol_framework::multi_action {
    use std::vector;
    use std::option::{Self, Option};
    use std::signer;
    use std::error;
    use std::guid;
    use diem_framework::multisig_account;
    use diem_framework::account::{Self, WithdrawCapability};
    use ol_framework::ballot::{Self, BallotTracker};
    use ol_framework::epoch_helper;
    use ol_framework::community_wallet;

    // use diem_std::debug::print;

    friend ol_framework::community_wallet_init;
    friend ol_framework::donor_voice_txs;
    friend ol_framework::safe;

    // TODO: Remove after migration
    friend ol_framework::multi_action_migration;

    #[test_only]
    friend ol_framework::test_multi_action_migration; // TODO: remove after offer migration

    #[test_only]
    friend ol_framework::test_multi_action;
    #[test_only]
    friend ol_framework::test_community_wallet;

    const EGOV_NOT_INITIALIZED: u64 = 1;
    /// The owner of this account can't be an authority, since it will subsequently be bricked. The signer of this account is no longer useful. The account is now controlled by the Governance logic.
    const ESIGNER_CANT_BE_AUTHORITY: u64 = 2;
    /// signer not authorized to approve a transaction.
    const ENOT_AUTHORIZED: u64 = 3;
    /// There are no pending transactions to search
    const EPENDING_EMPTY: u64 = 4;
    /// Not enough signers configured
    const ENO_SIGNERS: u64 = 5;
    /// The multisig setup  is not finalized, the sponsor needs to brick their authkey. The account setup sponsor needs to be verifiably locked out before operations can begin.
    const ENOT_FINALIZED_NOT_BRICK: u64 = 6;
    /// Already registered this action type
    const EACTION_ALREADY_EXISTS: u64 = 7;
    /// Action not found
    const EACTION_NOT_FOUND: u64 = 8;
    /// Proposal is expired
    const EPROPOSAL_EXPIRED: u64 = 9;
    /// Proposal is expired
    const EDUPLICATE_PROPOSAL: u64 = 10;
    /// Proposal is expired
    const EPROPOSAL_NOT_FOUND: u64 = 11;
    /// Proposal voting is closed
    const EVOTING_CLOSED: u64 = 12;
    /// No addresses in multisig changes
    const EEMPTY_ADDRESSES: u64 = 13;
    /// Duplicate vote
    const EDUPLICATE_VOTE: u64 = 14;
    /// Offer expired
    const EOFFER_EXPIRED: u64 = 15;
    /// Offer empty
    const EOFFER_EMPTY: u64 = 16;
    /// Not offered to initial authorities
    const ENOT_OFFERED: u64 = 17;
    /// Not enough claimed authorities
    const ENOT_ENOUGH_CLAIMED: u64 = 18;
    /// Account is already a multisig
    const EALREADY_MULTISIG: u64 = 19;
    /// Address not proposed for authority role
    const EADDRESS_NOT_PROPOSED: u64 = 20;
    /// Address proposed for authority role does not exist
    const EPROPOSED_NOT_EXISTS: u64 = 21;
    /// Offer duration must be greater than zero
    const EZERO_DURATION: u64 = 22;
    /// Offer already claimed
    const EALREADY_CLAIMED: u64 = 23;
    /// Too many addresses in offer - avoid DoS attack
    const ETOO_MANY_ADDRESSES: u64 = 24;
    /// Offer already exists
    const EOFFER_ALREADY_EXISTS: u64 = 25;
    /// Already an owner
    const EALREADY_OWNER: u64 = 26;
    /// Owner not found
    const EOWNER_NOT_FOUND: u64 = 27;
    /// Community wallet account
    const ECW_ACCOUNT: u64 = 28;

    /// default setting for a proposal to expire
    const DEFAULT_EPOCHS_EXPIRE: u64 = 14;
    /// default setting for an offer to expire
    const DEFAULT_EPOCHS_OFFER_EXPIRE: u64 = 7;
    /// minimum number of claimed authorities to cage the account
    const MIN_OFFER_CLAIMS_TO_CAGE: u64 = 2;
    /// maximum number of address to offer
    const MAX_OFFER_ADDRESSES: u64 = 10;

    /// A Governance account is an account which requires multiple votes from Authorities to  send a transaction.
    /// A multisig can be used to get agreement on different types of Actions, such as a payment transaction where the handler code for the transaction is an a separate contract. See for example MultiSigPayment.
    /// Governance struct holds the metadata for all the instances of Actions on this account.
    /// Every action has the same set of authorities and governance.
    /// This is intentional, since privilege escalation can happen if each action has a different set of governance, but access to funds and other state.
    /// If the organization wishes to have Actions with different governance, then a separate Account is necessary.


    /// DANGER
    /// Governance optionally holds a WithdrawCapability, which is used to withdraw funds from the account. All actions share the same WithdrawCapability.
    /// The WithdrawCapability can be used to withdraw funds from the account.
    /// Ordinarily only the signer/owner of this address can use it.
    /// We are bricking the signer, and as such the withdraw capability is now controlled by the Governance logic.
    /// Core Devs: This is a major attack vector. The WithdrawCapability should NEVER be returned to a public caller, UNLESS it is within the vote and approve flow.

    /// Note, the WithdrawCapability is moved to this shared structure, and as such the signer of the account is bricked. The signer who was the original owner of this account ("sponsor") can no longer issue transactions to this account, and as such the WithdrawCapability would be inaccessible. So on initialization we extract the WithdrawCapability into the Governance governance struct.

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
        // The transaction to be executed
        proposal_data: ProposalData,
        // The votes received
        votes: vector<address>,
        // approved
        approved: bool,
        // The expiration time for the transaction
        expiration_epoch: u64,
    }

    /// Offer struct to manage the proposal and claiming of new authorities.
    /// - proposed: List of authority addresses proposed
    /// - claimed: List of authority addresses that have claimed the offer.
    /// - expiration_epoch: The epoch when each proposed expires.
    /// - proposed_n_of_m: The n-of-m threshold for the account. Used only after account is cage.
    struct Offer has key, store {
        proposed: vector<address>,
        claimed: vector<address>,
        expiration_epoch: vector<u64>,
        proposed_n_of_m: Option<u64>,
    }

    fun construct_empty_offer(): Offer {
        Offer {
            proposed: vector::empty(),
            claimed: vector::empty(),
            expiration_epoch: vector::empty(),
            proposed_n_of_m: option::none(),
        }
    }

    fun clean_offer(addr: address) acquires Offer {
        let offer = borrow_global_mut<Offer>(addr);
        offer.proposed = vector::empty();
        offer.claimed = vector::empty();
        offer.expiration_epoch = vector::empty();
        offer.proposed_n_of_m = option::none();
    }

    public(friend) fun init_offer(sig: &signer, addr: address) {
        if (!exists<Offer>(addr)) {
            move_to(sig, construct_empty_offer());
        };
    }

    // Initialize the governance structs for this account.
    // Governance contains the constraints for each Action that are checked on each vote (n_sigs, expiration, signers, etc)
    // Also, an initial Action of type PropGovSigners is created, which is used to govern the signers and threshold for this account.
    public(friend) fun init_gov(sig: &signer) {
        // heals un-initialized state, and does nothing if state already exists.

        let multisig_address = signer::address_of(sig);
        // User footgun. The signer of this account is bricked, and as such the signer can no longer be an authority.

        if (!exists<Governance>(multisig_address)) {
            move_to(sig, Governance {
                cfg_duration_epochs: DEFAULT_EPOCHS_EXPIRE,
                cfg_default_n_sigs: 0, // deprecate
                signers: vector::empty(),
                withdraw_capability: option::none(),
                guid_capability: account::create_guid_capability(sig),
            });
        };

        if (!exists<Action<PropGovSigners>>(multisig_address)) {
            move_to(sig, Action<PropGovSigners> {
                can_withdraw: false,
                vote: ballot::new_tracker<Proposal<PropGovSigners>>(),
            });
        };

        init_offer(sig, multisig_address);
    }

    fun ensure_valid_propose_offer_state(addr: address) {
        // Ensure the account is not yet initialized as multisig
        assert!(!multisig_account::is_multisig(addr), error::invalid_state(EALREADY_MULTISIG));

        // Ensure the account has governance initialized and offer structure
        assert!(is_gov_init(addr), error::invalid_state(EGOV_NOT_INITIALIZED));
        assert!(exists_offer(addr), error::already_exists(EOFFER_ALREADY_EXISTS));
    }

    fun ensure_valid_propose_offer_params(addr: address, proposed: vector<address>, duration_epochs: Option<u64>) {
        // Ensure the proposed list is not empty
        assert!(vector::length(&proposed) > 0, error::invalid_argument(EOFFER_EMPTY));

        // Ensure the proposed list is not greater than the maximum limit - avoid DoS attack
        assert!(vector::length(&proposed) <= MAX_OFFER_ADDRESSES, error::invalid_argument(ETOO_MANY_ADDRESSES));

        // Ensure distinct addresses and multisign owner not in the list
        multisig_account::validate_owners(&proposed, addr);

        if (option::is_some(&duration_epochs)) {
        let duration_epochs = *option::borrow(&duration_epochs);
        // Ensure duration is greater than zero
        assert!(duration_epochs > 0, error::invalid_argument(EZERO_DURATION));
        };
    }

    // Propose an offer to new authorities on the signer account
    // or update the expiration epoch of the existing proposed authorities.
    // - sig: The signer proposing the offer.
    // - proposed: The list of authorities addresses proposed.
    // - duration_epochs: The duration in epochs before the offer expires.
    public fun propose_offer(sig: &signer, proposed: vector<address>, duration_epochs: Option<u64>) acquires Offer {
        // Propose the offer on the signer's account
        let addr = signer::address_of(sig);

        // Ensure the account is not community wallet
        // Community wallet has its own propose_offer function
        assert!(community_wallet::is_init(addr) == false, error::invalid_argument(ECW_ACCOUNT));

        propose_offer_internal(sig, proposed, duration_epochs);
    }

    public(friend) fun propose_offer_internal(sig: &signer, proposed: vector<address>, duration_epochs: Option<u64>) acquires Offer {
        let addr = signer::address_of(sig);
        ensure_valid_propose_offer_state(addr);
        ensure_valid_propose_offer_params(addr, proposed, duration_epochs);
        update_offer(addr, &mut proposed, duration_epochs);
    }

     // Update the offer with the new proposed authorities and expiration epoch.
    fun update_offer(addr: address, proposed: &mut vector<address>, duration_epochs: Option<u64>) acquires Offer {
        let offer = borrow_global_mut<Offer>(addr);

        // step 0
        let expiration_epoch = calculate_expiration_epoch(duration_epochs);

        // step 1
        remove_claimed_not_in_new_proposed(offer, proposed);

        // step 2
        remove_new_proposed_addresses_already_claimed(offer, proposed);

        // step 3
        remove_old_proposed_not_in_new_proposed(offer, proposed);

        // step 4
        upsert_new_proposed_and_expiration_epoch(offer, proposed, expiration_epoch);
    }

    // update_offer: step 0
    fun calculate_expiration_epoch(duration_epochs: Option<u64>): u64 {
        let duration_epochs = if (option::is_some(&duration_epochs)) {
            *option::borrow(&duration_epochs)
        } else {
            DEFAULT_EPOCHS_OFFER_EXPIRE
        };

        epoch_helper::get_current_epoch() + duration_epochs
    }

    // update_offer: step 1
    fun remove_claimed_not_in_new_proposed(offer: &mut Offer, proposed: &vector<address>) {
        let j = 0;
        while (j < vector::length(&offer.claimed)) {
            let claimed_addr = vector::borrow(&offer.claimed, j);
            if (!vector::contains(proposed, claimed_addr)) {
                vector::remove(&mut offer.claimed, j);
            } else {
                j = j + 1;
            };
        };
    }

    // update_offer: step 2
    fun remove_new_proposed_addresses_already_claimed(offer: &mut Offer, proposed: &mut vector<address>) {
        let i = 0;
        while (i < vector::length(proposed)) {
            let proposed_addr = vector::borrow(proposed, i);
            if (vector::contains(&offer.claimed, proposed_addr)) {
                vector::remove(proposed, i);
            };
            i = i + 1;
        };
    }

    // update_offer: step 3
    fun remove_old_proposed_not_in_new_proposed(offer: &mut Offer, proposed: &vector<address>) {
        let j = 0;
        while (j < vector::length(&offer.proposed)) {
            let proposed_addr = vector::borrow(&offer.proposed, j);
            if (!vector::contains(proposed, proposed_addr)) {
                vector::remove(&mut offer.proposed, j);
                vector::remove(&mut offer.expiration_epoch, j);
            } else {
                j = j + 1;
            };
        };
    }

    // update_offer: step 4
    fun upsert_new_proposed_and_expiration_epoch(offer: &mut Offer, proposed: &vector<address>, expiration_epoch: u64) {
        let i = 0;
        while (i < vector::length(proposed)) {
            let proposed_addr = vector::borrow(proposed, i);
            let (found, j) = vector::index_of(&offer.proposed, proposed_addr);
            if (found) {
                vector::remove(&mut offer.expiration_epoch, j);
                vector::insert(&mut offer.expiration_epoch, j, expiration_epoch);
            } else {
                vector::push_back(&mut offer.proposed, *proposed_addr);
                vector::push_back(&mut offer.expiration_epoch, expiration_epoch);
            };
            i = i + 1;
        };
    }

    // Allows a proposed authority to claim their offer.
    // - sig: The signer making the claim.
    // - multisig_address: The address of the multisig account.
    public entry fun claim_offer(sig: &signer, multisig_address: address) acquires Offer, Governance {
        let sender_addr = signer::address_of(sig);

        validate_claim_offer(multisig_address, sender_addr);

        let offer = borrow_global_mut<Offer>(multisig_address);

        // Remove the sender from the proposed list and expiration_epoch
        let (_, i) = vector::index_of(&offer.proposed, &sender_addr);
        vector::remove(&mut offer.proposed, i);
        vector::remove(&mut offer.expiration_epoch, i);

        if (multisig_account::is_multisig(multisig_address)) {
            // a) finalized account: add authority to the multisig account
            let gov = borrow_global_mut<Governance>(multisig_address);
            maybe_update_authorities(gov, true, &vector::singleton(sender_addr));
            if (vector::length(&offer.proposed) == 0) {
                // Update voted n_of_m after all authorities claimed
                let n_of_m = offer.proposed_n_of_m;
                let _ = offer;
                maybe_update_threshold(multisig_address, gov, &n_of_m);

                // clean the Offer
                clean_offer(multisig_address);
            };
        } else {
            // b) initiated account: add sender to the claimed list
            vector::push_back(&mut offer.claimed, sender_addr);
        };
    }

    // Validate account state and parameters to claim the offer.
    fun validate_claim_offer(multisig_address: address, sender_addr: address) acquires Offer{
        // Ensure the account has an offer
        assert!(exists_offer(multisig_address), error::not_found(ENOT_OFFERED));

        // Ensure the offer has not expired
        assert!(!is_offer_expired(multisig_address, sender_addr), error::out_of_range(EOFFER_EXPIRED));

        let offer = borrow_global<Offer>(multisig_address);

        // Ensure the sender is not in the claimed list
        assert!(!vector::contains(&offer.claimed, &sender_addr), error::already_exists(EALREADY_CLAIMED));

        // Ensure the sender is in the proposed list
        assert!(vector::contains(&offer.proposed, &sender_addr), error::not_found(EADDRESS_NOT_PROPOSED));
    }

    /// Finalizes the multisign account and locks it (cage).
    /// - sig: The signer finalizing the account.
    /// - num_signers: The number of signers required to approve a transaction.
    /// Aborts if governance is not initialized, the account is already a multisig,
    /// there are not enough claimed authorities, or the offer is not found.
    public fun finalize_and_cage(sig: &signer, num_signers: u64) acquires Offer {
        let addr = signer::address_of(sig);

        // check it is not yet initialized
        assert!(!multisig_account::is_multisig(addr), error::already_exists(EALREADY_MULTISIG));

        // check governance
        assert!(exists<Governance>(addr), error::invalid_state(EGOV_NOT_INITIALIZED));
        assert!(exists<Action<PropGovSigners>>(addr), error::invalid_state(EGOV_NOT_INITIALIZED));

        // check claimed authorities
        assert!(exists_offer(addr), error::not_found(ENOT_OFFERED));
        assert!(has_enough_offer_claimed(addr), error::invalid_state(ENOT_ENOUGH_CLAIMED));

        // finalize the account
        let initial_authorities = get_offer_claimed(addr);
        multisig_account::migrate_with_owners(sig, initial_authorities, num_signers, vector::empty(), vector::empty());

        // clean offer
        clean_offer(addr);
    }

    public(friend) fun proposal_constructor<ProposalData: store + drop>(proposal_data: ProposalData, duration_epochs: Option<u64>): Proposal<ProposalData> {

        let duration_epochs = if (option::is_some(&duration_epochs)) {
            *option::borrow(&duration_epochs)
        } else {
            DEFAULT_EPOCHS_EXPIRE
        };

        Proposal<ProposalData> {
            proposal_data,
            votes: vector::empty<address>(),
            approved: false,
            expiration_epoch: epoch_helper::get_current_epoch() + duration_epochs,
        }
    }

    fun assert_authorized(sig: &signer, multisig_address: address) {
        // cannot start manipulating contract until the sponsor gave up the auth key
        assert_multi_action(multisig_address);

        // check sender is authorized
        let sender_addr = signer::address_of(sig);
        assert!(is_authority(multisig_address, sender_addr), error::invalid_argument(ENOT_AUTHORIZED));
    }

    //////// Helper functions to check initialization //////////

    #[view]
    /// Is the Multisig Governance finalized
    public fun is_multi_action(addr: address): bool {
        exists<Governance>(addr) &&
        exists<Action<PropGovSigners>>(addr) &&
        multisig_account::is_multisig(addr)
    }

    // Check if the account has the Governance initialized
    #[view]
    public fun is_gov_init(addr: address): bool {
        exists<Governance>(addr) &&
        exists<Action<PropGovSigners>>(addr)
    }

    /// helper to assert if the account is in the right state
    fun assert_multi_action(addr: address) {
        assert!(multisig_account::is_multisig(addr), error::invalid_state(ENOT_FINALIZED_NOT_BRICK));
        assert!(exists<Governance>(addr), error::invalid_state(EGOV_NOT_INITIALIZED));
        assert!(exists<Action<PropGovSigners>>(addr), error::invalid_state(EGOV_NOT_INITIALIZED));
    }

    // Query if an offer exists for the given multisig address.
    #[view]
    public fun exists_offer(multisig_address: address): bool {
        exists<Offer>(multisig_address)
    }

    // Query proposed authorities for the given multisig address.
    #[view]
    public fun get_offer_proposed(multisig_address: address): vector<address> acquires Offer {
        borrow_global<Offer>(multisig_address).proposed
    }

    // Query claimed authorities for the given multisig address.
    #[view]
    public fun get_offer_claimed(multisig_address: address): vector<address> acquires Offer {
        borrow_global<Offer>(multisig_address).claimed
    }

    // Query offer expiration epoch.
    public fun get_offer_expiration_epoch(multisig_address: address): vector<u64> acquires Offer {
        borrow_global<Offer>(multisig_address).expiration_epoch
    }

    #[view]
    public fun get_offer_proposed_n_of_m(multisig_address: address): Option<u64> acquires Offer {
        borrow_global<Offer>(multisig_address).proposed_n_of_m
    }

    // Query if the offer has enough claimed authorities to cage the account.
    fun has_enough_offer_claimed(multisig_address: address): bool acquires Offer {
        let claimed = get_offer_claimed(multisig_address);
        vector::length(&claimed) >= MIN_OFFER_CLAIMS_TO_CAGE
    }

    // Query if the offer has expired.
    public fun is_offer_expired(multisig_address: address, authority_address: address): bool acquires Offer {
        let offer = borrow_global<Offer>(multisig_address);
        let (_, i) = vector::index_of(&offer.proposed, &authority_address);
        let expiration_epoch = vector::borrow(&offer.expiration_epoch, i);
        epoch_helper::get_current_epoch() >= *expiration_epoch
    }

    /// Has a multisig struct for a given action been created?
    public(friend) fun has_action<ProposalData: store>(addr: address):bool {
        exists<Action<ProposalData>>(addr)
    }

    /// An initial "sponsor" who is the signer of the initialization account calls this function.
    // This function creates the data structures.
    public(friend) fun init_type<ProposalData: store + drop >(
        sig: &signer,
        can_withdraw: bool,
    ) acquires Governance {
        let multisig_address = signer::address_of(sig);
        // TODO: there is no way of creating a new Action by multisig. The "signer" would need to be spoofed, which account does only in specific and scary situations (e.g. vm_create_account_migration)

        assert!(is_gov_init(multisig_address), error::invalid_argument(EGOV_NOT_INITIALIZED));

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

    /// Withdraw cap is a hot-potato and can never be dropped, we can extract and fill it into a struct that holds it.

    public(friend) fun maybe_restore_withdraw_cap(cap_opt: Option<WithdrawCapability>) acquires Governance {
        if (option::is_some(&cap_opt)) {
            let cap = option::extract(&mut cap_opt);
            let addr = account::get_withdraw_cap_address(&cap);
            let ms = borrow_global_mut<Governance>(addr);
            option::fill(&mut ms.withdraw_capability, cap);
        };
        option::destroy_none(cap_opt);
    }


    // Propose an Action
    // Transactions should be easy, and have one obvious way to do it. There should be no other method for voting for a tx.
    // this function will catch a duplicate, and vote in its favor.
    // This causes a user interface issue, users need to know that you cannot have two open proposals for the same transaction.
    // It's optional to state how many epochs from today the transaction should expire. If the transaction is not approved by then, it will be rejected.
    // The default will be 14 days.
    // Only the first proposer can set the expiration time. It will be ignored when a duplicate is caught.

    public(friend) fun propose_new<ProposalData: store + drop>(
        sig: &signer,
        multisig_address: address,
        proposal_data: Proposal<ProposalData>,
    ): guid::ID acquires Governance, Action {
        assert_authorized(sig, multisig_address);
        let ms = borrow_global_mut<Governance>(multisig_address);
        let action = borrow_global_mut<Action<ProposalData>>(multisig_address);
        // go through all proposals and clean up expired ones.
        lazy_cleanup_expired(action);
        // does this proposal already exist in the pending list?
        let (found, guid, _idx, status_enum, _is_complete) = search_proposals_by_data<ProposalData>(&action.vote, &proposal_data);
        if (found && status_enum == ballot::get_pending_enum()) {
            // this exact proposal is already pending, so we we will just return the guid of the existing proposal.
            // we'll let the caller decide what to do (we wont vote by default)
            return guid
        };

        let guid = account::create_guid_with_capability(&ms.guid_capability);
        let ballot = ballot::propose_ballot(&mut action.vote, guid, proposal_data);
        let id = ballot::get_ballot_id(ballot);

        id
    }

    /// helper function to vote with ID only
    public(friend) fun vote_with_id<ProposalData: store + drop>(sig: &signer, id: &guid::ID, multisig_address: address): (bool, Option<WithdrawCapability>) acquires Governance, Action {
        assert_authorized(sig, multisig_address);

        vote_impl<ProposalData>(sig, multisig_address, id)
    }

    // TODO: consider using multisig_account also for voting.
    // currently only used for governance.
    fun vote_impl<ProposalData: store + drop>(
        sig: &signer,
        multisig_address: address,
        id: &guid::ID
    ): (bool, Option<WithdrawCapability>) acquires Governance, Action {

        assert_authorized(sig, multisig_address); // belt and suspenders
        let ms = borrow_global_mut<Governance>(multisig_address);
        let action = borrow_global_mut<Action<ProposalData>>(multisig_address);
        // always run this to cleanup all missing ballots
        lazy_cleanup_expired(action);

        // does this proposal already exist in the pending list?
        let (found, _idx, status_enum, is_complete) = ballot::find_anywhere<Proposal<ProposalData>>(&action.vote, id);
        assert!(found, error::invalid_argument(EPROPOSAL_NOT_FOUND));
        assert!(status_enum == ballot::get_pending_enum(), error::invalid_state(EVOTING_CLOSED));
        assert!(!is_complete, error::invalid_argument(EVOTING_CLOSED));

        let b = ballot::get_ballot_by_id_mut(&mut action.vote, id);

        let t = ballot::get_type_struct_mut(b);
        let voter_addr = signer::address_of(sig);
        // prevent duplicates
        assert!(!vector::contains(&t.votes, &voter_addr),
        error::invalid_argument(EDUPLICATE_VOTE));

        vector::push_back(&mut t.votes, voter_addr);
        let (n, _m) = get_threshold(multisig_address);
        let passed = tally(t, n);

        if (passed) {
            ballot::complete_ballot(b);
            ballot::move_ballot(
                &mut action.vote,
                id,
                ballot::get_pending_enum(),
                ballot::get_approved_enum()
            );
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

        (passed, withdraw_cap)
    }


    // @returns bool, complete and passed
    // TODO: Multi_action will never pass a complete and rejected, which needs a UX
    fun tally<ProposalData: store + drop>(prop: &mut Proposal<ProposalData>, n: u64): bool {
        if (vector::length(&prop.votes) >= n) {
            prop.approved = true;
            return true
        };
        false
    }

    fun find_expired<ProposalData: store + drop>(a: & Action<ProposalData>): vector<guid::ID>{
        let epoch = epoch_helper::get_current_epoch();
        let b_vec = ballot::get_list_ballots_by_enum(&a.vote, ballot::get_pending_enum());
        let id_vec = vector::empty();
        let i = 0;
        while (i < vector::length(b_vec)) {
            let b = vector::borrow(b_vec, i);
            let t = ballot::get_type_struct<Proposal<ProposalData>>(b);

            if (epoch > t.expiration_epoch) {
                let id = ballot::get_ballot_id(b);
                vector::push_back(&mut id_vec, id);

            };
            i = i + 1;
        };

        id_vec
    }

    fun lazy_cleanup_expired<ProposalData: store + drop>(a: &mut Action<ProposalData>) {
        let expired_vec = find_expired(a);
        let len = vector::length(&expired_vec);
        let i = 0;
        while (i < len) {
            let id = vector::borrow(&expired_vec, i);
            // lets check the status just in case.
            ballot::move_ballot(&mut a.vote, id, ballot::get_pending_enum(), ballot::get_rejected_enum());
            i = i + 1;
        };
    }

    fun check_expired<ProposalData: store>(prop: &Proposal<ProposalData>): bool {
        let epoch_now = epoch_helper::get_current_epoch();
        epoch_now > prop.expiration_epoch
    }

    /// This function is used to copy the data from the proposal that is in the multisig.
    /// Note that this is the only way to get the data out of the multisig, and it is the only function to use the `copy` trait. If you have a workflow that needs copying, then the data struct for the action payload will need to use the `copy` trait.
    public(friend) fun extract_proposal_data<ProposalData: store + copy + drop>(multisig_address: address, uid: &guid::ID): ProposalData acquires Action {
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

    /// returns a tuple of (is_found: bool, id: guid:ID, index: u64, status_enum: u8, is_complete: bool)
    fun search_proposals_by_data<ProposalData: drop + store> (
        tracker: &BallotTracker<Proposal<ProposalData>>,
        data: &Proposal<ProposalData>,
    ): (bool, guid::ID, u64, u8, bool) {
        // looking in pending

        let (found, guid, idx) = find_index_of_ballot_by_data(tracker, data, ballot::get_pending_enum());
        if (found) {
            let b = ballot::get_ballot_by_id(tracker, &guid);
            let complete = ballot::is_completed<Proposal<ProposalData>>(b);
            return (true, guid, idx, ballot::get_pending_enum(), complete)
        };

        /* code not used and not tested
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
        };*/

        (false, guid::create_id(@0x0, 0), 0, 0, false)
    }

    /// returns the a tuple with (is_found, id, status_enum ) of ballot while seaching by data
    fun find_index_of_ballot_by_data<ProposalData: drop + store> (
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

    /// returns a tuple of (is_found: bool, index: u64, status_enum: u8, is_voting_complete: bool)
    public(friend) fun get_proposal_status_by_id<ProposalData: drop + store>(multisig_address: address, uid: &guid::ID): (bool, u64, u8, bool) acquires Action { // found, index, status_enum, is_voting_complete
        let a = borrow_global<Action<ProposalData>>(multisig_address);
        ballot::find_anywhere(&a.vote, uid)
    }

    /// get all IDs of multi_auth proposal that are pending
    fun get_pending_id<ProposalData: store + drop>(multisig_address: address): vector<guid::ID> acquires Action {
        let action = borrow_global<Action<ProposalData>>(multisig_address);
        let list = ballot::get_list_ballots_by_enum(&action.vote,
        ballot::get_pending_enum());

        let id_list = vector::map_ref(list, |el| {
            ballot::get_ballot_id(el)
        });

        id_list
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

    // TODO: check if the addresses are on chain, does not contain the multisig_address and the current authorities
    // Proposing a governance change of adding or removing signer, or changing the n-of-m of the authorities. Note that proposing will deduplicate in the event that two authorities miscommunicate and send the same proposal, in that case for UX purposes the second proposal becomes a vote.
    public(friend) fun propose_governance(sig: &signer, multisig_address: address, addresses: vector<address>, add_remove: bool, n_of_m: Option<u64>, duration_epochs: Option<u64> ): guid::ID acquires Governance, Action, Offer {
        assert_authorized(sig, multisig_address); // Duplicated with propose(), belt
        // and suspenders

        validate_owners(&addresses, multisig_address, add_remove);

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
    public(friend) fun vote_governance(sig: &signer, multisig_address: address, id: &guid::ID): bool acquires Governance, Action, Offer {
        assert_authorized(sig, multisig_address);

        let (passed, cap_opt) = {
            vote_impl<PropGovSigners>(sig, multisig_address, id)
        };
        maybe_restore_withdraw_cap(cap_opt); // don't need this but can't drop.

        if (passed) {
            let governance = borrow_global_mut<Governance>(multisig_address);
            let data = extract_proposal_data<PropGovSigners>(multisig_address, id);
            if (!vector::is_empty(&data.addresses)) {
                if (data.add_remove) {
                    propose_voted_offer(multisig_address, data.addresses, &data.n_of_m);
                    return passed
                } else {
                    maybe_update_authorities(governance, data.add_remove, &data.addresses);
                };
            };
            maybe_update_threshold(multisig_address, governance, &data.n_of_m);
        };
        passed
    }

    // New authorities voted must claim the offer to become authorities.
    fun propose_voted_offer(multisig_address: address, new_authorities: vector<address>, n_of_m: &Option<u64>) acquires Offer {
        let offer = borrow_global_mut<Offer>(multisig_address);
        let duration = epoch_helper::get_current_epoch() + DEFAULT_EPOCHS_OFFER_EXPIRE;
        let i = 0;
        while (i < vector::length(&new_authorities)) {
            let addr = vector::borrow(&new_authorities, i);
            vector::push_back(&mut offer.proposed, *addr);
            vector::push_back(&mut offer.expiration_epoch, duration);
            i = i + 1;
        };
        maybe_update_threshold_after_claim(multisig_address, n_of_m);
    }

    // If authorities voted to change the number of signatures required along authorities addition,
    // new authorities must claim the offer before the number of signatures required is applied.
    fun maybe_update_threshold_after_claim(multisig_address: address, n_of_m: &Option<u64>) acquires Offer {
        if (option::is_some(n_of_m)) {
            let new_n_of_m = *option::borrow(n_of_m);
            let current_n_of_m = multisig_account::num_signatures_required(multisig_address);
            if (current_n_of_m != new_n_of_m) {
                let offer = borrow_global_mut<Offer>(multisig_address);
                offer.proposed_n_of_m = option::some(new_n_of_m);
            };
        };
    }

    /// Updates the authorities of the multisig. This is a helper function for governance.
    // must be called with the withdraw capability and signer. belt and suspenders
    fun maybe_update_authorities(ms: &mut Governance, add_remove: bool, addresses: &vector<address>) {
        assert!(!vector::is_empty(addresses), error::invalid_argument(EEMPTY_ADDRESSES));
        multisig_account::multi_auth_helper_add_remove(&ms.guid_capability, add_remove, addresses);
    }

    fun maybe_update_threshold(multisig_address: address, governance: &mut Governance, n_of_m_opt: &Option<u64>) acquires Offer {
        if (option::is_some(n_of_m_opt)) {
            multisig_account::multi_auth_helper_update_signatures_required(&governance.guid_capability,
                *option::borrow(n_of_m_opt));

            // clean the Offer n_of_m to avoid a future claim change the n_of_m
            let offer = borrow_global_mut<Offer>(multisig_address);
            offer.proposed_n_of_m = option::none();
        };
    }

    fun validate_owners(addresses: &vector<address>, multisig_address: address, add_remove: bool) {
        let auths = multisig_account::owners(multisig_address);
        let i = 0;
        if (add_remove) {
        while (i < vector::length(addresses)) {
            let addr = vector::borrow(addresses, i);
            assert!(!vector::contains(&auths, addr), error::invalid_argument(EALREADY_OWNER));
            i = i + 1;
        };
        } else {
        while (i < vector::length(addresses)) {
            let addr = vector::borrow(addresses, i);
            assert!(vector::contains(&auths, addr), error::not_found(EOWNER_NOT_FOUND));
            i = i + 1;
        };
        };
        multisig_account::validate_owners(addresses, multisig_address);
    }

    //////// GETTERS ////////
    #[view]
    public fun get_authorities(multisig_address: address): vector<address> {
        multisig_account::owners(multisig_address)
    }

    #[view]
    public fun is_authority(multisig_addr: address, addr: address): bool {
        let auths = multisig_account::owners(multisig_addr);
        vector::contains(&auths, &addr)
    }

    #[view]
    public fun get_threshold(multisig_address: address): (u64, u64) {
        (multisig_account::num_signatures_required(multisig_address), vector::length(&multisig_account::owners(multisig_address)))
    }

    #[view]
    /// how many multi_action proposals are pending
    public fun get_count_of_pending<ProposalData: store + drop>(multisig_address: address): u64 acquires Action {
        let list = get_pending_id<ProposalData>(multisig_address);
        vector::length(&list)
    }

    #[view]
    /// the creation number u64 of the pending proposals
    public fun get_pending_by_creation_number<ProposalData: store + drop>(multisig_address: address): vector<u64> acquires Action {
        let list = get_pending_id<ProposalData>(multisig_address);
        vector::map(list, |el| {
            guid::id_creation_num(&el)
        })
    }

    #[view]
    /// returns the votes for a given proposal ID. For `view` functions must provide the destructured guid::ID as address and integer.
    public fun get_votes<ProposalData: store + drop>(multisig_address: address, id_num: u64): vector<address> acquires Action {
        let action = borrow_global<Action<ProposalData>>(multisig_address);
        let id = guid::create_id(multisig_address, id_num);
        let b = ballot::get_ballot_by_id(&action.vote, &id);
        let prop = ballot::get_type_struct(b);
        prop.votes
    }

    #[view]
    /// returns the votes for a given proposal ID. For `view` functions must provide the destructured guid::ID as address and integer.
    public fun get_expiration<ProposalData: store + drop>(multisig_address: address, id_num: u64): u64 acquires Action {
        let action = borrow_global<Action<ProposalData>>(multisig_address);
        let id = guid::create_id(multisig_address, id_num);
        let b = ballot::get_ballot_by_id(&action.vote, &id);
        let prop = ballot::get_type_struct(b);
        prop.expiration_epoch
    }


    // TODO: remove this after offer migration is completed
    public(friend) entry fun init_gov_deprecated(sig: &signer) {
        let multisig_address = signer::address_of(sig);

        if (!exists<Governance>(multisig_address)) {
            move_to(sig, Governance {
                cfg_duration_epochs: DEFAULT_EPOCHS_EXPIRE,
                cfg_default_n_sigs: 0, // deprecate
                signers: vector::empty(),
                withdraw_capability: option::none(),
                guid_capability: account::create_guid_capability(sig),
            });
        };

        if (!exists<Action<PropGovSigners>>(multisig_address)) {
            move_to(sig, Action<PropGovSigners> {
                can_withdraw: false,
                vote: ballot::new_tracker<Proposal<PropGovSigners>>(),
            });
        };
    }

  #[test_only]
  /// get the withdraw capability for testing
  public fun danger_test_get_withdraw_capability(vm: &signer, sig: &signer): Option<WithdrawCapability> acquires Governance {
    use ol_framework::testnet;
    testnet::assert_testnet(vm);
    let multisig_address = signer::address_of(sig);
    let ms = borrow_global_mut<Governance>(multisig_address);

    let c = option::extract(&mut ms.withdraw_capability);
    option::some(c)
  }

    // TODO: remove this function after offer migration is completed
    #[test_only]
    public entry fun finalize_and_cage_deprecated(sig: &signer, initial_authorities: vector<address>, num_signers: u64) {
        let addr = signer::address_of(sig);
        assert!(exists<Governance>(addr),
        error::invalid_argument(EGOV_NOT_INITIALIZED));
        assert!(exists<Action<PropGovSigners>>(addr),
        error::invalid_argument(EGOV_NOT_INITIALIZED));
        // not yet initialized
        assert!(!multisig_account::is_multisig(addr),
        error::invalid_argument(EGOV_NOT_INITIALIZED));

        multisig_account::migrate_with_owners(sig, initial_authorities, num_signers, vector::empty(), vector::empty());
    }
}
