spec ol_framework::vouch {
    use diem_framework::coin;
    use ol_framework::libra_coin::LibraCoin;
    use std::signer;
    use diem_framework::system_addresses;
    use ol_framework::epoch_helper;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = false;

        // Global invariant: ensure vectors are always aligned
        invariant forall addr: address where exists<ReceivedVouches>(addr):
            len(global<ReceivedVouches>(addr).incoming_vouches) == 
            len(global<ReceivedVouches>(addr).epoch_vouched);
    }



    spec true_friends(addr: address): vector<address> {
        pragma aborts_if_is_partial = true;
        aborts_if false;
        ensures forall a: address where vector::contains(result, a):
            exists i: u64 where i < len(global<ReceivedVouches>(addr).incoming_vouches):
                global<ReceivedVouches>(addr).incoming_vouches[i] == a;
    }

    spec is_valid_voucher_for(voucher: address, recipient: address): bool {
        aborts_if false;
        ensures result == vector::contains(true_friends(recipient), voucher);
    }

    spec revoke(grantor: &signer, friend_account: address) {
        let grantor_addr = signer::address_of(grantor);
        
        aborts_if grantor_addr == friend_account 
            with error::invalid_argument(ETRY_SELF_VOUCH_REALLY);
        aborts_if !exists<GivenVouches>(grantor_addr) 
            with error::invalid_state(EGRANTOR_NOT_INIT);
        
        let given = global<GivenVouches>(grantor_addr);
        aborts_if !vector::contains(given.outgoing_vouches, friend_account)
            with error::invalid_argument(EVOUCH_NOT_FOUND);

        ensures {
            let post given = global<GivenVouches>(grantor_addr);
            !vector::contains(given.outgoing_vouches, friend_account)
        };
    }

    spec all_not_expired(addr: address): vector<address> {
        let current_epoch = epoch_helper::get_current_epoch();
        
        aborts_if !exists<ReceivedVouches>(addr);
        
        ensures forall a: address where vector::contains(result, a): {
            let received = global<ReceivedVouches>(addr);
            let (found, i) = vector::index_of(received.incoming_vouches, a);
            found && received.epoch_vouched[i] + EXPIRATION_ELAPSED_EPOCHS > current_epoch
        };
    }

    spec schema VouchInvariant {
        invariant update forall addr: address 
            where exists<GivenVouches>(addr):
                len(global<GivenVouches>(addr).outgoing_vouches) <= BASE_MAX_VOUCHES;
    }

    spec bulk_set(addr: address, buddies: vector<address>) {
        pragma aborts_if_is_partial = true;
        
        ensures exists<ReceivedVouches>(addr);
        ensures {
            let received = global<ReceivedVouches>(addr);
            received.incoming_vouches == buddies &&
            len(received.epoch_vouched) == len(buddies)
        };
    }

    spec get_vouch_price(): u64 {
        ensures result == if (exists<VouchPrice>(@ol_framework)) {
            global<VouchPrice>(@ol_framework).amount
        } else {
            1_000
        };
    }

    spec true_friends_in_list(addr: address, list: &vector<address>): (vector<address>, u64) {
        ensures result_2 == len(result_1);
        ensures forall a: address where vector::contains(result_1, a):
            vector::contains(true_friends(addr), a) && vector::contains(list, a);
    }
}
