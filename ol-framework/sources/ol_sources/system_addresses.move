module ol_framework::ol_addresses {
    use std::error;
    use std::signer;

    /// The address/account did not correspond to the ol root address
    const ENOT_OL_ROOT_ADDRESS: u64 = 0;
    
    public fun is_ol_root_address(addr: address): bool {
        addr == @ol_root
    }

    public fun assert_ol_root_address(addr: address) {
        assert!(is_ol_root_address(addr),
        error::permission_denied(ENOT_OL_ROOT_ADDRESS))
    }

    public fun assert_ol_root(account: &signer) {
        assert_ol_root_address(signer::address_of(account))
    }
}