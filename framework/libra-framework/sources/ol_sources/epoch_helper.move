module ol_framework::epoch_helper {
    friend diem_framework::reconfiguration;
    // too many cyclic dependencies from reconfiguration being the only place to reach epoch.
    struct EpochHelper has key {
      epoch: u64
    }

    public fun initialize(root: &signer) {
      if (!exists<EpochHelper>(@ol_framework)){
        move_to(root, EpochHelper {
          epoch: 0
        })
      };
    }

    public(friend) fun set_epoch(epoch: u64) acquires EpochHelper{
      let state = borrow_global_mut<EpochHelper>(@ol_framework);
      state.epoch = epoch;
    }

    #[view]
    public fun get_current_epoch():u64 acquires EpochHelper{
      let state = borrow_global<EpochHelper>(@ol_framework);
      state.epoch
    }
}