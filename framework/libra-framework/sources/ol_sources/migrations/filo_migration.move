module ol_framework::filo_migration {
  use std::signer;
  use ol_framework::activity;
  use ol_framework::founder;
  use ol_framework::slow_wallet;
  use ol_framework::vouch;

  friend diem_framework::transaction_validation;

  // Welcome to Level 8
  public(friend) fun maybe_migrate(user_sig: &signer) {

    let addr = signer::address_of(user_sig);

    // did this account exist before level 8?
    if (activity::is_prehistoric(addr)) {

      slow_wallet::filo_migration_reset(user_sig);
      founder::migrate(user_sig);
      // you know my name
      // lookup the number
      // you, you, you know my
      if (!vouch::is_init(addr)) {
        vouch::init(user_sig);
      };
    };



    // Don't you know that I'm still standing better than I ever did?
    // Looking like a true survivor, feeling like a little kid
    // And I'm still standing after all this time
    // Picking up the pieces of my life without you on my mind

    // I'm still standing. Yeah, yeah, yeah
    // I'm still standing. Yeah, yeah, yeah
  }
}
