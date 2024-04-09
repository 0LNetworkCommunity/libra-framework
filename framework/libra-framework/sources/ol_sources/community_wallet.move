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

/// MINIMUM_SIGS. The multisig account has a minimum of MINIMUM_AUTH Authorities, and a threshold of MINIMUM_SIGS signatures. If there are more authorities, a MINIMUM_SIGS/MINIMUM_AUTH ratio or more should be preserved.

/// 4. CommunityWallets have a high threshold for sybils: all multisig authorities must be unrelated in their permission trees, per ancestry.

module ol_framework::community_wallet {
    use std::signer;

    friend ol_framework::community_wallet_init;

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
    /// The multisig threshold does not equal MINIMUM_SIGS/MINIMUM_AUTH
    const ESIG_THRESHOLD_RATIO: u64 = 6;
    /// Signers may be sybil
    const ESIGNERS_SYBIL: u64 = 7;


    // STATICS
    /// minimum n signatures for a transaction
    const MINIMUM_SIGS: u64 = 2;
    /// minimum m authorities for a wallet
    const MINIMUM_AUTH: u64 = 3;

    // A flag on the account that it wants to be considered a community wallet
    // this is PERMANENT
    struct CommunityWallet has key { }

    #[view]
    public fun is_init(addr: address):bool {
      exists<CommunityWallet>(addr)
    }

    public(friend) fun set_comm_wallet(sender: &signer) {
      let addr = signer::address_of(sender);
      if (!is_init(addr)) {
        move_to(sender, CommunityWallet{});
      }
    }
}
