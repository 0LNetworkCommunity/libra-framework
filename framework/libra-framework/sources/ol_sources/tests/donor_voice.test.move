#[test_only]

module ol_framework::test_donor_voice {
    use std::option;
    use ol_framework::ballot;
    use ol_framework::donor_voice;
    use ol_framework::donor_voice_txs;
    use ol_framework::mock;
    use ol_framework::ol_account;
    use ol_framework::ol_features_constants;
    use ol_framework::multi_action;
    use ol_framework::receipts;
    use ol_framework::donor_voice_governance;
    use ol_framework::donor_voice_reauth;
    use ol_framework::burn;
    use ol_framework::slow_wallet;
    use std::features;
    use std::guid;
    use std::vector;
    use std::signer;


    #[test(root = @ol_framework, alice = @0x1000a)]
    fun test_meta_set_authorized(root: &signer, alice: &signer, ) {
        // Scenario: Test that the test_set_authorized helper function works correctly
        // Create a community wallet and verify that the test_set_authorized function correctly
        // changes its authorization status

        let vals = mock::genesis_n_vals(root, 3);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0xabcdef");
        let donor_voice_address = signer::address_of(&resource_sig);

        // Set up the donor voice account
        donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);
        donor_voice_reauth::assert_authorized(donor_voice_address);
        donor_voice_reauth::set_requires_reauth(alice, donor_voice_address);

        // Check if it's not yet authorized
        assert!(!donor_voice_reauth::is_authorized(donor_voice_address), 7357001);

        // Use the test_set_authorized to authorize the wallet
        donor_voice_reauth::test_set_authorized(root, donor_voice_address);
        donor_voice_reauth::assert_authorized(donor_voice_address);

        // Verify that it's now authorized
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dv_init(root: &signer, alice: &signer, bob: &signer) {
      let vals = mock::genesis_n_vals(root, 2);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      let list = donor_voice::get_root_registry();
      assert!(vector::length(&list) == 1, 7357001);

      //shouldnt be donor voice yet. Needs to be caged first
      assert!(!donor_voice_txs::is_donor_voice(donor_voice_address), 7357002);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      assert!(donor_voice_txs::is_donor_voice(donor_voice_address), 7357003);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dv_propose_payment(root: &signer, alice: &signer, bob: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 2);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vector::length(&vals));

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, marlon_rando = @0x123456)]
    fun dv_propose_payment_unlocked(root: &signer, alice: signer, bob: signer, marlon_rando: address) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 2);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

      ol_account::create_account(root, marlon_rando);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(&alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(&alice, donor_voice_address);
      multi_action::claim_offer(&bob, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vector::length(&vals));

      ol_account::transfer(&alice, donor_voice_address, 1000);

      let (_unlocked_dv, dv_balance) = ol_account::balance(donor_voice_address);
      let (_, marlon_balance) = ol_account::balance(marlon_rando);
      diem_std::debug::print(&dv_balance);
      diem_std::debug::print(&marlon_balance);


      let unlocked_advance = true;
      // sends an 0.1% of the CW balance to an unrestricted/ordinary account
      donor_voice_txs::propose_payment_tx(bob, donor_voice_address, marlon_rando, 1, b"thanks marlon", unlocked_advance);

      let list = donor_voice_txs::list_by_status(donor_voice_address, donor_voice_txs::voting_enum());
      let tx_id_num = *vector::borrow(&list, 0);

      let (proposal_found, proposal_idx, proposal_status, proposal_completed, schedule_found, _schedule_idx, _schedule_status) = donor_voice_txs::get_tx_status(donor_voice_address, tx_id_num);
      assert!(proposal_found, 7357004);
      assert!(proposal_idx == 0, 7357005);
      assert!(proposal_status == ballot::get_pending_enum(), 7357006);
      assert!(!proposal_completed, 7357007);
      assert!(!schedule_found, 7357008);

      // // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, tx_id_num), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    fun dv_schedule_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);

      //need to cage to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_approved_enum(), 7357006);
      assert!(completed, 7357007); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    fun dv_propose_and_veto(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer) {
      // Scenario: Eve wants to veto a transaction on a donor directed account.
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account. The veto can only happen after Alice Bob and Carol are able to schedule a tx.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues

      mock::trigger_epoch(root);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);
      multi_action::claim_offer(eve, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      // EVE Establishes some governance over the wallet, when donating.
      let eve_donation = 42;
      ol_account::transfer(eve, donor_voice_address, eve_donation);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), donor_voice_address);
      assert!(total_funds_sent_by_eve == eve_donation, 7357001); // last payment
      let is_donor = donor_voice_governance::check_is_donor(donor_voice_address, signer::address_of(eve));
      assert!(is_donor, 7357002);

      // Dave will also be a donor
      ol_account::transfer(dave, donor_voice_address, 1);
      let is_donor = donor_voice_governance::check_is_donor(donor_voice_address, signer::address_of(dave));
      assert!(is_donor, 7357003);

      // Bob proposes a tx that will come from the donor directed account.
      // It is not yet scheduled because it doesn't have the MultiAuth quorum. Still waiting for Alice or Carol to approve.
      let uid_of_transfer = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (_found, _idx, _status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid_of_transfer);
      assert!(!completed, 7357004);

      // Eve wants to propose a Veto, but this should fail at this, because
      // the tx is not yet scheduled
      let _uid_of_veto_prop = donor_voice_txs::test_propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_voice_governance::tx_has_veto_pending(donor_voice_address, guid::id_creation_num(&uid_of_transfer));
      assert!(!has_veto, 7357005); // propose does not cast a vote.

      // Now Carol, along with Bob, as admins have proposed the payment.
      // Now the payment should be scheduled
      let uid_of_transfer = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid_of_transfer);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_approved_enum(), 7357006);
      assert!(completed, 7357007); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid_of_transfer)), 7357008);

      // Eve tries again after it has been scheduled
      let ballot_id = donor_voice_txs::test_propose_veto(eve, &uid_of_transfer);
      let ballot_id_num = guid::id_creation_num(option::borrow(&ballot_id));

      let has_veto = donor_voice_governance::tx_has_veto_pending(donor_voice_address, guid::id_creation_num(&uid_of_transfer));
      assert!(has_veto, 7357007); // propose does not cast a vote.

      // now vote on the proposal
      // note need to destructure ID for the entry and view functions
      let tx_id_num = guid::id_creation_num(&uid_of_transfer);

      // proposing is not same as voting, now eve votes
      // NOTE: there is a tx function that can propose and vote in single step
      donor_voice_txs::vote_veto_tx(eve, donor_voice_address, tx_id_num);

      let is_pending = donor_voice_governance::tx_has_veto_pending(donor_voice_address, tx_id_num);

      // the vote has completed, and so this payment tx
      // will not appear as having a veto pending
      assert!(!is_pending, 7357008);



      let (approve_pct, _, req_threshold, _, _, approved, is_closed, status, is_complete) = donor_voice_governance::get_veto_tally(donor_voice_address, ballot_id_num);

      assert!(approve_pct == 10000, 7357008);
      assert!(req_threshold == 5100, 7357009);
      assert!(approved, 7357010);
      assert!(is_closed, 7357011);
      assert!(status == 2, 7357012);
      assert!(is_complete, 7357013);

      // // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address,
      guid::id_creation_num(&uid_of_transfer)), 7357014);

      // it's vetoed
      assert!(donor_voice_txs::is_veto(donor_voice_address, guid::id_creation_num(&uid_of_transfer)), 7357015);
    }

    // should not be able sign a tx twice
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 65550, location = 0x1::multi_action)]
    fun dv_reject_duplicate_proposal(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);

      //need to cage to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000c, 100, b"thanks carol");

      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000c, 100, b"thanks carol");

      // confirm it is scheduled
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    fun dv_process_unit(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

      let (bob_unlocked_pre, bob_balance_pre) = ol_account::balance(@0x1000b);
      assert!(bob_balance_pre == 10000000, 7357001);
      assert!(bob_unlocked_pre == 10000000, 7357002);


      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_voice_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 100, 7357003);


      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);


      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357009);
      assert!(idx == 0, 7357010);
      assert!(status_enum == ballot::get_approved_enum(), 7357011);
      assert!(completed, 7357012); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357013);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357014);

      // process epoch 3 accounts
      donor_voice_txs::process_donor_voice_accounts(root, 3);

      let (bob_unlocked, bob_balance) = ol_account::balance(@0x1000b);
      assert!(bob_balance > bob_balance_pre, 7357015);
      assert!(bob_balance == 10000100, 7357016);

      // the new balance is locked
      assert!(bob_unlocked != bob_balance, 7357017);
      assert!(bob_unlocked == bob_unlocked_pre, 7357018);


      // the first proposal should be processed
      let (found, idx, status_enum, completed) =
      donor_voice_txs::get_multisig_proposal_state(donor_voice_address,
      &uid);
      assert!(found, 7357018);
      assert!(idx == 0, 7357019);
      assert!(status_enum == ballot::get_approved_enum(), 7357020);
      assert!(completed, 7357021); // now vote is completed
      assert!(donor_voice_txs::is_paid(donor_voice_address, guid::id_creation_num(&uid)), 7357022);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, marlon_rando = @0x123456)]
    fun dv_process_epoch_boundary(root: &signer, alice: &signer, bob: &signer, carol: &signer, marlon_rando: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

      ol_account::create_account(root, signer::address_of(marlon_rando));
      let (_bal, marlon_rando_balance_pre) = ol_account::balance(signer::address_of(marlon_rando));
      assert!(marlon_rando_balance_pre == 0, 7357000);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_voice_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 100, 7357002);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);
      slow_wallet::user_set_slow(marlon_rando);

      donor_voice_reauth::test_set_authorized(root, donor_voice_address);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), 100, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), 100, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_approved_enum(), 7357006);
      assert!(completed, 7357007); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);

      // process epoch 3 accounts
      // donor_voice_txs::process_donor_voice_accounts(root, 3);
      mock::trigger_epoch(root); // into epoch 1
      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment

      let (_bal, marlon_rando_balance_post) = ol_account::balance(signer::address_of(marlon_rando));

      assert!(marlon_rando_balance_post == marlon_rando_balance_pre + 100, 7357006);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, marlon_rando = @0x123456)]
    // Scenario: we are testing that the community wallet can use some
    // unlocked coins, sending to marlon's unlocked account
    fun dv_process_epoch_boundary_unlocked(root: &signer, alice: signer, bob: signer, carol: signer, marlon_rando: signer) {
      let vals = mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);
      let marlon_addr = signer::address_of(&marlon_rando);
      ol_account::create_account(root, marlon_addr);
      let (marlon_unlocked_pre, marlon_rando_balance_pre) = ol_account::balance(marlon_addr);
      assert!(marlon_rando_balance_pre == 0, 7357000);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(&alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(&alice, donor_voice_address, 100_000);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 100_000, 7357002);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(&alice, donor_voice_address);
      multi_action::claim_offer(&bob, donor_voice_address);
      multi_action::claim_offer(&carol, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);
      // slow_wallet::user_set_slow(marlon_rando);

      donor_voice_reauth::test_set_authorized(root, donor_voice_address);

      let is_unlocked_advance = true;

      donor_voice_txs::propose_payment_tx(bob, donor_voice_address, marlon_addr, 100, b"thanks marlon", is_unlocked_advance);

      donor_voice_txs::propose_payment_tx(carol, donor_voice_address, marlon_addr, 100, b"thanks marlon", is_unlocked_advance);

      // PROCESS THE PAYMENT
      // process epoch 3 accounts

      mock::trigger_epoch(root); // into epoch 1
      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment

      let (marlon_unlocked_post, marlon_rando_balance_post) = ol_account::balance(marlon_addr);

      assert!(marlon_rando_balance_post == marlon_rando_balance_pre + 100, 7357006);

      // check the marlon's balance is unlocked
      assert!(marlon_unlocked_post > marlon_unlocked_pre, 7357007);
      assert!(marlon_unlocked_post == marlon_unlocked_pre + 100, 7357007);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, marlon_rando = @0x123456)]
    fun dv_process_multi_same_epoch(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, marlon_rando: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      // Marlon will get two payments scheduled twice on the SAME epoch
      // First time for 111, then 222.
      let marlon_pay_one = 111;
      let marlon_pay_two = 222;

      let vals = mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

      ol_account::create_account(root, signer::address_of(marlon_rando));
      let (_bal, marlon_rando_balance_pre) = ol_account::balance(signer::address_of(marlon_rando));
      assert!(marlon_rando_balance_pre == 0, 7357000);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account with 10000
      ol_account::transfer(alice, donor_voice_address, 10000);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 10000, 7357002);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      slow_wallet::user_set_slow(marlon_rando);

      let first_uid_bob = donor_voice_txs::test_propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_one, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &first_uid_bob);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&first_uid_bob)), 7357008);

      let first_uid_carol = donor_voice_txs::test_propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_one, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &first_uid_carol);
      assert!(found, 7357009);
      assert!(idx == 0, 73570010);
      assert!(status_enum == ballot::get_approved_enum(), 73570011);
      assert!(completed, 73570012); // now vote is completed
      assert!(first_uid_bob == first_uid_carol, 73570021);

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&first_uid_carol)), 73570013);

      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &first_uid_bob), 73570014);

      let second_uid_bob = donor_voice_txs::test_propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_two, b"thanks again!!!");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &second_uid_bob);
      assert!(found, 73570015);
      assert!(idx == 0, 73570016); // since the above payment was approved, now
      // this is the only one in the multi_auth pending list
      assert!(status_enum == ballot::get_pending_enum(), 73570017);
      assert!(!completed, 73570018);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&second_uid_bob)), 73570019);

      let second_uid_carol = donor_voice_txs::test_propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_two, b"thanks again!!!");
      let (found, idx, status_enum, completed) =
      donor_voice_txs::get_multisig_proposal_state(donor_voice_address,
      &second_uid_carol);
      assert!(second_uid_bob == second_uid_carol, 73570021);
      assert!(found, 73570021);
      assert!(idx == 1, 73570022); // now there should be two payments that
      // multi_auth has approved, and thus scheduled
      assert!(status_enum == ballot::get_approved_enum(), 73570023);
      assert!(completed, 73570024); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&second_uid_carol)), 73570025);
      assert!(vector::contains(&list, &first_uid_bob), 73570026);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &first_uid_bob), 73570027);
      assert!(vector::contains(&list, &second_uid_bob), 73570028);

      // process epoch 3 accounts
      // one epoch goes by and then new payment to marlon
      mock::trigger_epoch(root); // into epoch 1

      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment

      // MARLON'S FIRST PAYMENT GOES THROUGH
      let (_, marlon_rando_balance_post) =
      ol_account::balance(signer::address_of(marlon_rando));

      // the first proposal should be processed
      let (found, idx, status_enum, completed) =
      donor_voice_txs::get_multisig_proposal_state(donor_voice_address,
      &first_uid_bob);
      assert!(found, 73570029);
      assert!(idx == 0, 73570030);
      assert!(status_enum == ballot::get_approved_enum(), 73570031);
      assert!(completed, 73570032); // now vote is completed
      assert!(donor_voice_txs::is_paid(donor_voice_address, guid::id_creation_num(&first_uid_bob)), 73570033);
      // the second proposal should be processed
      let (found, _idx, status_enum, completed) =
      donor_voice_txs::get_multisig_proposal_state(donor_voice_address,
      &second_uid_bob);
      assert!(found, 73570034);
      assert!(status_enum == ballot::get_approved_enum(), 73570035);
      assert!(completed, 73570036); // now vote is completed

      assert!(marlon_rando_balance_post == (marlon_rando_balance_pre +
      marlon_pay_one + marlon_pay_two),
      73570027);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, marlon_rando = @0x123456)]
    fun dv_process_multi_different_epochs(root: &signer, alice: &signer, bob: &signer, carol: &signer, marlon_rando: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      // Marlon will get two payment scheduled twice, over two DIFFERENT epochs
      // First time for 111, then 222.
      let marlon_pay_one = 111;
      let marlon_pay_two = 222;


      let vals = mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

      ol_account::create_account(root, signer::address_of(marlon_rando));
      let (_, marlon_rando_balance_pre) = ol_account::balance(signer::address_of(marlon_rando));
      assert!(marlon_rando_balance_pre == 0, 7357000);


      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account with 10000
      ol_account::transfer(alice, donor_voice_address, 10000);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 10000, 7357002);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      slow_wallet::user_set_slow(marlon_rando);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_one, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == ballot::get_pending_enum(), 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_one, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357009);
      assert!(idx == 0, 73570010); // it's index 0 of approved payments
      assert!(status_enum == ballot::get_approved_enum(), 73570011);
      assert!(completed, 73570012); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 73570013);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 73570014);

      // one epoch goes by and then new payment to marlon
      mock::trigger_epoch(root); // into epoch 1

      let second_uid_bob = donor_voice_txs::test_propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_two, b"thanks again!!!");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &second_uid_bob);
      assert!(found, 73570015);
      assert!(idx == 0, 73570016); // now pending is empty, should be first
      // index of pending list.
      assert!(status_enum == ballot::get_pending_enum(), 73570017);
      assert!(!completed, 73570018);
      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&second_uid_bob)), 73570019);

      let second_uid_carol = donor_voice_txs::test_propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), marlon_pay_two, b"thanks again!!!");
      let (found, idx, status_enum, completed) =
      donor_voice_txs::get_multisig_proposal_state(donor_voice_address,
      &second_uid_carol);
      assert!(second_uid_bob == second_uid_carol, 73570021);
      assert!(found, 73570021);
      assert!(idx == 1, 73570022); // now there should be two approved payments
      // from multi_auth and thus scheduled
      assert!(status_enum == ballot::get_approved_enum(), 73570023);
      assert!(completed, 73570024); // now vote is completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&second_uid_carol)), 73570025);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 73570026);

      // process epoch 3 accounts
      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment

      // MARLON'S FIRST PAYMENT GOES THROUGH
      let (_, marlon_rando_balance_post) = ol_account::balance(signer::address_of(marlon_rando));

      assert!(marlon_rando_balance_post == (marlon_rando_balance_pre + marlon_pay_one),
      73570027);

      // MARLON'S SECOND PAYMENT SHOULD SUCCEED

      mock::trigger_epoch(root); // epoch 5 should include the next payment
      let (_, marlon_rando_balance_post) =
      ol_account::balance(signer::address_of(marlon_rando));

      assert!(marlon_rando_balance_post == (marlon_rando_balance_pre +
      marlon_pay_one + marlon_pay_two), 73570028);
    }


    #[test(root = @ol_framework, _alice = @0x1000a, dave = @0x1000d, eve = @0x1000e, donor_voice = @0x1000f)]
    fun dv_liquidate_to_donor(root: &signer, _alice: &signer, dave: &signer, eve: &signer, donor_voice: &signer) {
      // Scenario:
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account.
      // Dave and Eve are unhappy, and vote to liquidate the account.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100_000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause
      // issues
      mock::trigger_epoch(root);
      // a burn happened in the epoch above, so let's compare it to end of epoch
      let (lifetime_burn_pre, _) = burn::get_lifetime_tracker();

      let (donor_voice, _cap) = ol_account::test_ol_create_resource_account(donor_voice, b"0x1");
      let donor_voice_address = signer::address_of(&donor_voice);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &donor_voice, vals);

      // EVE Establishes some governance over the wallet, when donating.
      let eve_donation = 42;
      ol_account::transfer(eve, donor_voice_address, eve_donation);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), donor_voice_address);
      assert!(total_funds_sent_by_eve == eve_donation, 7357001);

      let is_donor = donor_voice_governance::check_is_donor(donor_voice_address, signer::address_of(eve));
      assert!(is_donor, 7357002);

      // Dave will also be a donor, with half the amount of what Eve
      let dave_donation = 21;
      ol_account::transfer(dave, donor_voice_address, dave_donation);
      let is_donor = donor_voice_governance::check_is_donor(donor_voice_address, signer::address_of(dave));
      assert!(is_donor, 7357003);

      // Dave proposes to liquidate the account.
      donor_voice_txs::test_propose_liquidation(dave, donor_voice_address);

      assert!(donor_voice_governance::is_liquidation_proposed(donor_voice_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_voice_txs::vote_liquidation_tx(eve, donor_voice_address);

      let list = donor_voice::get_liquidation_queue();
      assert!(vector::length(&list) > 0, 7357005);

      let (addrs, refunds) = donor_voice_txs::get_pro_rata(donor_voice_address);

      assert!(*vector::borrow(&addrs, 0) == @0x1000e, 7357006);
      assert!(*vector::borrow(&addrs, 1) == @0x1000d, 7357007);

      let eve_donation_pro_rata = vector::borrow(&refunds, 0);
      let superman_3 = 1; // rounding from fixed_point32
      assert!((*eve_donation_pro_rata + superman_3) == eve_donation, 7357008);

      let dave_donation_pro_rata = vector::borrow(&refunds, 1);
      let superman_3 = 1; // rounding from fixed_point32
      assert!((*dave_donation_pro_rata + superman_3) == dave_donation, 7357009);

      let (_, program_balance_pre) = ol_account::balance(donor_voice_address);
      let (_, eve_balance_pre) = ol_account::balance(@0x1000e);

      donor_voice_txs::test_helper_vm_liquidate(root);

      let (_, program_balance) = ol_account::balance(donor_voice_address);
      let (_, eve_balance) = ol_account::balance(@0x1000e);

      assert!(program_balance < program_balance_pre, 7357010);
      assert!(program_balance == 0, 7357011);

      // eve should have received funds back
      assert!(eve_balance > eve_balance_pre, 7357010);

      // nothing should have been burned, it was a refund
      let (lifetime_burn_now, _) = burn::get_lifetime_tracker();
      assert!(lifetime_burn_now == lifetime_burn_pre, 7357011);
    }

    #[test(root = @ol_framework, alice = @0x1000a, dave = @0x1000d, eve = @0x1000e)]
    fun dv_liquidate_to_match_index(root: &signer, alice: &signer, dave: &signer, eve: &signer) {
      // Scenario:
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account.
      // Dave and Eve are unhappy, and vote to liquidate the account.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);
      // a burn happened in the epoch above, so let's compare it to end of epoch
      let (lifetime_burn_pre, _) = burn::get_lifetime_tracker();

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);
      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);
      donor_voice_txs::set_liquidate_to_match_index(&resource_sig, true);

      // EVE Establishes some governance over the wallet, when donating.
      let eve_donation = 42;
      ol_account::transfer(eve, donor_voice_address, eve_donation);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), donor_voice_address);
      assert!(total_funds_sent_by_eve == eve_donation, 7357001);

      let is_donor = donor_voice_governance::check_is_donor(donor_voice_address, signer::address_of(eve));
      assert!(is_donor, 7357002);

      // Dave will also be a donor, with half the amount of what Eve sends
      let dave_donation = 21;
      ol_account::transfer(dave, donor_voice_address, dave_donation);
      let is_donor = donor_voice_governance::check_is_donor(donor_voice_address, signer::address_of(dave));
      assert!(is_donor, 7357003);

      // Dave proposes to liquidate the account.
      donor_voice_txs::test_propose_liquidation(dave, donor_voice_address);

      assert!(donor_voice_governance::is_liquidation_proposed(donor_voice_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_voice_txs::vote_liquidation_tx(eve, donor_voice_address);

      let list = donor_voice::get_liquidation_queue();
      assert!(vector::length(&list) > 0, 7357005);

      // check the table of refunds
      // dave and eve should be receiving
      let (addrs, refunds) = donor_voice_txs::get_pro_rata(donor_voice_address);

      assert!(*vector::borrow(&addrs, 0) == @0x1000e, 7357006);
      assert!(*vector::borrow(&addrs, 1) == @0x1000d, 7357007);

      // for the first refund,
      let eve_donation_pro_rata = vector::borrow(&refunds, 0);
      let superman_3 = 1; // rounding from fixed_point32
      assert!((*eve_donation_pro_rata + superman_3) == eve_donation, 7357008);

      let dave_donation_pro_rata = vector::borrow(&refunds, 1);
      let superman_3 = 1; // rounding from fixed_point32
      assert!((*dave_donation_pro_rata + superman_3) == dave_donation, 7357009);

      let (_, program_balance_pre) = ol_account::balance(donor_voice_address);
      let (_, eve_balance_pre) = ol_account::balance(@0x1000e);

      donor_voice_txs::test_helper_vm_liquidate(root);

      let (_, program_balance) = ol_account::balance(donor_voice_address);
      let (_, eve_balance) = ol_account::balance(@0x1000e);

      assert!(program_balance < program_balance_pre, 7357010);
      assert!(program_balance == 0, 7357011);

      // eve's account should not have changed.
      // she does not get a refund, and the policy of the
      // program says it goes to the matching index.
      assert!(eve_balance == eve_balance_pre, 7357010);

      let (lifetime_burn_now, _lifetime_match) = burn::get_lifetime_tracker();

      // nothing should have been burned, it was a refund
      assert!(lifetime_burn_now == lifetime_burn_pre, 7357011);
    }


    fun create_friendly_helper_accounts(root: &signer): vector<address>{

        // recruit 3 friendly folks to be signers or donors
        ol_account::create_account(root, @80801);
        ol_account::create_account(root, @80802);
        ol_account::create_account(root, @80803);

        let workers = vector::empty<address>();

        // helpers in line to help
        vector::push_back(&mut workers, @80801);
        vector::push_back(&mut workers, @80802);
        vector::push_back(&mut workers, @80803);

        workers
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 196616, location = 0x1::donor_voice_txs)]

    fun dv_init_fails_in_gov_mode(root: &signer, alice: &signer) {
      // Scenario: Alice tries to make an account a DV account
      // but fails because we are in governance mode.

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let _donor_voice_address = signer::address_of(&resource_sig);

      //////// ENTER GOVERNANCE MODE ////////
      let gov_mode_id = ol_features_constants::get_governance_mode();
      features::change_feature_flags(root, vector::singleton(gov_mode_id), vector::empty());

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 196616, location = 0x1::donor_voice_txs)]

    fun dv_schedule_fails_in_gov_mode(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
      // Scenario: Alice has a pre-existing DV account
      // and tries to schedule a transactions but fails because
      // we are in governance mode.

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);

      //need to cage to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      //////// ENTER GOVERNANCE MODE ////////
      let gov_mode_id = ol_features_constants::get_governance_mode();
      features::change_feature_flags(root, vector::singleton(gov_mode_id), vector::empty());

      let _uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    fun dv_schedule_advance_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
      // Scenario: The community wallet will get some unlocked coins as an advance.
      // Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig, vals);

      // vals claim the offer
      multi_action::claim_offer(alice, donor_voice_address);
      multi_action::claim_offer(bob, donor_voice_address);
      multi_action::claim_offer(carol, donor_voice_address);
      multi_action::claim_offer(dave, donor_voice_address);

      //need to cage to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, 2);

      let (_bal, bob_bal_before) = ol_account::balance(@0x1000b);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357001);
      assert!(idx == 0, 7357002);
      assert!(status_enum == ballot::get_pending_enum(), 7357003);
      assert!(!completed, 7357004);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, guid::id_creation_num(&uid)), 7357005);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357006);
      assert!(idx == 0, 7357007);
      assert!(status_enum == ballot::get_approved_enum(), 7357008);
      assert!(completed, 7357009); // now vote is completed

      let tx_id_num = guid::id_creation_num(&uid);
      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, tx_id_num), 7357008);

      // process epoch 3 accounts
      mock::trigger_epoch(root); // into epoch 1
      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment

      let (_bal, bob_bal_now) = ol_account::balance(@0x1000b);

      assert!(bob_bal_now > bob_bal_before, 7357009);

    }
}
