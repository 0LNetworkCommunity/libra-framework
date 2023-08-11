module ol_framework::testnet {
    ///////////////////////////////////////////////////////////////////////////
    // sets an env variable for test constants for devs and ci testing
    // File Prefix for errors: 2002
    ///////////////////////////////////////////////////////////////////////////
    use std::error;
    use std::signer;
    use std::chain_id;

    /// trying something that should only be done on testnet, out satan!
    const ENOT_TESTNET: u64 = 1;
    /// yo! only root should be trying this
    const EWHY_U_NO_ROOT: u64 = 2;

    public fun is_testnet(): bool {
        chain_id::get() == 4
    }

    public fun assert_testnet(vm: &signer): bool {
      assert!(
          signer::address_of(vm) == @ol_framework,
          error::permission_denied(EWHY_U_NO_ROOT)
      );
      assert!(is_testnet(), error::invalid_state(ENOT_TESTNET));
      true
    }

    public fun is_staging_net(): bool {
        chain_id::get() == 2
    }

    #[test_only]
    public fun unset(vm: &signer) {
      use aptos_framework::system_addresses;
      system_addresses::assert_ol(vm);
      chain_id::set_for_test(vm, 1);

    }
}
