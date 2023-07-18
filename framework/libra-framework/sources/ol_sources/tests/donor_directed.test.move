
#[test_only]

module ol_framework::test_donor_directed {
  use ol_framework::donor_directed;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use aptos_framework::resource_account;
  use ol_framework::receipts;
  use ol_framework::donor_directed_governance;
  use std::guid;
  use std::vector;
  use std::signer;

  use aptos_std::debug::print;

    #[test(root = @ol_framework, alice = @0x1000a)]
    fun dd_init(root: &signer, alice: &signer) {
      mock::genesis_n_vals(root, 4);

      // Alice turns this account into a resource account. This can also happen after other donor directed initializations happen (or deposits). But must be complete before the donor directed wallet can be used.
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);
      assert!(resource_account::is_resource_account(donor_directed_address), 7357003);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);


      let list = donor_directed::get_root_registry();
      assert!(vector::length(&list) == 1, 7357001);

      assert!(donor_directed::is_donor_directed(donor_directed_address), 7357002);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dd_propose_payment(root: &signer, alice: &signer, bob: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_schedule_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);

      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_directed::find_by_deadline(donor_directed_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    fun dd_propose_and_veto(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer) {
      // Scenario: Eve wants to veto a transaction on a donor directed account.
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account. The veto can only happen after Alice Bob and Carol are able to schedule a tx.

      mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000a, @0x1000b, @0x1000c, 2);

      // EVE Establishes some governance over the wallet, when donating.
      let eve_donation = 42;
      ol_account::transfer(eve, donor_directed_address, eve_donation);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), donor_directed_address);
      assert!(total_funds_sent_by_eve == eve_donation, 7357001); // last payment
      let is_donor = donor_directed_governance::check_is_donor(donor_directed_address, signer::address_of(eve));
      assert!(is_donor, 7357002);

      // Dave will also be a donor
      ol_account::transfer(dave, donor_directed_address, 1);
      let is_donor = donor_directed_governance::check_is_donor(donor_directed_address, signer::address_of(dave));
      assert!(is_donor, 7357003);

      // Bob proposes a tx that will come from the donor directed account.
      // It is not yet scheduled because it doesnt have the MultiAuth quorum. Still waiting for Alice or Carol to approve.
      let uid_of_transfer = donor_directed::propose_payment(bob, donor_directed_address, @0x1000b, 100, b"thanks bob");

      let (_found, _idx, _status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid_of_transfer);
      assert!(!completed, 7357004);


      // Eve wants to propose a Veto, but this should fail at this, because
      // the tx is not yet scheduled
      let _uid_of_veto_prop = donor_directed::propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_directed_governance::tx_has_veto(donor_directed_address, guid::id_creation_num(&uid_of_transfer));
      assert!(!has_veto, 7357005); // propose does not cast a vote.


      // Now Carol, along with Bob, as admins have proposed the payment.
      // Now the payment should be scheduled
      let uid_of_transfer = donor_directed::propose_payment(carol, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (_found, _idx, state) = donor_directed::get_schedule_state(donor_directed_address, &uid_of_transfer);
      assert!(state == 1, 7357006); // is scheduled


      // Eve tries again after it has been scheduled
      let _uid_of_veto_prop = donor_directed::propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_directed_governance::tx_has_veto(donor_directed_address, guid::id_creation_num(&uid_of_transfer));
      assert!(has_veto, 7357007); // propose does not cast a vote.

      // now vote on the proposal
      // note need to destructure ID for the entry and view functions
      let id_num = guid::id_creation_num(&uid_of_transfer);

      // proposing is not same as voting, now eve votes
      // NOTE: there is a tx function that can propose and vote in single step
      donor_directed::vote_veto_tx(eve, donor_directed_address, id_num);

      let (approve_pct, req_threshold) = donor_directed_governance::get_veto_tally(donor_directed_address, guid::id_creation_num(&uid_of_transfer));

      assert!(approve_pct == 10000, 7357008);
      assert!(req_threshold == 5100, 7357009);

      // since Eve is the majority donor, and has almost all the votes, the voting will end early. On the next epoch the vote should end
      mock::trigger_epoch(root);
      donor_directed::vote_veto_tx(dave, donor_directed_address, id_num);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(donor_directed_address, &uid_of_transfer), 7357010);

      // should not be scheduled
      let (_found, _idx, state) = donor_directed::get_schedule_state(donor_directed_address, &uid_of_transfer);
      assert!(state == 2, 7357006); // is veto
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_process_unit(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000);

      let (_, bob_balance_pre) = ol_account::balance(@0x1000b);
      assert!(bob_balance_pre == 10000000, 7357001);

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_directed_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_directed_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_directed::find_by_deadline(donor_directed_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);

      // process epoch 3 accounts
      donor_directed::process_donor_directed_accounts(root, 3);

      let (_, bob_balance) = ol_account::balance(@0x1000b);
      assert!(bob_balance > bob_balance_pre, 7357005);
      assert!(bob_balance == 10000100, 7357006);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_process_epoch_boundary(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 10000000);

      let (_, bob_balance_pre) = ol_account::balance(@0x1000b);
      assert!(bob_balance_pre == 10000000, 7357001);

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_directed_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_directed_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, donor_directed_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(donor_directed_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_directed::is_scheduled(donor_directed_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_directed::find_by_deadline(donor_directed_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);

      // process epoch 3 accounts
      // donor_directed::process_donor_directed_accounts(root, 3);
      mock::trigger_epoch(root); // into epoch 1
      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment


      let (_, bob_balance) = ol_account::balance(@0x1000b);
      assert!(bob_balance > bob_balance_pre, 7357005);
      assert!(bob_balance == 10000100, 7357006);
    }


    #[test(root = @ol_framework, alice = @0x1000a, dave = @0x1000d, eve = @0x1000e)]
    fun dd_liquidate(root: &signer, alice: &signer, dave: &signer, eve: &signer) {
      // Scenario:
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account.
      // Dave and Eve are unhappy, and vote to liquidate the account.

      mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_directed_address = signer::address_of(&resource_sig);
      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000a, @0x1000b, @0x1000c, 2);

      // EVE Establishes some governance over the wallet, when donating.
      let eve_donation = 42;
      ol_account::transfer(eve, donor_directed_address, eve_donation);
      let (_, _, total_funds_sent_by_eve) = receipts::read_receipt(signer::address_of(eve), donor_directed_address);
      assert!(total_funds_sent_by_eve == eve_donation, 7357001);

      let is_donor = donor_directed_governance::check_is_donor(donor_directed_address, signer::address_of(eve));
      assert!(is_donor, 7357002);

      // Dave will also be a donor, with half the amount of what Eve sends
      ol_account::transfer(dave, donor_directed_address, 21);
      let is_donor = donor_directed_governance::check_is_donor(donor_directed_address, signer::address_of(dave));
      assert!(is_donor, 7357003);

      // Dave proposes to liquidate the account.
      donor_directed::propose_liquidation(dave, donor_directed_address);

      assert!(donor_directed_governance::is_liquidation_propsed(donor_directed_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_directed::vote_liquidation_tx(eve, donor_directed_address);

      mock::trigger_epoch(root);
      // needs a vote on the day after the threshold has passed
      donor_directed::vote_liquidation_tx(dave, donor_directed_address);

      let list = donor_directed::get_liquidation_queue();
      assert!(vector::length(&list) > 0, 7357005);

      let (addrs, refunds) = donor_directed::get_pro_rata(donor_directed_address);

      print(&addrs);
      print(&refunds);

      assert!(*vector::borrow(&addrs, 0) == @0x1000e, 7357006);
      assert!(*vector::borrow(&addrs, 1) == @0x1000d, 7357007);


      let eve_donation_pro_rata = vector::borrow(&refunds, 0);
      let superman_3 = 1; // rounding from fixed_point32
      assert!((*eve_donation_pro_rata + superman_3) == eve_donation, 7357008);

      let eve_donation_pro_rata = vector::borrow(&refunds, 1);
      let superman_3 = 1; // rounding from fixed_point32
      assert!((*eve_donation_pro_rata + superman_3) == 21, 7357009);

    }

}

