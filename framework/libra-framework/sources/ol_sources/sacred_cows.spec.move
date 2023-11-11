spec ol_framework::sacred_cows {
    spec module {
        use diem_framework::chain_status;
        pragma verify = true;
        pragma aborts_if_is_strict;

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

    // only 0x2 can init
    spec get_stored<T>(): u64 {

    }

    // only 0x2 can init
    spec assert_same(stored: u64, code: u64) {
      aborts_if stored != code;
    }

        // only 0x2 can init
    spec bless_this_cow<T>(zero_x_two_sig: &signer, value: u64) {
        use std::signer;
        use diem_framework::chain_status;
        aborts_if signer::address_of(zero_x_two_sig) != @0x2;
        aborts_if exists<SacredCow<T>>(@0x2);
        aborts_if !chain_status::is_genesis();
    }


    spec get_slow_drip_const(): u64 {
        aborts_if !exists<SacredCow<SlowDrip>>(@0x2);
        aborts_if borrow_global<SacredCow<SlowDrip>>(@0x2).value != SLOW_WALLET_EPOCH_DRIP;
    }
}
