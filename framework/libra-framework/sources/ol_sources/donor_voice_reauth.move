/// Periodically, Donor Voice accounts, (a.k.a Community Wallets) must
/// be re-authorized by their donors. See docs in donor_voice.move for more details.

module ol_framework::donor_voice_reauth {
    use std::error;
    use std::signer;
    use std::timestamp;
    use diem_framework::account;
    use ol_framework::activity;

    // use std::debug::print;

    friend ol_framework::donor_voice_txs;
    friend ol_framework::donor_voice_governance;
    #[test_only]
    friend ol_framework::test_community_wallet;
    #[test_only]
    friend ol_framework::test_donor_voice;
    /// ERROR CODES
    /// Donor Voice account has not been reauthorized and the donor authorization expired.
    const EDONOR_VOICE_AUTHORITY_EXPIRED: u64 = 0;

    /// CONSTANTS
    /// Seconds in one year
    const SECONDS_IN_YEAR: u64 = 31536000;
    /// Window which Donor Voice accounts are be reauthorized
    const YEARS_AUTHORIZE_WINDOW: u64 = 5;


    struct DonorAuthorized has key {
      timestamp: u64,
    }

    /// Private implementation to reauthorize with timestamp, after governance concludes.
    /// This creates the state if it does not yet exist. Does not assume it is authorized (timestamp == 0).
    public(friend) fun maybe_init(dv_signer: &signer) acquires DonorAuthorized {
      if (!exists<DonorAuthorized>(signer::address_of(dv_signer))) {
        move_to<DonorAuthorized>(dv_signer, DonorAuthorized {
          timestamp: 0
        });
      } else {
        let state = borrow_global_mut<DonorAuthorized>(signer::address_of(dv_signer));
        state.timestamp = 0;
      }
    }


    /// With the account tx capability, reauthorize the account.
    /// This is a public function, but only the Donor Voice Txs can call it, while owning the GUID capability.
    public(friend) fun reauthorize_now(cap: &account::GUIDCapability) acquires DonorAuthorized {
      let now = timestamp::now_seconds();
      let dv_account = account::get_guid_capability_address(cap);

      let state = borrow_global_mut<DonorAuthorized>(dv_account);
      state.timestamp = now;
    }

    /// force the abort if not authorized
    public(friend) fun assert_authorized(dv_account: address) acquires DonorAuthorized {
      assert!(is_authorized(dv_account), error::invalid_state(EDONOR_VOICE_AUTHORITY_EXPIRED));
    }

    #[view]
    /// Checks if there is a DonorAuthorized state, and if the timestamp
    /// is within the YEARS_AUTHORIZE_WINDOW.
    public fun is_within_authorize_window(dv_account: address): bool acquires DonorAuthorized {
      let now = timestamp::now_seconds();
      let five_years_secs = YEARS_AUTHORIZE_WINDOW * SECONDS_IN_YEAR;
      let start_authorize_window = 0;
      if (now > five_years_secs) {
        start_authorize_window = now - five_years_secs
      };
      let state = borrow_global_mut<DonorAuthorized>(dv_account);
      // note: greater than or equal for test cases
      if (state.timestamp > start_authorize_window) {
        return true
      };
      false
    }

    #[view]
    /// Checks if there is a DonorAuthorized state, and if the timestamp

    public fun has_activity_in_last_year(dv_account: address): bool {

      let latest_tx = activity::get_last_activity_usecs(dv_account);
      let now = timestamp::now_seconds();

      let one_year_ago = 0;
      if (now > SECONDS_IN_YEAR) {
        one_year_ago = now - SECONDS_IN_YEAR
      };

      if (latest_tx > one_year_ago) {
        return true
      };
      false
    }

    #[view]
    /// Account authorized to be active
    public fun is_authorized(dv_account: address): bool acquires DonorAuthorized {
      if (!exists<DonorAuthorized>(dv_account)) {
        return false
      };
      is_within_authorize_window(dv_account) &&
      has_activity_in_last_year(dv_account)
    }

    ///////// TEST HELPERS ////////
    #[test_only]
    public(friend) fun test_set_authorized(framework: &signer, dv_account: address) acquires DonorAuthorized{
      diem_framework::system_addresses::assert_diem_framework(framework);

      let now = timestamp::now_seconds() + 1;
      let state = borrow_global_mut<DonorAuthorized>(dv_account);
      state.timestamp = now;

      activity::test_set_activity(framework, dv_account, now);

      assert_authorized(dv_account);
    }
}
