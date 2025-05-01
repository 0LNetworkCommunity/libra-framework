/// simple move module to check if authorization for v8
module ol_framework::reauthorization {
    use std::error;
    use ol_framework::activity;
    use ol_framework::community_wallet;
    use ol_framework::donor_voice_reauth;
    use ol_framework::founder;


    /// user account never migrated from V7 to V8
    const ENOT_MIGRATED_FROM_V7: u64 = 1;

    /// this is a pre-v8 "founder" account, but it has insufficient vouches
    const EFOUNDER_HAS_NO_FRIENDS: u64 = 2;

    /// this community wallet has not been reauthorized by its donors
    const ECOMMUNITY_WALLET_HAS_NO_AUTH: u64 = 1;

    public fun assert_v8_authorized(account: address) {

      let migrated = activity::is_initialized(account);
      let founder = founder::is_founder(account);
      let is_cw = community_wallet::is_init(account);

      if (is_cw) {
        donor_voice_reauth::assert_authorized(account)
      };
      // case 1: not migrated from V7
      if(!founder && !is_cw) {
        assert!(migrated, error::invalid_state(ENOT_MIGRATED_FROM_V7));
      } else if(founder && migrated && !is_cw) {
        assert!(founder::has_friends(account), error::invalid_state(EFOUNDER_HAS_NO_FRIENDS));
      }
    }

    #[view]
    public fun is_v8_authorized(account: address): bool {
      // Every newly create account since v8 will have the Activity struct.
      // Check if this is a case of an account that has yet to migrate from V7.
      // This also applies to donor-voice accounts.
      if (!activity::is_initialized(account)) {
        return false
      };

      // first deal with the special case of the community wallet
      // case 1: a community wallet which was reauthorized
      if(community_wallet::is_init(account)) {
        return donor_voice_reauth::is_authorized(account)
      };

      // case 2: deal with v7 accounts "founder"
      // and check that they are not bots
      if (founder::is_founder(account)) {
        return founder::has_friends(account)
      };


      // case 3: user onboarded since v8 started
      // check only if the account is malformed
      // i.e. has an activity struct but no activity

      if (!activity::is_prehistoric(account)) {
        return true
      };

      false
    }
}
