#[test_only]
module ol_framework::test_donor_voice_governance {
    use std::signer;
    use std::vector;
    use ol_framework::mock;
    use ol_framework::donor_voice_governance;
    use ol_framework::donor_voice_txs;
    use ol_framework::donor_voice_reauth;

    #[test(framework = @ol_framework, marlon_sponsor = @0x1234)]
    fun reauthorize_tx(framework: &signer, marlon_sponsor: &signer) {
      let _vals = mock::genesis_n_vals(framework, 3);
      mock::ol_initialize_coin_and_fund_vals(framework, 100000, true);

      let (dv_sig, _admin_sigs, donor_sigs) = mock::mock_dv(framework, marlon_sponsor, 2);

      let dv_address = signer::address_of(&dv_sig);
      donor_voice_reauth::test_set_requires_reauth(framework, dv_address);
      assert!(donor_voice_reauth::flagged_for_reauthorization(dv_address), 7357001);

      // assert!(donor_voice_governance::is_reauth_proposed(dv_address), 7357000);

      let prev_turnout_percent = 0;
      let ballot_id = 0;
      // reauthorize tx by each of the donors
      let i = 0;
      while (i < vector::length(&donor_sigs)) {
        let donor_sig = vector::borrow(&donor_sigs, i);
        donor_voice_txs::vote_reauth_tx(donor_sig, dv_address);

        let (pending, _, _ ) = donor_voice_governance::get_reauth_ballots(dv_address);

        // find the ballot id while it's pending
        if (vector::length(&pending) > 0) {
          ballot_id = *vector::borrow(&pending, 0);
        };

        let (_percent_approval, turnout_percent, _threshold_needed_to_pass, _epoch_deadline, _minimum_turnout_required, _is_complete, _approved) =  donor_voice_governance::get_reauth_tally(dv_address, ballot_id);

        assert!(turnout_percent >= prev_turnout_percent, 7357000);
        prev_turnout_percent = turnout_percent;

        i = i + 1;
      };

      let (percent_approval, turnout_percent, threshold_needed_to_pass, epoch_deadline, minimum_turnout_required, _is_complete, _approved) =  donor_voice_governance::get_reauth_tally(dv_address, ballot_id);

      assert!(percent_approval == 10000, 7357001);
      assert!(turnout_percent == 6666, 7357002);
      assert!(threshold_needed_to_pass == 6461, 7357003);
      assert!(epoch_deadline == 30, 7357004);
      assert!(minimum_turnout_required == 1250, 7357005);
    }

    #[test(framework = @ol_framework, marlon_sponsor = @0x1234)]
    fun reauthorize_poll_closes(framework: &signer, marlon_sponsor: &signer) {
      let _vals = mock::genesis_n_vals(framework, 3);
      mock::ol_initialize_coin_and_fund_vals(framework, 100000, true);

      let (dv_sig, _admin_sigs, donor_sigs) = mock::mock_dv(framework, marlon_sponsor, 2);

      let dv_address = signer::address_of(&dv_sig);
      donor_voice_reauth::test_set_requires_reauth(framework, dv_address);
      assert!(donor_voice_reauth::flagged_for_reauthorization(dv_address), 7357001);
      // reauthorize tx by each of the donors
      let i = 0;
      while (i < vector::length(&donor_sigs)) {
        let donor_sig = vector::borrow(&donor_sigs, i);
        donor_voice_txs::vote_reauth_tx(donor_sig, dv_address);
        i = i + 1;
      };

      let (pending, _, _ ) = donor_voice_governance::get_reauth_ballots(dv_address);

      let pending_id = vector::borrow(&pending, 0);
      let (percent_approval, _turnout_percent, _threshold_needed_to_pass, epoch_deadline, _minimum_turnout_required, is_complete, _approved) =  donor_voice_governance::get_reauth_tally(dv_address, *pending_id);

      assert!(percent_approval == 10000, 7357001);
      assert!(epoch_deadline == 30, 7357002);
      assert!(is_complete == true, 7357003);

      let (pending, _, _ ) = donor_voice_governance::get_reauth_ballots(dv_address);
      let pending_id = vector::borrow(&pending, 0);

      // tally result also shows complete
      let (_percent_approval, _turnout_percent, _threshold_needed_to_pass, _epoch_deadline, _minimum_turnout_required, is_complete, approved) =  donor_voice_governance::get_reauth_tally(dv_address, *pending_id);

      assert!(is_complete == true, 7357004);
      assert!(approved == true, 7357005);


      // great, the CW should now be operational
      donor_voice_reauth::assert_authorized(dv_address);
    }

    #[test(framework = @ol_framework, marlon_sponsor = @0x1234)]
    fun turnout_tally_allows_duplicate_tx(framework: &signer, marlon_sponsor: &signer) {
      let _vals = mock::genesis_n_vals(framework, 3);
      mock::ol_initialize_coin_and_fund_vals(framework, 100000, true);

      let (dv_sig, _admin_sigs, donor_sigs) = mock::mock_dv(framework, marlon_sponsor, 2);

      let dv_address = signer::address_of(&dv_sig);
      donor_voice_reauth::test_set_requires_reauth(framework, dv_address);
      assert!(donor_voice_reauth::flagged_for_reauthorization(dv_address), 7357001);

      // reauthorize tx by a donor
      let donor_sig = vector::borrow(&donor_sigs, 0);
      donor_voice_txs::vote_reauth_tx(donor_sig, dv_address);
      // again
      let donor_sig = vector::borrow(&donor_sigs, 0);
      donor_voice_txs::vote_reauth_tx(donor_sig, dv_address);


      let (pending, _, _ ) = donor_voice_governance::get_reauth_ballots(dv_address);
      let pending_id = vector::borrow(&pending, 0);
      let (percent_approval, _turnout_percent, _threshold_needed_to_pass, _epoch_deadline,_minimum_turnout_required, _is_complete, _approved) =  donor_voice_governance::get_reauth_tally(dv_address, *pending_id);

      assert!(percent_approval == 10000, 7357001);
    }
}
