/// Periodically, Donor Voice accounts, (a.k.a Community Wallets) must
/// be re-authorized by their donors. See docs in donor_voice.move for more details.


module ol_framework::donor_voice_reauth {
    use std::signer;
    use std::timestamp;
    use diem_framework::account;
    use ol_framework::activity;

    friend ol_framework::donor_voice_txs;
    friend ol_framework::donor_voice_governance;

    /// CONSTANTS
    /// Seconds in one year
    const SECONDS_IN_YEAR: u64 = 31536000;


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

    #[view]
    /// Checks if there is a DonorAuthorized state, and if the timestamp
    /// is within the AUTHORIZE_WINDOW.
    public fun is_within_authorize_window(): bool {

      false
    }

    #[view]
    /// Checks if there is a DonorAuthorized state, and if the timestamp
    /// is within the AUTHORIZE_WINDOW.
    public fun has_activity_in_last_year(dv_account: address): bool {
      let latest_tx = activity::get_last_activity_usecs(dv_account);
      let today = timestamp::now_seconds();
      let last_year = today - SECONDS_IN_YEAR;
      if (latest_tx < last_year) {
        return true
      };
      false
    }

    #[view]
    /// Account authorized to be active
    public fun is_authorized(dv_account: address): bool {
      is_within_authorize_window() &&
      has_activity_in_last_year(dv_account)
    }
}
