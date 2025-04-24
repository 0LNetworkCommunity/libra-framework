#[test_only]
module ol_framework::test_donor_voice_governance {
    use std::signer;
    use std::guid;

    use ol_framework::mock;
    use ol_framework::ol_account;
    use ol_framework::receipts;
    use ol_framework::donor_voice_txs;
    use ol_framework::donor_voice_governance;
    use ol_framework::multi_action;

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    fun dv_propose_reauth(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer) {
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
      assert!(donor_voice_txs::is_scheduled(donor_voice_address, &uid_of_transfer), 7357006); // is scheduled

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

      let (approve_pct, _turnout, req_threshold) = donor_voice_governance::get_veto_tally(donor_voice_address, guid::id_creation_num(&uid_of_transfer));

      assert!(approve_pct == 10000, 7357008);
      assert!(req_threshold == 5100, 7357009);

      // since Eve is the majority donor, and has almost all the votes, the voting will end early. On the next epoch the vote should end
      mock::trigger_epoch(root);
      donor_voice_txs::vote_veto_tx(dave, donor_voice_address, id_num);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_voice_txs::is_scheduled(donor_voice_address,
      &uid_of_transfer), 7357010);

      // it's vetoed
      assert!(donor_voice_txs::is_veto(donor_voice_address, &uid_of_transfer), 7357011);
    }
}
