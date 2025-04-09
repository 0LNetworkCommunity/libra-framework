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

    /// Get the quality score for a voucher based on social distance
    /// Score is a percentage, out of 100
    /// A higher score represents a closer social connection
    public fun calculate_voucher_quality(voucher: address, user: address): u64 {
        let opt = ancestry::get_degree(voucher, user);
        if (option::is_none(&opt)) {
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

    /// Calculate the total quality of vouches for a user given a list of valid vouchers
    /// This is a pure function that doesn't access storage directly
    public fun calculate_quality_from_list(user: address, vouchers: &vector<address>): u64 {
        let total_score = 0;

        let i = 0;
        while (i < vector::length(vouchers)) {
            let one_voucher = vector::borrow(vouchers, i);
            let score = calculate_voucher_quality(*one_voucher, user);
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
