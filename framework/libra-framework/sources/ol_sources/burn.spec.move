spec ol_framework::burn {
    use std::signer;
    use diem_framework::system_addresses;
    use ol_framework::libra_coin::LibraCoin;
    use ol_framework::coin;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
        
        // Global invariant: BurnCounter must be initialized before any operations
        invariant [suspendable] exists<BurnCounter>(@ol_framework);
    }

    spec BurnCounter {
        // Resource invariant
        invariant lifetime_burned <= MAX_U64;
        invariant lifetime_recycled <= MAX_U64;
    }

    spec initialize {
        let addr = signer::address_of(vm);
        
        // Preconditions
        requires system_addresses::is_ol_framework_address(addr);
        
        // Abort conditions
        aborts_if !system_addresses::is_ol_framework_address(addr);
        aborts_if exists<BurnCounter>(@ol_framework);
        
        // Postconditions
        ensures exists<BurnCounter>(@ol_framework);
        ensures global<BurnCounter>(@ol_framework).lifetime_burned == 0;
        ensures global<BurnCounter>(@ol_framework).lifetime_recycled == 0;
    }

    spec epoch_burn_fees {
        let addr = signer::address_of(vm);
        let initial_value = coin::value(coins);
        let pre_burned = global<BurnCounter>(@ol_framework).lifetime_burned;
        
        // Preconditions
        requires system_addresses::is_ol_framework_address(addr);
        requires exists<BurnCounter>(@ol_framework);
        requires pre_burned + initial_value <= MAX_U64;
        
        // Abort conditions
        aborts_if !system_addresses::is_ol_framework_address(addr);
        aborts_if !exists<BurnCounter>(@ol_framework);
        aborts_if pre_burned + initial_value > MAX_U64;
        
        // Postconditions
        ensures exists<BurnCounter>(@ol_framework);
        ensures result_1 ==> global<BurnCounter>(@ol_framework).lifetime_burned > pre_burned;
        ensures !result_1 ==> global<BurnCounter>(@ol_framework).lifetime_burned == pre_burned;
    }

    spec burn_and_track(coin: Coin<LibraCoin>) {
        let pre_burned = global<BurnCounter>(@ol_framework).lifetime_burned;
        let amount = coin::value(coin);
        
        // Preconditions
        requires exists<BurnCounter>(@ol_framework);
        requires pre_burned + amount <= MAX_U64;
        
        // Abort conditions
        aborts_if !exists<BurnCounter>(@ol_framework);
        aborts_if pre_burned + amount > MAX_U64;
        
        // Postconditions
        ensures global<BurnCounter>(@ol_framework).lifetime_burned == pre_burned + amount;
        ensures global<BurnCounter>(@ol_framework).lifetime_recycled == 
            old(global<BurnCounter>(@ol_framework).lifetime_recycled);
    }

    spec set_send_community(sender: &signer, community: bool) {
        let addr = signer::address_of(sender);
        
        // No aborts_if conditions needed since function handles both cases
        aborts_if false;
        
        // Postconditions
        ensures exists<UserBurnPreference>(addr);
        ensures global<UserBurnPreference>(addr).send_community == community;
    }
} 
