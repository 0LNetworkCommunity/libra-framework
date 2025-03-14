module ol_framework::filo_migration {
  use std::signer;
  use ol_framework::activity;
  use ol_framework::slow_wallet;

  friend diem_framework::transaction_validation;

  // Welcome to Level 8
  public(friend) fun maybe_migrate(user_sig: &signer) {

    let addr = signer::address_of(user_sig);
    if (activity::is_founder(addr)) {
      slow_wallet::filo_migration_reset(user_sig)
    }

    // Don't you know that I'm still standing better than I ever did?
    // Looking like a true survivor, feeling like a little kid
    // And I'm still standing after all this time
    // Picking up the pieces of my life without you on my mind

    // I'm still standing. Yeah, yeah, yeah
    // I'm still standing. Yeah, yeah, yeah
  }
}
