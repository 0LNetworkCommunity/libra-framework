#[test_only]
module ol_framework::test_donor_voice_governance {
    // use std::signer;
    // use std::vector;

    use ol_framework::mock;
    // use ol_framework::ol_account;
    // use ol_framework::receipts;
    // use ol_framework::donor_voice_txs;
    // use ol_framework::donor_voice_governance;
    // use ol_framework::multi_action;

    #[test(framework = @ol_framework, marlon_sponsor = @0x1234)]
    fun reauthorize_tx(framework: &signer, marlon_sponsor: &signer) {
      let _vals = mock::genesis_n_vals(framework, 3);
      mock::ol_initialize_coin_and_fund_vals(framework, 100000, true);

      let (_dv_sig, _admin_sigs, _donor_sigs) = mock::mock_dv(framework, marlon_sponsor, 2);
    }

}
