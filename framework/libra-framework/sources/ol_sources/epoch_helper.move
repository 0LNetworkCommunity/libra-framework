module ol_framework::epoch_helper {
    use diem_framework::system_addresses;

    friend diem_framework::reconfiguration;
    friend ol_framework::proof_of_fee;

    // too many cyclic dependencies from reconfiguration being the only place to reach epoch.
    struct EpochHelper has key {
      epoch: u64
    }

    // some dependency cycling in vouch requires a separate data structure
    // TODO: refactor cycling.
    struct NominalRewardHelper has key {
      value: u64
    }

    public fun initialize(framework: &signer) {
      if (!exists<EpochHelper>(@ol_framework)){
        move_to(framework, EpochHelper {
          epoch: 0
        })
      };
    }

    //////// SETTERS ////////

    public(friend) fun set_epoch(epoch: u64) acquires EpochHelper{
      let state = borrow_global_mut<EpochHelper>(@ol_framework);
      state.epoch = epoch;
    }

    public(friend) fun set_nominal_reward(framework: &signer, value: u64) acquires NominalRewardHelper {
      system_addresses::assert_diem_framework(framework);
      // initialize on the fly for
      if (!exists<NominalRewardHelper>(@ol_framework)){
        move_to(framework, NominalRewardHelper {
          value
        })
      };
      let state = borrow_global_mut<NominalRewardHelper>(@ol_framework);
      state.value = value;
    }


    //////// GETTERS ////////
    #[view]
    public fun get_current_epoch():u64 acquires EpochHelper{
      let state = borrow_global<EpochHelper>(@ol_framework);
      state.epoch
    }

    #[view]
    public fun get_nominal_reward():u64 acquires NominalRewardHelper{
      let state = borrow_global<NominalRewardHelper>(@ol_framework);
      state.value
    }
}
