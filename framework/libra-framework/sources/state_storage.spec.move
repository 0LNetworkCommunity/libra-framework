spec diem_framework::state_storage {
    spec module {
        use diem_std::chain_status;
        pragma verify = true;
        pragma aborts_if_is_strict;
        // After genesis, `StateStorageUsage` and `GasParameter` exist.
        invariant [suspendable] chain_status::is_operating() ==> exists<StateStorageUsage>(@diem_framework);
        invariant [suspendable] chain_status::is_operating() ==> exists<GasParameter>(@diem_framework);
    }

    /// ensure caller is admin.
    /// aborts if StateStorageUsage already exists.
    spec initialize(diem_framework: &signer) {
        use std::signer;
        let addr = signer::address_of(diem_framework);
        aborts_if !system_addresses::is_diem_framework_address(addr);
        aborts_if exists<StateStorageUsage>(@diem_framework);
    }

    spec on_new_block(epoch: u64) {
        use diem_std::chain_status;
        requires chain_status::is_operating();
        aborts_if false;
    }

    spec current_items_and_bytes(): (u64, u64) {
        aborts_if !exists<StateStorageUsage>(@diem_framework);
    }

    spec get_state_storage_usage_only_at_epoch_beginning(): Usage {
        // TODO: temporary mockup.
        pragma opaque;
    }

    spec on_reconfig {
        aborts_if true;
    }
}
