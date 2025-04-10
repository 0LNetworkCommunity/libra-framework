/*
  Vouch Metrics Module

  This module provides pure utility functions for calculating vouch quality metrics
  without directly accessing vouch data. This avoids circular dependencies between modules.

  The module only contains stateless helper functions that can be used by both
  vouch.move and founder.move without creating dependency cycles.
*/

module ol_framework::vouch_metrics {
    use std::option;
    use std::vector;
    use ol_framework::ancestry;

    use diem_std::debug::print;

    /// Get the score for the social distance between two accounts
    /// Score is a percentage, out of 100
    /// A higher score represents a closer social connections
    public fun social_distance(left: address, right: address): u64 {
        print(&20000);
        print(&left);
        print(&right);
        let opt = ancestry::get_degree(left, right);
        print(&opt);
        if (option::is_none(&opt)) {
            // please maintain social distance
            return 0
        };
        let degree = *option::borrow(&opt);
        if (degree == 0 ) {
            return 100
        };
        if (degree > 100) {
            return 0
        };

        100 / degree
    }

    /// Calculate the total social score for a user versus a list of accounts
    /// e.g. used for distance to a root of trust
    public fun calculate_total_social_score(user: address, list: &vector<address>): u64 {
        let total_score = 0;

        let i = 0;
        while (i < vector::length(list)) {
            let one_root = vector::borrow(list, i);
            let score = social_distance(*one_root, user);
            total_score = total_score + score;
            i = i + 1;
        };

        total_score
    }

    /// Filter a list of addresses to include only those unrelated by ancestry
    /// This is a pure helper function that doesn't access vouch data
    public fun filter_unrelated(addresses: vector<address>): vector<address> {
        ancestry::list_unrelated(addresses)
    }
}
