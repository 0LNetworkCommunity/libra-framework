module ol_framework::recovery_mode {
  use diem_framework::system_addresses;
  struct RecoveryMode has key {
    on: bool
  }

  /// during an offline upgrade, or incident, governance
  /// can set recovery mode where the validator selection stops
  public(friend) fun set_recovery_mode(framework: &signer, on: bool) acquires
  RecoveryMode {
    system_addresses::assert_diem_framework(framework);

    if (!exists<RecoveryMode>(@ol_framework)){
      move_to(framework, RecoveryMode { on })
    } else {
      let state = borrow_global_mut<RecoveryMode>(@ol_framework);
      state.on = on;
    }
  }

  #[view]
  public fun is_recovery_mode(): bool acquires RecoveryMode {
    borrow_global<RecoveryMode>(@ol_framework).on
  }

}
