// 0L 
// This is a dummy module to test if we can import and use Aptos fw fns

module ol_framework::util {
    use aptos_framework::chain_id;
    use aptos_std::ed25519;

    // 0L
    public entry fun use_fn_from_aptos_framework() {
        let _chain_id = chain_id::get();
    }

    // 0L
    public entry fun use_fn_from_aptos_std(
        account_public_key_bytes: vector<u8>
    ) {
        let _pubkey = ed25519::new_unvalidated_public_key_from_bytes(account_public_key_bytes);
    }

    /// Native function to deserialize a type T.
    ///
    /// Note that this function does not put any constraint on `T`. If code uses this function to
    /// deserialized a linear value, its their responsibility that the data they deserialize is
    /// owned.
    public(friend) native fun from_bytes<T>(bytes: vector<u8>): T;

    public fun address_from_bytes(bytes: vector<u8>): address {
        from_bytes(bytes)
    }
}