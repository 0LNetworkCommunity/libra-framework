
module ol_framework::vouch_migration {
  use std::vector;
  use diem_framework::validator_universe;
  use diem_framework::create_signer::create_signer;
  use ol_framework::vouch;

  // use diem_std::debug::print;

  #[test_only]
  friend ol_framework::test_vouch_migration;

  friend ol_framework::migrations;

  public(friend) fun migrate_my_vouches() {
    let all_vals = validator_universe::get_eligible_validators();
    vector::for_each(all_vals, |val| {
      // We create the signer for the val here since this is required
      // to add the GivenVouches resource.
      let val_signer = &create_signer(val); // <<< DANGER

      vouch::migrate_given_vouches(val_signer, all_vals);
    });

    vector::for_each(all_vals, |val| {
      // We create the signer for the val here since this is required
      // to add the ReceivedVouches resource and drop MyVouches resource.
      let val_signer = &create_signer(val); // <<< DANGER

      vouch::migrate_received_vouches(val_signer);
    });
  }
}
