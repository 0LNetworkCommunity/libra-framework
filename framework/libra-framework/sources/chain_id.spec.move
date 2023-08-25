spec diem_framework::chain_id {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec initialize {
        use std::signer;
        let addr = signer::address_of(diem_framework);
        aborts_if addr != @diem_framework;
        aborts_if exists<ChainId>(@diem_framework);
    }

    spec get {
        aborts_if !exists<ChainId>(@diem_framework);
    }
}
