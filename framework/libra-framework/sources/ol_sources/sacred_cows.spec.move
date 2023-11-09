spec ol_framework::sacred_cows {
    spec module {
        use diem_framework::chain_status;
        pragma verify = true;
        // pragma aborts_if_is_strict;

        invariant [suspendable] chain_status::is_operating() ==>
        exists<SacredCow<SlowDrip>>(@0x2);

        invariant [suspendable] chain_status::is_operating() ==>
        borrow_global<SacredCow<SlowDrip>>(@0x2).value == 30000 * 1000000;
    }

    // only 0x2 can init
    spec init(zero_x_two_sig: &signer) {
        use std::signer;
        aborts_if signer::address_of(zero_x_two_sig) != @0x2;
        aborts_if exists<SacredCow<SlowDrip>>(@0x2);

    }
}
