spec ol_framework::sacred_cows {
    spec module {
        use diem_framework::chain_status;
        pragma verify = true;
        pragma aborts_if_is_strict;

        invariant [suspendable] chain_status::is_operating() ==> exists<SacredCow<SlowDrip>>(@0x2);
    }
}
