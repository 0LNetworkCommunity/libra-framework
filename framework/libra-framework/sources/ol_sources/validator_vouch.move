
module ol_framework::validator_vouch {
    use std::signer;
    use std::vector;
    use ol_framework::epoch_helper;
    use ol_framework::vouch;
    use diem_framework::stake;

    friend ol_framework::vouch_txs;

    /// How many epochs must pass before the voucher expires.
    const EXPIRATION_ELAPSED_EPOCHS: u64 = 45;

    /// Returns true if both addresses are validators
    fun both_are_validators(addr1: address, addr2: address): bool {
        stake::is_valid(addr1) && stake::is_valid(addr2)
    }

    /// Filter expired vouches for validator accounts only
    public fun filter_expired(account: address, list_addr: vector<address>, epoch_vouched: vector<u64>): vector<address> {
        let valid_vouches = vector::empty<address>();
        let current_epoch = epoch_helper::get_current_epoch();
        let i = 0;
        while (i < vector::length(&list_addr)) {
            let vouch_received = vector::borrow(&list_addr, i);
            let when_vouched = *vector::borrow(&epoch_vouched, i);
            if (both_are_validators(account, *vouch_received)) {
                // Only expire if both voucher and vouchee are validators
                if ((when_vouched + EXPIRATION_ELAPSED_EPOCHS) > current_epoch) {
                    vector::push_back(&mut valid_vouches, *vouch_received);
                }
            } else {
                // Otherwise, never expire vouches
                vector::push_back(&mut valid_vouches, *vouch_received);
            };
            i = i + 1;
        };
        valid_vouches
    }


    /// Garbage collect expired vouches for validator accounts using signer
    public(friend) fun garbage_collect_expired_for_signer(user_sig: &signer)  {
        let user = signer::address_of(user_sig);
        let (outgoing_vouches, outgoing_epochs) = vouch::get_given_vouches(user);
        let (incoming_vouches, incoming_epochs) = vouch::get_received_vouches(user);
        garbage_collect_expired(user, outgoing_vouches, outgoing_epochs, incoming_vouches, incoming_epochs);
    }

    /// Garbage collect expired vouches for validator accounts only
    fun garbage_collect_expired(user: address, outgoing_vouches: vector<address>, outgoing_epochs: vector<u64>, incoming_vouches: vector<address>, incoming_epochs: vector<u64>): (vector<address>, vector<u64>, vector<address>, vector<u64>) {
        let current_epoch = epoch_helper::get_current_epoch();
        let i = 0;
        while (i < vector::length(&outgoing_vouches)) {
            let vouch_received = vector::borrow(&outgoing_vouches, i);
            let when_vouched = *vector::borrow(&outgoing_epochs, i);
            if (both_are_validators(user, *vouch_received)) {
                if ((when_vouched + EXPIRATION_ELAPSED_EPOCHS) <= current_epoch) {
                    // Expired: remove vouch
                    vouch::remove_vouch(user, *vouch_received);
                }
            };
            // else: do nothing, non-validator vouches never expire
            i = i + 1;
        };

        let i = 0;
        while (i < vector::length(&incoming_vouches)) {
            let vouch_received = vector::borrow(&incoming_vouches, i);
            let when_vouched = *vector::borrow(&incoming_epochs, i);
            if (both_are_validators(user, *vouch_received)) {
                if ((when_vouched + EXPIRATION_ELAPSED_EPOCHS) <= current_epoch) {
                    // Expired: remove vouch
                    vouch::remove_vouch(*vouch_received, user);
                }
            };
            // else: do nothing, non-validator vouches never expire
            i = i + 1;
        };
        // No need to return filtered lists, as state is mutated directly
        (vector::empty<address>(), vector::empty<u64>(), vector::empty<address>(), vector::empty<u64>())
    }

    #[view]
    /// gets the received vouches not expired
    // TODO: duplicated with true_friends
    public fun get_received_vouches_not_expired(addr: address): vector<address> {
      let (all_received, epoch_vouched) = vouch::get_received_vouches(addr);
      filter_expired(addr, all_received, epoch_vouched)
    }
}
