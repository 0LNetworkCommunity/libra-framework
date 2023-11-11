spec ol_framework::slow_wallet {
    spec module {
        pragma verify = true;
        // pragma aborts_if_is_strict;
    }

    // setting a slow wallet should abort only if there is no SlowWalletList
    // present in the 0x1 address
    spec set_slow(sig: &signer) {
        aborts_if !exists<SlowWalletList>(@ol_framework);
    }

    // at epoch boundaries the slow wallet drip should never abort
    // if genesis is initialized properly
    spec on_new_epoch(vm: &signer): (bool, u64) {
        use ol_framework::sacred_cows::{SacredCow, SlowDrip};

        aborts_if !system_addresses::signer_is_ol_root(vm);

        aborts_if !exists<SacredCow<SlowDrip>>(@0x2);

        aborts_if borrow_global<SacredCow<SlowDrip>>(@0x2).value != 30000 * 1000000;
    }
}
