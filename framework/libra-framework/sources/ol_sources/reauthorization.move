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

      let migrated = activity::has_ever_been_touched(account);
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
      // case 1: user onboarded since v8 started
      // has activity struct, and not founder, or community wallet
      if (
        activity::has_ever_been_touched(account) &&
        !founder::is_founder(account) &&
        !community_wallet::is_init(account)
      ) {
        return true
      };

      // case 2: a friendly founder with struct and vouches
      if (
        activity::has_ever_been_touched(account) &&
        founder::is_founder(account) &&
        !community_wallet::is_init(account)
      ) {

        return founder::has_friends(account)
      };
      // case 3: a community wallet which was reauthorized
      if(community_wallet::is_init(account)) {
        return donor_voice_reauth::is_authorized(account)
      };

      false
    }
}
