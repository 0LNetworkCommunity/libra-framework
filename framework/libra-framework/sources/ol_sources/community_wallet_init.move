/// For an account to qualify for the Burn or Match recycling of accounts.

/// Entry functions for community wallet, so that we don't cause dependency cycles

module ol_framework::community_wallet_init {
    use std::error;
    use std::vector;
    use std::signer;
    use std::option;
    use std::fixed_point32;
    use ol_framework::donor_voice_txs;
    use ol_framework::multi_action;
    use ol_framework::ancestry;
    use ol_framework::match_index;
    use ol_framework::community_wallet;


    // use diem_std::debug::print;

    /// not authorized to operate on this account
    const ENOT_AUTHORIZED: u64 = 1;
    /// does not meet criteria for community wallet
    const ENOT_QUALIFY_COMMUNITY_WALLET: u64 = 2;
    /// Recipient does not have a slow wallet
    const EPAYEE_NOT_SLOW_WALLET: u64 = 8;
    /// Too few signers for Match Index
    const ETOO_FEW_SIGNERS: u64 = 9;
    /// This account needs to be donor directed.
    const ENOT_DONOR_VOICE: u64 = 3;
    /// This account needs a multisig enabled
    const ENOT_MULTISIG: u64 = 4;
    /// The multisig does not have minimum MINIMUM_AUTH signers and MINIMUM_SIGS approvals in config
    const ESIG_THRESHOLD: u64 = 5;
    /// The multisig threshold is not better than MINIMUM_SIGS/MINIMUM_AUTH
    const ESIG_THRESHOLD_RATIO: u64 = 6;
    /// Signers may be sybil
    const ESIGNERS_SYBIL: u64 = 7;
    /// does not liquidate to match index
    const ENOT_MATCH_INDEX_LIQ: u64 = 8;
    /// does not have the community wallet flag
    const ENO_CW_FLAG: u64 = 9;

    // STATICS
    /// minimum n signatures for a transaction
    const MINIMUM_SIGS: u64 = 2;
    /// minimum m authorities for a wallet
    const MINIMUM_AUTH: u64 = 3;


    #[test_only]
    public fun migrate_community_wallet_account(vm: &signer, dv_account:
    &signer) {
      use diem_framework::system_addresses;
      system_addresses::assert_ol(vm);
      donor_voice_txs::migrate_community_wallet_account(vm, dv_account);
      community_wallet::set_comm_wallet(dv_account);
    }

    //////// MULTISIG TX HELPERS ////////

    // Helper to initialize the PaymentMultiAction but also while confirming that the signers are not related family
    // These transactions can be sent directly to donor_voice, but this is a helper to make it easier to initialize the multisig with the acestry requirements.

    public entry fun init_community(
      sig: &signer,
      init_signers: vector<address>
    ) {
      // policy is to have at least m signers as auths on the account.
      let len = vector::length(&init_signers);
      assert!(len >= MINIMUM_AUTH, error::invalid_argument(ETOO_FEW_SIGNERS));

      // enforce n/m multi auth
      let n = if (len == 3) { 2 }
      else {
        (MINIMUM_SIGS * len) / MINIMUM_AUTH
      };

      let (fam, _, _) = ancestry::any_family_in_list(*&init_signers);
      assert!(!fam, error::invalid_argument(ESIGNERS_SYBIL));

      // set as donor directed with any liquidation going to contemporary
      // matching index (not liquidated to historical community wallets)

      donor_voice_txs::make_donor_voice(sig, init_signers, n);
      if (!donor_voice_txs::is_liquidate_to_match_index(signer::address_of(sig))) {
        donor_voice_txs::set_liquidate_to_match_index(sig, true);
      };
      match_index::opt_into_match_index(sig);

      community_wallet::set_comm_wallet(sig);
    }

    /// convenience function to check if the account can be caged
    /// after all the structs are in place
    public entry fun finalize_and_cage(sig: &signer, initial_authorities: vector<address>, num_signers: u64) {
      let addr = signer::address_of(sig);
      assert!(community_wallet::is_init(addr), error::invalid_argument(ENO_CW_FLAG));
      assert!(donor_voice_txs::is_liquidate_to_match_index(addr), error::invalid_argument(ENOT_MATCH_INDEX_LIQ));

      multi_action::finalize_and_cage(sig, initial_authorities, num_signers);

      assert!(multisig_thresh(addr), error::invalid_argument(ESIG_THRESHOLD_RATIO));
      assert!(!multisig_common_ancestry(addr), error::invalid_argument(ESIGNERS_SYBIL));

    }

    #[view]

    /// Dynamic check to see if CommunityWallet is qualifying.
    /// if it is not qualifying it wont be part of the burn funds matching.
    public fun qualifies(addr: address): bool {
      // The CommunityWallet flag is set
      community_wallet::is_init(addr) &&
      // has donor_voice instantiated properly
      donor_voice_txs::is_donor_voice(addr) &&
      donor_voice_txs::is_liquidate_to_match_index(addr) &&

      // multisig has minimum requirement of MINIMUM_SIGS signatures, and minimum list of MINIMUM_AUTH signers, and a minimum of MINIMUM_SIGS/MINIMUM_AUTH threshold. I.e. OK to have MINIMUM_SIGS+1/MINIMUM_AUTH signatures.
      multisig_thresh(addr)
      &&
      // the multisig authorities are unrelated per ancestry
      !multisig_common_ancestry(addr)
    }


    /// Dynamic check to see if CommunityWallet is qualifying.
    /// if it is not qualifying it wont be part of the burn funds matching.
    public fun assert_qualifies(addr: address) {
      // The CommunityWallet flag is set
      assert!(community_wallet::is_init(addr), error::invalid_argument(ENO_CW_FLAG));
      assert!(donor_voice_txs::is_donor_voice(addr), error::invalid_argument(ENOT_DONOR_VOICE));
      assert!(donor_voice_txs::is_liquidate_to_match_index(addr), error::invalid_argument(ENOT_MATCH_INDEX_LIQ));
      assert!(multisig_thresh(addr), error::invalid_argument(ESIG_THRESHOLD_RATIO));
      assert!(!multisig_common_ancestry(addr), error::invalid_argument(ESIGNERS_SYBIL));
    }

    fun multisig_thresh(addr: address): bool{
      let (n, m) = multi_action::get_threshold(addr);

      // can't have less than three signatures
      if (n < MINIMUM_SIGS) return false;
      // can't have less than five authorities
      // pentapedal locomotion https://www.youtube.com/watch?v=bgWJ9DN1Qak
      if (m < MINIMUM_AUTH) return false;

      let r = fixed_point32::create_from_rational(MINIMUM_SIGS, MINIMUM_AUTH);
      let pct_baseline = fixed_point32::multiply_u64(100, r);
      let r = fixed_point32::create_from_rational(n, m);
      let pct = fixed_point32::multiply_u64(100, r);

      pct >= pct_baseline
    }

    fun multisig_common_ancestry(addr: address): bool {
      let list = multi_action::get_authorities(addr);

      let (fam, _, _) = ancestry::any_family_in_list(list);

      fam
    }

    // /// check qualifications of community wallets
    // /// need to check every epoch so that wallets who no longer qualify are not biasing the Match algorithm.
    // public fun epoch_reset_ratios(root: &signer) {
    //   system_addresses::assert_ol(root);
    //   let good = get_qualifying();
    //   match_index::calc_ratios(root, good);
    // }

    #[view]
    /// from the list of addresses that opted into the match_index, filter for only those that qualified.
    public fun get_qualifying(opt_in_list: vector<address>): vector<address> {
      // let opt_in_list = match_index::get_address_list();

      vector::filter(opt_in_list, |addr|{
        qualifies(*addr)
      })
    }

    #[view]
    /// Get authorities resposible for the community wallet
    public fun get_community_wallet_authorities(multisig_address: address): vector<address> {
      multi_action::get_authorities(multisig_address)
    }

    /// Add or remove a signer to/from the multisig, and check if they may be related in the ancestry tree
    public entry fun change_signer_community_multisig(
      sig: &signer,
      multisig_address: address,
      new_signer: address,
      is_add_operation: bool,
      n_of_m: u64,
      vote_duration_epochs: u64
    ) {
      assert!(n_of_m >= MINIMUM_SIGS , error::invalid_argument(ETOO_FEW_SIGNERS));

      let current_signers = multi_action::get_authorities(multisig_address);

      // Checking family relations only if adding a new signer
      if (is_add_operation) {
          let (fam, _, _) = ancestry::is_family_one_in_list(new_signer, &current_signers);
          assert!(!fam, error::invalid_argument(ESIGNERS_SYBIL));
      };

      // Verify the signers will not fall below the threshold the signers will fall below threshold
      if (!is_add_operation) {
          assert!((vector::length(&current_signers) - 1) >  MINIMUM_AUTH, error::invalid_argument(ESIG_THRESHOLD));
      };

      multi_action::propose_governance(
        sig,
        multisig_address,
        vector::singleton(new_signer),
        is_add_operation,
        option::some(n_of_m),
        option::some(vote_duration_epochs)
      );
    }
}
