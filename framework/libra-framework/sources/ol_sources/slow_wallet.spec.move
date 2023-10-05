spec ol_framework::slow_wallet {
    spec module {
        pragma verify = true;
        // pragma aborts_if_is_strict;
    }

    spec set_slow(sig: &signer) {
        // use diem_framework::
        aborts_if !exists<SlowWalletList>(@ol_framework);
    }
}
