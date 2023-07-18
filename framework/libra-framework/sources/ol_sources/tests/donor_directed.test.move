
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
      let new_resource_address = signer::address_of(&resource_sig);
      assert!(resource_account::is_resource_account(new_resource_address), 7357003);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);


      let list = donor_directed::get_root_registry();
      assert!(vector::length(&list) == 1, 7357001);

      assert!(donor_directed::is_donor_directed(new_resource_address), 7357002);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dd_propose_payment(root: &signer, alice: &signer, bob: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let new_resource_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_propose_and_veto(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      mock::ol_initialize_coin_and_fund_vals(root, 100000);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let new_resource_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      // Establish some governance over the wallet. Alice is a
      // see cumu_deposits.test.move
      let carols_donation = 42;
      ol_account::transfer(carol, new_resource_address, 42); // this should track
      let (_, _, total_funds_sent_by_carol) = receipts::read_receipt(@0x1000c, new_resource_address);
      assert!(total_funds_sent_by_carol == carols_donation, 7357005); // last payment

      // let (_a, _b, _c) = Receipts::read_receipt(@Carol, @Alice);

      let uid_of_transfer = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      print(&uid_of_transfer);
      let (_found, _idx, _status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid_of_transfer);
      // assert!(found, 7357004);
      // assert!(idx == 0, 7357005);
      // assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);


      let is_donor = donor_directed_governance::check_is_donor(new_resource_address, signer::address_of(carol));
      assert!(is_donor, 7357009);
      // let guid = GUID::create_id(@Alice, 2);
      // print(&a);
      // need to destructure ID for the entry and view functions

      let _uid_of_veto_prop = donor_directed::propose_veto(carol, &uid_of_transfer);

      // now vote on the proposal
      let id_num = guid::id_creation_num(&uid_of_transfer);
      donor_directed::vote_veto_tx(carol, new_resource_address, id_num);

      // let has_veto = donor_directed_governance::has_veto(new_resource_address, guid::id_creation_num(&uid_of_veto_prop));
      // assert!(has_veto, 7357010); // propose does not cast a vote.


      // let (approve_pct, req_threshold) = donor_directed_governance::get_veto_tally(new_resource_address, guid::id_creation_num(&uid));

      // print(&approve_pct);
      // print(&req_threshold);

      // // it is not yet scheduled, it's still only a proposal by an admin
      // assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_schedule_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let new_resource_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_directed::find_by_deadline(new_resource_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);
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
      let new_resource_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, new_resource_address, 100);
      let (_, resource_balance) = ol_account::balance(new_resource_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_directed::find_by_deadline(new_resource_address, 3);
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
      let new_resource_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, new_resource_address, 100);
      let (_, resource_balance) = ol_account::balance(new_resource_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_directed::find_by_deadline(new_resource_address, 3);
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

}

