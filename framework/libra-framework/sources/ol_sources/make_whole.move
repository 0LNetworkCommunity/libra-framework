
module ol_framework::make_whole {


  struct Incident has store {
    // name of the event that needs to be remedied
    incident_name: vector<u8>,
  }

  // cannot copy or drop, explicitly
  // must move from unclaimed to claimed
  struct Credit has store {
    // user addr
    user: address,
    // value
    value: u64,
  }

  struct MakeWhole<phantom Incident> has key {
    budget: LibraCoin,
    expiration_epoch: u64,
    unclaimed: vector<Credit>,
    claimed: vector<Credit>
  }

  /// creates a credit for incident T
  fun create_each_user_credit<T>(vm: &signer, incident_name: vector<u8>, ) {
    let state = borrow_global_mut<MakeWhole<T>>(@ol_framework);
    let c = Credit {
      user,
      calue
    };

    vector::push_back()

  }

  /// anyone can call the method after expiry and the coins will be burned.
  public fun lazy_burn_on_expire() {
    //
  }
}
