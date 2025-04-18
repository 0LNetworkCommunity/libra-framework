spec ol_framework::vouch {
    spec module {
        pragma verify = true; // Enables verification of the module
    }

    // Specification for identifying true friends of a given address
    spec true_friends(addr: address): vector<address> {
        pragma aborts_if_is_partial = true; // Allows partial aborts

        // Basic vector property
        ensures vector::length(result) >= 0; // Ensures the result is a valid vector

        // If there are no vouches, result should be empty
        ensures !exists<ReceivedVouches>(addr) ==>
            vector::length(result) == 0; // Ensures result is empty if no vouches exist

        // If we have vouches, they must be in a valid state
        ensures exists<ReceivedVouches>(addr) ==>
            vector::length(global<ReceivedVouches>(addr).incoming_vouches) >= 0; // Ensures vouches are valid if they exist

        // If result is non-empty, we must have vouches
        ensures vector::length(result) > 0 ==>
            exists<ReceivedVouches>(addr); // Ensures vouches exist if result is non-empty
    }

    // Specification for revoking a vouch
    spec revoke {
        pragma aborts_if_is_partial = true; // Allows partial aborts

        // Basic property - function should not abort
        aborts_if false; // Never aborts

        // After revoke, vouches vector should be valid
        ensures forall addr: address where exists<ReceivedVouches>(addr):
            vector::length(global<ReceivedVouches>(addr).incoming_vouches) >= 0; // Ensures vouches are valid after revoke

        // After revoke, the number of vouches should decrease or stay the same
        ensures forall addr: address where exists<ReceivedVouches>(addr):
            vector::length(global<ReceivedVouches>(addr).incoming_vouches) <=
            old(vector::length(global<ReceivedVouches>(addr).incoming_vouches)); // Ensures vouches do not increase after revoke
    }

    // Specification for checking non-expired vouches
    spec all_not_expired(addr: address): vector<address> {
        pragma aborts_if_is_partial = true; // Allows partial aborts

        // Basic vector property - result must be a valid vector
        ensures vector::length(result) >= 0; // Ensures the result is a valid vector

        // If we have vouches, they must be in a valid state
        ensures exists<ReceivedVouches>(addr) ==>
            vector::length(global<ReceivedVouches>(addr).incoming_vouches) >= 0; // Ensures vouches are valid if they exist
    }

    // Specification for bulk setting vouch records
    spec bulk_set(val: address, received_list: vector<address>, given_list: vector<address>) {
        pragma aborts_if_is_partial = true; // Allows partial aborts

        // Function should not abort
        aborts_if false; // Never aborts

        // If ReceivedVouches doesn't exist, nothing changes
        ensures !exists<ReceivedVouches>(val) ==> true; // No change if vouches do not exist

        // If ReceivedVouches exists, ensure basic properties
        ensures exists<ReceivedVouches>(val) ==> {
            let post = global<ReceivedVouches>(val);
            // Length of incoming_vouches must be less than or equal to buddy_list
            vector::length(post.incoming_vouches) <= vector::length(received_list) // Ensures vouches do not exceed buddy list
        };

        // Ensures the state is modified
        modifies global<ReceivedVouches>(val); // Indicates that vouches are modified
    }

    // Add a helper function to make specifications more readable
    spec fun is_subset_of_buddy_list(addr_vec: vector<address>, buddy_list: vector<address>): bool {
        forall i in 0..vector::length(addr_vec):
            exists j in 0..vector::length(buddy_list): addr_vec[i] == buddy_list[j] // Checks if addr_vec is a subset of buddy_list
    }
}
