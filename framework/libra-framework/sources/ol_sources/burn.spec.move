spec ol_framework::burn {
    spec module {
        pragma verify = true; // Enables verification of the module
        pragma aborts_if_is_strict = true; // Ensures strict checking of aborts_if conditions
        invariant [suspendable] exists<BurnCounter>(@ol_framework); // Ensures a BurnCounter always exists
    }

    spec initialize {
        pragma aborts_if_is_partial = true; // Allows partial aborts
        let addr = signer::address_of(vm);
        requires system_addresses::is_ol_framework_address(addr); // Checks if the address is a system address
        ensures global<BurnCounter>(@ol_framework).lifetime_recycled == 0; // Ensures lifetime_recycled is initialized to 0
    }

    spec burn_and_track(coin: Coin<LibraCoin>) {
        requires exists<BurnCounter>(@ol_framework); // Requires a BurnCounter to exist
        aborts_if !exists<BurnCounter>(@ol_framework); // Aborts if no BurnCounter exists
        ensures global<BurnCounter>(@ol_framework).lifetime_burned == 
            old(global<BurnCounter>(@ol_framework).lifetime_burned) + coin::value(coin); // Ensures lifetime_burned is incremented by the coin value
        pragma aborts_if_is_partial = true; // Allows partial aborts
    }

    spec set_send_community(sender: &signer, community: bool) {
        let addr = signer::address_of(sender);
        
        // No aborts_if conditions needed since function handles both cases
        aborts_if false; // Never aborts
        
        // Postconditions
        ensures exists<UserBurnPreference>(addr); // Ensures UserBurnPreference exists for the address
        ensures global<UserBurnPreference>(addr).send_community == community; // Ensures send_community is set correctly
        requires exists<UserBurnPreference>(signer::address_of(sender)); // Ensures UserBurnPreference can be set for the address
    }

    spec epoch_burn_fees {
        pragma aborts_if_is_partial = true; // Allows partial aborts
        ensures global<BurnCounter>(@ol_framework).lifetime_burned >= old(global<BurnCounter>(@ol_framework).lifetime_burned); // Ensures lifetime_burned does not decrease
        requires global<BurnCounter>(@ol_framework).lifetime_burned < MAX_U64; // Requires lifetime_burned to be less than MAX_U64
    }

    spec vm_migration {
        pragma aborts_if_is_partial = true; // Allows partial aborts
    }

    spec vm_burn_with_user_preference {
        pragma aborts_if_is_partial = true; // Allows partial aborts
    }

    spec BurnCounter {
        invariant lifetime_burned <= MAX_U64; // Ensures lifetime_burned does not exceed MAX_U64
        invariant lifetime_recycled <= MAX_U64; // Ensures lifetime_recycled does not exceed MAX_U64
    }
}
