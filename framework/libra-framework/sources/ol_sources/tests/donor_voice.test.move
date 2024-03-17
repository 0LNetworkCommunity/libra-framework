
#[test_only]

module ol_framework::test_donor_voice {
  use ol_framework::donor_voice;
  use ol_framework::donor_voice_txs;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::multi_action;
  use ol_framework::receipts;
  use ol_framework::donor_voice_governance;
  use ol_framework::community_wallet_init;
  use ol_framework::community_wallet;
  use ol_framework::burn;
  use diem_framework::multisig_account;
  use std::guid;
  use std::vector;
  use std::signer;

  // use diem_std::debug::print;

    #[test(root = @ol_framework, alice = @0x1000a)]
    fun dd_init(root: &signer, alice: &signer) {
      mock::genesis_n_vals(root, 4);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      let auths = mock::personas();

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);

      let list = donor_voice::get_root_registry();
      assert!(vector::length(&list) == 1, 7357001);

      //shouldnt be donor voice yet. Needs to be caged first
      assert!(!donor_voice_txs::is_donor_voice(donor_voice_address), 7357002);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, auths, vector::length(&auths));

      assert!(donor_voice_txs::is_donor_voice(donor_voice_address), 7357003);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dd_propose_payment(root: &signer, alice: &signer, bob: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_schedule_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);

      //need to cage to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vals, 2);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);

      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    fun dd_propose_and_veto(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer) {
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
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vals, 2);

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
      // It is not yet scheduled because it doesnt have the MultiAuth quorum. Still waiting for Alice or Carol to approve.
      let uid_of_transfer = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");

      let (_found, _idx, _status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid_of_transfer);
      assert!(!completed, 7357004);


      // Eve wants to propose a Veto, but this should fail at this, because
      // the tx is not yet scheduled
      let _uid_of_veto_prop = donor_voice_txs::test_propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_voice_governance::tx_has_veto(donor_voice_address, guid::id_creation_num(&uid_of_transfer));
      assert!(!has_veto, 7357005); // propose does not cast a vote.


      // Now Carol, along with Bob, as admins have proposed the payment.
      // Now the payment should be scheduled
      let uid_of_transfer = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (_found, _idx, state) = donor_voice_txs::get_schedule_state(donor_voice_address, &uid_of_transfer);
      assert!(state == 1, 7357006); // is scheduled


      // Eve tries again after it has been scheduled
      let _uid_of_veto_prop = donor_voice_txs::test_propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_voice_governance::tx_has_veto(donor_voice_address, guid::id_creation_num(&uid_of_transfer));
      assert!(has_veto, 7357007); // propose does not cast a vote.

      // now vote on the proposal
      // note need to destructure ID for the entry and view functions
      let id_num = guid::id_creation_num(&uid_of_transfer);

      // proposing is not same as voting, now eve votes
      // NOTE: there is a tx function that can propose and vote in single step
      donor_voice_txs::vote_veto_tx(eve, donor_voice_address, id_num);

      let (approve_pct, req_threshold) = donor_voice_governance::get_veto_tally(donor_voice_address, guid::id_creation_num(&uid_of_transfer));

      assert!(approve_pct == 10000, 7357008);
      assert!(req_threshold == 5100, 7357009);

      // since Eve is the majority donor, and has almost all the votes, the voting will end early. On the next epoch the vote should end
      mock::trigger_epoch(root);
      donor_voice_txs::vote_veto_tx(dave, donor_voice_address, id_num);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, &uid_of_transfer), 7357010);

      // should not be scheduled
      let (_found, _idx, state) = donor_voice_txs::get_schedule_state(donor_voice_address, &uid_of_transfer);
      assert!(state == 2, 7357006); // is veto
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_process_unit(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

      let (_, bob_balance_pre) = ol_account::balance(@0x1000b);
      assert!(bob_balance_pre == 10000000, 7357001);

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_voice_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);

      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vals, 2);


      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice_txs::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);

      // process epoch 3 accounts
      donor_voice_txs::process_donor_voice_accounts(root, 3);

      let (_, bob_balance) = ol_account::balance(@0x1000b);
      assert!(bob_balance > bob_balance_pre, 7357005);
      assert!(bob_balance == 10000100, 7357006);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, marlon_rando = @0x123456)]
    fun dd_process_epoch_boundary(root: &signer, alice: &signer, bob: &signer, carol: &signer, marlon_rando: &signer) {
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
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);


      //need to be caged to finalize donor directed workflow and release control of the account
      multi_action::finalize_and_cage(&resource_sig, vals, 2);

      let uid = donor_voice_txs::test_propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), 100, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);

      let uid = donor_voice_txs::test_propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), 100, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, &uid), 7357008);

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

    #[test(root = @ol_framework, _alice = @0x1000a, dave = @0x1000d, eve = @0x1000e, donor_voice = @0x1000f)]
    fun dd_liquidate_to_donor(root: &signer, _alice: &signer, dave: &signer, eve: &signer, donor_voice: &signer) {
      // Scenario:
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account.
      // Dave and Eve are unhappy, and vote to liquidate the account.

      mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause
      // issues
      mock::trigger_epoch(root);
      // a burn happend in the epoch above, so let's compare it to end of epoch
      let (lifetime_burn_pre, _) = burn::get_lifetime_tracker();

      let donor_voice_address = signer::address_of(donor_voice);
      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, donor_voice);
      // donor_voice_txs::set_liquidate_to_match_index(&resource_sig, true);

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

      assert!(donor_voice_governance::is_liquidation_propsed(donor_voice_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_voice_txs::vote_liquidation_tx(eve, donor_voice_address);

      mock::trigger_epoch(root);

      // needs a vote on the day after the threshold has passed
      donor_voice_txs::vote_liquidation_tx(dave, donor_voice_address);

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

      // eve shoul have received funds back
      assert!(eve_balance > eve_balance_pre, 7357010);

      let (lifetime_burn_now, _) = burn::get_lifetime_tracker();
      // nothing should have been burned, it was a refund
      assert!(lifetime_burn_now == lifetime_burn_pre, 7357011);

    }

    #[test(root = @ol_framework, alice = @0x1000a, dave = @0x1000d, eve = @0x1000e)]
    fun dd_liquidate_to_match_index(root: &signer, alice: &signer, dave: &signer, eve: &signer) {
      // Scenario:
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account.
      // Dave and Eve are unhappy, and vote to liquidate the account.

      mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);
      // a burn happend in the epoch above, so let's compare it to end of epoch
      let (lifetime_burn_pre, _) = burn::get_lifetime_tracker();

      let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);
      // the account needs basic donor directed structs
      donor_voice_txs::test_helper_make_donor_voice(root, &resource_sig);
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

      assert!(donor_voice_governance::is_liquidation_propsed(donor_voice_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_voice_txs::vote_liquidation_tx(eve, donor_voice_address);

      mock::trigger_epoch(root);
      // needs a vote on the day after the threshold has passed
      donor_voice_txs::vote_liquidation_tx(dave, donor_voice_address);

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

    #[test(root = @ol_framework, community = @0x10011)]
    fun migrate_cw_bug_not_resource(root: &signer, community: &signer) {

      // create genesis and fund accounts
      let auths = mock::genesis_n_vals(root, 3);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);


      let community_wallet_address = signer::address_of(community);
      // genesis migration would have created this account.
      ol_account::create_account(root, community_wallet_address);
      // migrate community wallet
      // THIS PUTS THE ACCOUNT IN LIMBO
      community_wallet_init::migrate_community_wallet_account(root, community);

      // verify correct migration of community wallet
      assert!(community_wallet::is_init(community_wallet_address), 7357001); //TODO: find appropriate error codes

      // the usual initialization should fix the structs
      community_wallet_init::init_community(community, auths, 2);
      // confirm the bug
      assert!(!multisig_account::is_multisig(community_wallet_address), 7357002);

      // fix it by calling multi auth:
      community_wallet_init::finalize_and_cage(community, auths, 2);
      assert!(multisig_account::is_multisig(community_wallet_address), 7357003);

      community_wallet_init::assert_qualifies(community_wallet_address);
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
}
