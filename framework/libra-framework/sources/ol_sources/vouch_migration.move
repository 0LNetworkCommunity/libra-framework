
module ol_framework::vouch_migration {
  use std::vector;
  use diem_framework::validator_universe;
  use diem_framework::create_signer::create_signer;
  use ol_framework::vouch;


  fun migrate_given_vouches() {
    let all_vals = validator_universe::get_eligible_validators();
    vector::for_each(all_vals, |val| {
      // We create the signer for the val here since this is required
      // to add the GivenVouches resource.
      let val_signer = &create_signer(val); // <<< DANGER

      vouch::migrate_given_vouches(val_signer, all_vals);
      vouch::migrate_received_vouches(val_signer);
    });
  }
}
