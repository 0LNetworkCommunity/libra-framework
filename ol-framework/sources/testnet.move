///////////////////////////////////////////////////////////////////
// 0L Module
// 
// Error code: 
///////////////////////////////////////////////////////////////////

module ol_framework::testnet {
    ///////////////////////////////////////////////////////////////////////////
    // sets an env variable for test constants for devs and ci testing
    // File Prefix for errors: 2002
    ///////////////////////////////////////////////////////////////////////////
    use std::error;
    use std::signer;

    const ENOT_TESTNET: u64 = 666; // out satan!
    const EWHY_U_NO_ROOT: u64 = 667;

    struct IsTestnet has key { }

    public fun initialize(account: &signer) {
        assert!(
            signer::address_of(account) == @ol_root,
            error::permission_denied(200201)
        );
        move_to(account, IsTestnet{})
    }

    public fun is_testnet(): bool {
        exists<IsTestnet>(@ol_root)
    }

    public fun assert_testnet(vm: &signer): bool {
      assert!(
          signer::address_of(vm) == @ol_root,
          error::permission_denied(EWHY_U_NO_ROOT)
      );
      assert!(is_testnet(), error::invalid_state(ENOT_TESTNET));
      true
    }


    // only used for testing purposes
    public fun remove_testnet(account: &signer) acquires IsTestnet {
        assert!(
            signer::address_of(account) == @ol_root,
            error::permission_denied(EWHY_U_NO_ROOT)
        );
        IsTestnet{} = move_from<IsTestnet>(@ol_root);
    }
}

module ol_framework::staging_net {
    ///////////////////////////////////////////////////////////////////////////
    // sets an env variable for testing production settings except with 
    // shorter epochs and lower vdf difficulty.
    // File Prefix for errors: 1903
    ///////////////////////////////////////////////////////////////////////////
    use std::error;
    use std::signer;

    const EWHY_U_NO_ROOT: u64 = 667;
    struct IsStagingNet has key { }

    public fun initialize(account: &signer) {
        assert!(
            signer::address_of(account) == @ol_root,
            error::permission_denied(EWHY_U_NO_ROOT)
        );
        move_to(account, IsStagingNet{})
    }

    public fun is_staging_net(): bool {
        exists<IsStagingNet>(@ol_root)
    }

}