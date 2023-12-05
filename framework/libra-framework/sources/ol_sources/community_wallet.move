/// For an account to qualify for the Burn or Match recycling of accounts.

/// This module is used to dynamically check if an account qualifies for the CommunityWallet flag.

/// Community Wallet is a flag that can be applied to an account.
/// These accounts are voluntarily creating a number of restrictions and guarantees for users that interact with it.
/// In essence, a group of people may set up a wallet with these characteristics to provide funding for a common program.

/// For example the matching donation game, which validators provide with burns from their account will check that a destination account has a Community Wallet Flag.

/// The CommunityWallets will have the following properties enabled by their owners.


/// 0. This wallet is initialized as a donor_voice account. This means that it observes the policies of those accounts: namely, that the donors have Veto rights over the transactions which are proposed by the Owners of the account. Repeated rejections or an outright freeze poll, will prevent the Owners from transferring funds, and may ultimately revert the funds to a different community account (or burn).

/// !. They have instantiated a multi_action controller, which means that actions on this wallet can only be done by an n-of-m consensus by the authorities of the account. Plus, the nominal credentials which created the account cannot be used, since the keys will no longer be valid.

/// 2. The Multisig account holders do not have common ancestry. This is important to prevent an account holder from trivially creating sybil accounts to qualify as a community wallet. Sybils are possibly without common ancestry, but it is much harder.

/// 3. The multisig account has a minimum of 5 Authorities, and a threshold of 3 signatures. If there are more authorities, a 3/5 ratio or more should be preserved.

/// 4. CommunityWallets have a high threshold for sybils: all multisig authorities must be unrelated in their permission trees, per ancestry.

module ol_framework::community_wallet {
    use std::error;
    use std::vector;
    use std::signer;
    use std::option;
    use std::fixed_point32;
    use ol_framework::donor_voice;
    use ol_framework::multi_action;
    use ol_framework::ancestry;
    use ol_framework::match_index;
    use diem_framework::system_addresses;

    // use diem_std::debug::print;

    /// not authorized to operate on this account
    const ENOT_AUTHORIZED: u64 = 1;
    /// does not meet criteria for community wallet
    const ENOT_QUALIFY_COMMUNITY_WALLET: u64 = 2;
    /// This account needs to be donor directed.
    const ENOT_donor_voice: u64 = 3;
    /// This account needs a multisig enabled
    const ENOT_MULTISIG: u64 = 4;
    /// The multisig does not have minimum 5 signers and 3 approvals in config
    const ESIG_THRESHOLD: u64 = 5;
    /// The multisig threshold does not equal 3/5
    const ESIG_THRESHOLD_RATIO: u64 = 6;
    /// Signers may be sybil
    const ESIGNERS_SYBIL: u64 = 7;
    /// Recipient does not have a slow wallet
    const EPAYEE_NOT_SLOW_WALLET: u64 = 8;
    /// Too few signers for Match Index
    const ETOO_FEW_SIGNERS: u64 = 9;


    // A flag on the account that it wants to be considered a community wallet
    struct CommunityWallet has key { }

    public fun is_init(addr: address):bool {
      exists<CommunityWallet>(addr)
    }
    public fun set_comm_wallet(sender: &signer) {
      let addr = signer::address_of(sender);
      assert!(donor_voice::is_donor_voice(addr), error::invalid_state(ENOT_donor_voice));

      if (is_init(addr)) {
        move_to(sender, CommunityWallet{});
      }
    }

    public fun migrate_community_wallet_account(vm: &signer, dv_account:
    &signer) {
      system_addresses::assert_ol(vm);
      donor_voice::migrate_community_wallet_account(vm, dv_account);
      move_to(dv_account, CommunityWallet{});
    }

    /// Dynamic check to see if CommunityWallet is qualifying.
    /// if it is not qualifying it wont be part of the burn funds matching.
    public fun qualifies(addr: address): bool {
      // The CommunityWallet flag is set
      is_init(addr) &&
      // has donor_voice instantiated properly
      donor_voice::is_donor_voice(addr) &&
      donor_voice::is_liquidate_to_match_index(addr) &&
      // has multi_action instantialized
      multi_action::is_multi_action(addr) &&
      // multisig has minimum requirement of 3 signatures, and minimum list of 5 signers, and a minimum of 3/5 threshold. I.e. OK to have 4/5 signatures.
      multisig_thresh(addr)
      &&
      // the multisig authorities are unrelated per ancestry
      !multisig_common_ancestry(addr)
    }

    fun multisig_thresh(addr: address): bool{
      let (n, m) = multi_action::get_threshold(addr);

      // can't have less than three signatures
      if (n < 3) return false;
      // can't have less than five authorities
      // pentapedal locomotion https://www.youtube.com/watch?v=bgWJ9DN1Qak
      if (m < 5) return false;

      let r = fixed_point32::create_from_rational(3, 5);
      let pct_baseline = fixed_point32::multiply_u64(100, r);
      let r = fixed_point32::create_from_rational(n, m);
      let pct = fixed_point32::multiply_u64(100, r);

      pct > pct_baseline
    }

    fun multisig_common_ancestry(addr: address): bool {
      let list = multi_action::get_authorities(addr);

      let (fam, _, _) = ancestry::any_family_in_list(list);

      fam
    }

    /// check qualifications of community wallets
    /// need to check every epoch so that wallets who no longer qualify are not biasing the Match algorithm.
    public fun epoch_reset_ratios(root: &signer) {
      system_addresses::assert_ol(root);
      let good = get_qualifying();
      match_index::calc_ratios(root, good);
    }

    #[view]
    /// from the list of addresses that opted into the match_index, filter for only those that qualified.
    public fun get_qualifying(): vector<address> {
      let opt_in_list = match_index::get_address_list();

      vector::filter(opt_in_list, |addr|{
        qualifies(*addr)
      })
    }

    //////// MULTISIG TX HELPERS ////////

    // Helper to initialize the PaymentMultiAction but also while confirming that the signers are not related family
    // These transactions can be sent directly to donor_voice, but this is a helper to make it easier to initialize the multisig with the acestry requirements.

    public entry fun init_community(
      sig: &signer,
      init_signers: vector<address>
    ) {
      // policy is to have at least 5 signers as auths on the account.
      let len = vector::length(&init_signers);
      assert!(len > 4, error::invalid_argument(ETOO_FEW_SIGNERS));

      // enforce 3/5 multi auth
      let n = (3 * len) / 5;

      let (fam, _, _) = ancestry::any_family_in_list(*&init_signers);
      assert!(!fam, error::invalid_argument(ESIGNERS_SYBIL));

      // set as donor directed with any liquidation going to existing matching index

      donor_voice::make_donor_voice(sig, init_signers, n);
      donor_voice::set_liquidate_to_match_index(sig, true);
      match_index::opt_into_match_index(sig);
    }

    /// add signer to multisig, and check if they may be related in ancestry tree
    public entry fun add_signer_community_multisig(
      sig: &signer,
      multisig_address: address,
      new_signer: address,
      n_of_m: u64,
      vote_duration_epochs: u64
    ) {
      let current_signers = multi_action::get_authorities(multisig_address);
      let (fam, _, _) = ancestry::is_family_one_in_list(new_signer, &current_signers);

      assert!(!fam, error::invalid_argument(ESIGNERS_SYBIL));

      multi_action::propose_governance(
        sig,
        multisig_address,
        vector::singleton(new_signer),
        true, option::some(n_of_m),
        option::some(vote_duration_epochs)
      );
    }
}
