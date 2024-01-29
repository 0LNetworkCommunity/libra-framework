
#[test_only]

module ol_framework::test_donor_voice {
  use ol_framework::donor_voice;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use ol_framework::ancestry;
  use diem_framework::resource_account;
  use ol_framework::receipts;
  use ol_framework::donor_voice_governance;
  use ol_framework::community_wallet;
  use ol_framework::burn;
  use std::guid;
  use std::vector;
  use std::signer;

  use diem_std::debug::print;

    #[test(root = @ol_framework, alice = @0x1000a)]
    fun dd_init(root: &signer, alice: &signer) {
      let vals = mock::genesis_n_vals(root, 4);

      // Alice turns this account into a resource account. This can also happen after other donor directed initializations happen (or deposits). But must be complete before the donor directed wallet can be used.
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);
      assert!(resource_account::is_resource_account(donor_voice_address), 7357003);

      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);


      let list = donor_voice::get_root_registry();
      assert!(vector::length(&list) == 1, 7357001);

      assert!(donor_voice::is_donor_voice(donor_voice_address), 7357002);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dd_propose_payment(root: &signer, alice: &signer, bob: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);

      let uid = donor_voice::propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun dd_schedule_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      let vals = mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);

      let uid = donor_voice::propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);

      let uid = donor_voice::propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);

      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice::find_by_deadline(donor_voice_address, 3);
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

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);

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
      let uid_of_transfer = donor_voice::propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");

      let (_found, _idx, _status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid_of_transfer);
      assert!(!completed, 7357004);


      // Eve wants to propose a Veto, but this should fail at this, because
      // the tx is not yet scheduled
      let _uid_of_veto_prop = donor_voice::propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_voice_governance::tx_has_veto(donor_voice_address, guid::id_creation_num(&uid_of_transfer));
      assert!(!has_veto, 7357005); // propose does not cast a vote.


      // Now Carol, along with Bob, as admins have proposed the payment.
      // Now the payment should be scheduled
      let uid_of_transfer = donor_voice::propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (_found, _idx, state) = donor_voice::get_schedule_state(donor_voice_address, &uid_of_transfer);
      assert!(state == 1, 7357006); // is scheduled


      // Eve tries again after it has been scheduled
      let _uid_of_veto_prop = donor_voice::propose_veto(eve, &uid_of_transfer);
      let has_veto = donor_voice_governance::tx_has_veto(donor_voice_address, guid::id_creation_num(&uid_of_transfer));
      assert!(has_veto, 7357007); // propose does not cast a vote.

      // now vote on the proposal
      // note need to destructure ID for the entry and view functions
      let id_num = guid::id_creation_num(&uid_of_transfer);

      // proposing is not same as voting, now eve votes
      // NOTE: there is a tx function that can propose and vote in single step
      donor_voice::vote_veto_tx(eve, donor_voice_address, id_num);

      let (approve_pct, req_threshold) = donor_voice_governance::get_veto_tally(donor_voice_address, guid::id_creation_num(&uid_of_transfer));

      assert!(approve_pct == 10000, 7357008);
      assert!(req_threshold == 5100, 7357009);

      // since Eve is the majority donor, and has almost all the votes, the voting will end early. On the next epoch the vote should end
      mock::trigger_epoch(root);
      donor_voice::vote_veto_tx(dave, donor_voice_address, id_num);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice::is_scheduled(donor_voice_address, &uid_of_transfer), 7357010);

      // should not be scheduled
      let (_found, _idx, state) = donor_voice::get_schedule_state(donor_voice_address, &uid_of_transfer);
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

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_voice_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);

      let uid = donor_voice::propose_payment(bob, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);

      let uid = donor_voice::propose_payment(carol, donor_voice_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);

      // process epoch 3 accounts
      donor_voice::process_donor_voice_accounts(root, 3);

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


      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);

      // fund the account
      ol_account::transfer(alice, donor_voice_address, 100);
      let (_, resource_balance) = ol_account::balance(donor_voice_address);
      assert!(resource_balance == 100, 7357002);


      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);

      let uid = donor_voice::propose_payment(bob, donor_voice_address, signer::address_of(marlon_rando), 100, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);

      let uid = donor_voice::propose_payment(carol, donor_voice_address, signer::address_of(marlon_rando), 100, b"thanks marlon");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(donor_voice_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice::is_scheduled(donor_voice_address, &uid), 7357008);

      // PROCESS THE PAYMENT
      // the default timed payment is 3 epochs, we are in epoch 1
      let list = donor_voice::find_by_deadline(donor_voice_address, 3);
      assert!(vector::contains(&list, &uid), 7357009);

      // process epoch 3 accounts
      // donor_voice::process_donor_voice_accounts(root, 3);
      mock::trigger_epoch(root); // into epoch 1
      mock::trigger_epoch(root); // into epoch 2
      mock::trigger_epoch(root); // into epoch 3, processes at the end of this epoch.
      mock::trigger_epoch(root); // epoch 4 should include the payment

      let (_bal, marlon_rando_balance_post) = ol_account::balance(signer::address_of(marlon_rando));

      assert!(marlon_rando_balance_post == marlon_rando_balance_pre + 100, 7357006);
    }


    #[test(root = @ol_framework, alice = @0x1000a, dave = @0x1000d, eve = @0x1000e)]
    fun dd_liquidate_to_donor(root: &signer, alice: &signer, dave: &signer, eve: &signer) {
      // Scenario:
      // Alice creates a donor directed account where Alice, Bob and Carol, are admins.
      // Dave and Eve make a donation and so are able to have some voting on that account.
      // Dave and Eve are unhappy, and vote to liquidate the account.

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause
      // issues
      mock::trigger_epoch(root);
      // a burn happend in the epoch above, so let's compare it to end of epoch
      let (lifetime_burn_pre, _) = burn::get_lifetime_tracker();

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);
      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);
      // donor_voice::set_liquidate_to_match_index(&resource_sig, true);

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
      donor_voice::propose_liquidation(dave, donor_voice_address);

      assert!(donor_voice_governance::is_liquidation_propsed(donor_voice_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_voice::vote_liquidation_tx(eve, donor_voice_address);

      mock::trigger_epoch(root);
      // needs a vote on the day after the threshold has passed
      donor_voice::vote_liquidation_tx(dave, donor_voice_address);

      let list = donor_voice::get_liquidation_queue();
      assert!(vector::length(&list) > 0, 7357005);

      let (addrs, refunds) = donor_voice::get_pro_rata(donor_voice_address);

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

      donor_voice::test_helper_vm_liquidate(root);

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

      let vals = mock::genesis_n_vals(root, 5); // need to include eve to init funds
      mock::ol_initialize_coin_and_fund_vals(root, 100000, true);
      // start at epoch 1, since turnout tally needs epoch info, and 0 may cause issues
      mock::trigger_epoch(root);
      // a burn happend in the epoch above, so let's compare it to end of epoch
      let (lifetime_burn_pre, _) = burn::get_lifetime_tracker();

      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let donor_voice_address = signer::address_of(&resource_sig);
      // the account needs basic donor directed structs
      donor_voice::make_donor_voice(&resource_sig, vals, 2);
      donor_voice::set_liquidate_to_match_index(&resource_sig, true);

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
      donor_voice::propose_liquidation(dave, donor_voice_address);

      assert!(donor_voice_governance::is_liquidation_propsed(donor_voice_address), 7357004);

      // Dave proposed, now Dave and Eve need to vote
      donor_voice::vote_liquidation_tx(eve, donor_voice_address);

      mock::trigger_epoch(root);
      // needs a vote on the day after the threshold has passed
      donor_voice::vote_liquidation_tx(dave, donor_voice_address);

      let list = donor_voice::get_liquidation_queue();
      assert!(vector::length(&list) > 0, 7357005);

      // check the table of refunds
      // dave and eve should be receiving
      let (addrs, refunds) = donor_voice::get_pro_rata(donor_voice_address);

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

      donor_voice::test_helper_vm_liquidate(root);

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

    #[test(root = @ol_framework, community = @0x10011, alice = @0x1000a, bob =
    @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e, joan = @0x1000f)]
    fun migrate_cw_bug(root: &signer, community: &signer, alice: &signer, bob:
    &signer, carol: &signer, dave: &signer, eve: &signer) {
      mock::genesis_n_vals(root, 5);

      // create resource account for community wallet
      let (comm_resource_sig, _cap) = ol_account::ol_create_resource_account(community, b"senior sword vocal target latin rug ship summer bar suit cake derive pluck sunset can scorpion tornado hungry erosion heart away suggest glad seek");
      let community_wallet_resource_address = signer::address_of(&comm_resource_sig);

      let community_wallet_address = signer::address_of(community);
      
      // verify resource account was created succesfully for the community wallet
      assert!(resource_account::is_resource_account(community_wallet_resource_address), 7357003);
      assert!(!community_wallet::is_init(community_wallet_address), 100); //TODO: find appropriate error codes

      // migrate community wallet
      community_wallet::migrate_community_wallet_account(root, community);

      // verify correct migration of community wallet
      assert!(community_wallet::is_init(community_wallet_address), 101); //TODO: find appropriate error codes


      let _b = donor_voice::is_liquidate_to_match_index(community_wallet_address);

      // create vectors of multi signers for the community wallet and set to different ancestrys
      let alice_addr = signer::address_of(alice);
      let bob_addr = signer::address_of(bob);
      let carol_addr = signer::address_of(carol);
      let dave_addr = signer::address_of(dave);
      let eve_addr = signer::address_of(eve);

      let addrs = vector::singleton(alice_addr);
      vector::push_back(&mut addrs, bob_addr);
      vector::push_back(&mut addrs, carol_addr);
      vector::push_back(&mut addrs, dave_addr);
      vector::push_back(&mut addrs, eve_addr);

      ancestry::fork_migrate(root, alice, vector::singleton(alice_addr));
      ancestry::fork_migrate(root, bob, vector::singleton(bob_addr));
      ancestry::fork_migrate(root, carol, vector::singleton(carol_addr));
      ancestry::fork_migrate(root, dave, vector::singleton(dave_addr));
      ancestry::fork_migrate(root, eve, vector::singleton(eve_addr));

      // wake up and initialize community wallet as a multi sig, donor voice account 
      community_wallet::init_community(&comm_resource_sig, addrs);

      // verify wallet is a initialized correctly
      assert!(donor_voice::is_donor_voice(signer::address_of(&comm_resource_sig)), 102);


      // make a transaction and have signers execute  
      mock::trigger_epoch(root); // into epoch 1

      let uid = donor_voice::propose_payment(bob, community_wallet_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(community_wallet_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice::is_scheduled(community_wallet_resource_address, &uid), 7357008);

      donor_voice::propose_payment(carol, community_wallet_resource_address, @0x1000b, 100, b"thanks bob");
      let uid = donor_voice::propose_payment(alice, community_wallet_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_voice::get_multisig_proposal_state(community_wallet_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007); // now completed

      // confirm it is scheduled
      assert!(donor_voice::is_scheduled(community_wallet_resource_address, &uid), 7357008);

      // the default timed payment is 4 epochs, we are in epoch 2
      let list = donor_voice::find_by_deadline(community_wallet_resource_address, 4);
      assert!(vector::contains(&list, &uid), 7357009);


    }
}
