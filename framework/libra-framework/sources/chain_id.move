/// The chain id distinguishes between different chains (e.g., testnet and the main network).
/// One important role is to prevent transactions intended for one chain from being executed on another.
/// This code provides a container for storing a chain id and functions to initialize and get it.
module diem_framework::chain_id {
    use diem_framework::system_addresses;

    friend diem_framework::genesis;

    struct ChainId has key {
        id: u8
    }

    /// Only called during genesis.
    /// Publish the chain ID `id` of this instance under the SystemAddresses address
    public(friend) fun initialize(diem_framework: &signer, id: u8) {
        system_addresses::assert_diem_framework(diem_framework);
        move_to(diem_framework, ChainId { id })
    }

    #[view]
    /// Return the chain ID of this instance.
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(@diem_framework).id
    }

    #[test_only]
    public fun initialize_for_test(diem_framework: &signer, id: u8) {
        initialize(diem_framework, id);
    }
    #[test_only]
    public fun set_for_test(vm: &signer, id: u8) acquires ChainId {
        system_addresses::assert_ol(vm);
        let state = borrow_global_mut<ChainId>(@diem_framework);
        state.id = id;
    }

    #[test(diem_framework = @0x1)]
    fun test_get(diem_framework: &signer) acquires ChainId {
        initialize_for_test(diem_framework, 1u8);
        assert!(get() == 1u8, 1);
    }
}
