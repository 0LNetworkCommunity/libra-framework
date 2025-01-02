spec ol_framework::burn {
    use std::signer;
    use diem_framework::system_addresses;
    use ol_framework::libra_coin::LibraCoin;
    use ol_framework::coin;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = false;
    }

    spec initialize(vm: &signer) {
        aborts_if !system_addresses::is_ol_framework_address(signer::address_of(vm));
    }
}
