spec ol_framework::lockbox {

  spec maybe_initialize(user: &signer) {
      ensures exists<SlowWalletV2>(signer::address_of(user));
  }

  spec delay_impl(user: &signer, duration_from: u64, duration_to: u64, units: u64) {

    // cannot move to same duration
    ensures duration_from != duration_to;
    // cannot move to lower duration
    ensures duration_from < duration_to;

  }

  spec checked_transfer_impl(from: &signer, to: address, duration_type: u64, units: u64) {
    requires exists<SlowWalletV2>(signer::address_of(from));
    requires exists<SlowWalletV2>(to);

  }

}

