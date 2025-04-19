/// Periodically, Donor Voice accounts, (a.k.a Community Wallets) must
/// be re-authorized by their donors. See docs in donor_voice.move for more details.
///
/// Why this matters:
/// Community wallets are intended to be active participants in the ecosystem, consistently
/// executing their programs and supporting their causes. However, over time, some community
/// wallets may become dormant due to various reasons: loss of interest, departure of key
/// stakeholders, or simply being forgotten. This reauthorization mechanism creates a protocol
/// to periodically validate that community wallets are still actively managed, requiring
/// explicit reauthorization every 5 years, or after 1 year of complete inactivity. This
/// ensures funds are not locked indefinitely in abandoned wallets and provides donors with
/// confidence that their contributions remain under proper governance and stewardship.

module ol_framework::donor_voice_reauth {
    use std::error;
    use std::signer;
    use std::timestamp;
    use diem_framework::account;
    use ol_framework::activity;

    friend ol_framework::donor_voice_txs;
    friend ol_framework::donor_voice_governance;
    friend ol_framework::reauthorization;

    #[test_only]
    friend ol_framework::test_community_wallet;
    #[test_only]
    friend ol_framework::test_donor_voice;
    /// ERROR CODES

    /// Donor Voice authority not initialized
    const ENOT_INITIALIZED: u64 = 0;
    /// Authority expired after the 5 year window
    const EDONOR_VOICE_AUTHORITY_EXPIRED: u64 = 1;
    /// No activity in past year, reauthorization pending
    const ENO_YEARLY_ACTIVITY: u64 = 2;

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

       assert!(exists<DonorAuthorized>(dv_account), error::invalid_state(ENOT_INITIALIZED));
       assert!(!authorization_expired(dv_account), error::invalid_state(EDONOR_VOICE_AUTHORITY_EXPIRED));
       assert!(has_activity_in_last_year(dv_account), error::invalid_state(ENO_YEARLY_ACTIVITY));
    }

    #[view]
    /// Checks if there is a DonorAuthorized state, and if the timestamp
    /// is within the YEARS_AUTHORIZE_WINDOW.
    public fun authorization_expired(dv_account: address): bool acquires DonorAuthorized {
      let five_years_secs = YEARS_AUTHORIZE_WINDOW * SECONDS_IN_YEAR;
      let now = timestamp::now_seconds();
      let five_years_ago = if (now > five_years_secs) {
        now - five_years_secs
      } else { 0 };
      let state = borrow_global_mut<DonorAuthorized>(dv_account);

      // the last authorization was longer than five years ago
      // the account hasn't been authorized
      // within the last YEARS_AUTHORIZE_WINDOW
      if (state.timestamp < five_years_ago) {
        return true // expired
      };
      false
    }
    // #[view]
    // /// Checks if there is a DonorAuthorized state, and if the timestamp
    // /// is within the YEARS_AUTHORIZE_WINDOW.
    // public fun is_within_authorize_window(dv_account: address): bool acquires DonorAuthorized {
    //   let now = timestamp::now_seconds();
    //   let five_years_secs = YEARS_AUTHORIZE_WINDOW * SECONDS_IN_YEAR;
    //   let start_authorize_window = 0;

    //   if (now > five_years_secs) {
    //     start_authorize_window = now - five_years_secs
    //   };
    //   let state = borrow_global_mut<DonorAuthorized>(dv_account);

    //   // the account hasn't been authorized
    //   // within the last YEARS_AUTHORIZE_WINDOW
    //   if (state.timestamp > start_authorize_window) {
    //     return false
    //   };
    //   true
    // }

    #[view]
    /// Checks if there is a DonorAuthorized state, and if the timestamp
    // TODO: this should be checked against payment activity
    // not simply transactions on account
    public fun has_activity_in_last_year(dv_account: address): bool {

      let latest_tx = activity::get_last_touch_usecs(dv_account);
      let now = timestamp::now_seconds();

      let one_year_ago = 0;
      if (now > SECONDS_IN_YEAR) {
        one_year_ago = now - SECONDS_IN_YEAR
      };

      if (latest_tx >= one_year_ago) {
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

      !authorization_expired(dv_account) &&
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
