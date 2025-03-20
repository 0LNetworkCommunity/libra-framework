/// OL relies on a social graph, and proactive "vouching" of human-operated
/// accounts as part of its sybil-resistance strategy.
/// This is intentional. In all crypto economic systems there is a blend of relationship
/// and economic guarantees. In OL we experiment in both domains, sometimes leaning
/// more explicitly on relationships (as other protocols may do this tacitly).
///
/// As such our relationship games would ideally use classic graph algorithms for community detection.
/// However the state available on the Move virtual machine is very limited. As such
/// we rely mostly on ancestry of lineages of account (ancestry.move), together with scoring
/// of the degree of relationships.
///
/// This approach however relies on always crawling the graph from a root of trust. This implies three
/// minimal goals for the software
/// 1. Scoring connections to a root of trust.
/// The software below, allows users to score a participant's connection to an arbitrary, user-defined
/// root of trust.
/// 2. Rotating Root of Trust
/// However not implemented in the protocol was a way to rotate this root of trust (up to V7).
/// This module now proposes a method of rotating root of trues
/// 3. Proposing a default root of trust.
/// This module sets a reasonable root of trust for a number of games
/// played in OL. It is not the only root of trust, and neither is it static.
/// Previously the Validator qualification game, had these functions built in using the (vouch.move).
/// For validators the root of trust was presumed to be the Genesis validator set. This source does not change that policy: it commences a default root of trust based on that genesis validator set, and with #2 above allows for the rotation of that root of trust.
/// There are a number of ways of instantiating and updating roots of trust,
/// this module is agnostic to the method of instantiation, besides providing
/// a transition from how it was implemented prior to V8.
/// It's expected that there will be experimentation in the rotating and
/// and selecting of roots of trust. For now this module is agnostic,
/// understanding the relationship building happens off-chain.


module ol_framework::root_of_trust {
    use std::vector;
    use std::signer;
    use diem_framework::system_addresses;
    use diem_framework::timestamp;

    /// Struct to store the root of trust configuration
    struct RootOfTrust has key {
        roots: vector<address>,
        last_updated_secs: u64,
        minimum_cohort: u64,
        rotate_window_days: u64,
    }

    /// Error codes
    const ENOT_INITIALIZED: u64 = 1;
    const ENOT_AUTHORIZED: u64 = 2;
    const EINVALID_ROOT: u64 = 3;
    const EINVALID_ROTATION: u64 = 4; // New error code for invalid rotation params
    const EROTATION_WINDOW_NOT_ELAPSED: u64 = 5;

    // Constants for time conversion
    const SECONDS_IN_DAY: u64 = 86400; // 24 * 60 * 60

    /// Anyone can initialize a root of trust on their account.
    /// as an initial implementation 0x1 framework address will also
    /// keep a default root of trust.
    fun maybe_initialize(user_sig: &signer, roots: vector<address>, minimum_cohort: u64, rotate_window_days: u64) {
        let user_addr = signer::address_of(user_sig);
        if (!exists<RootOfTrust>(user_addr)) {
            move_to(user_sig, RootOfTrust {
                roots,
                last_updated_secs: 0, // Initialize at 0
                minimum_cohort,
                rotate_window_days,
            });
        };
    }

    /// At the time of V8 upgrade, the framework
    /// will migrate the prior root of trust implementation
    /// to the new explicit one.
    public(friend) fun framework_migration(framework: &signer, roots: vector<address>, minimum_cohort: u64) {
        // Verify this is called by the framework account
        system_addresses::assert_diem_framework(framework);

        // Initialize the root of trust at the framework address
        maybe_initialize(framework, roots, minimum_cohort, 0);
    }

    /// Score a participant's connection to the root of trust
    fun score_connection(_participant: address): u64 {
        // TODO: Implementation
        0
    }

    #[view]
    /// Check if rotation is possible for a given registry
    public fun can_rotate(registry: address): bool acquires RootOfTrust {
        if (!exists<RootOfTrust>(registry)) {
            false
        } else {
            let root_of_trust = borrow_global<RootOfTrust>(registry);
            let elapsed_secs = timestamp::now_seconds() - root_of_trust.last_updated_secs;
            let required_secs = root_of_trust.rotate_window_days * SECONDS_IN_DAY;
            elapsed_secs >= required_secs
        }
    }

    /// Rotate the root of trust set by adding and removing addresses
    fun rotate_roots(user_sig: &signer, adds: vector<address>, removes: vector<address>) acquires RootOfTrust {
        let user_addr = signer::address_of(user_sig);
        assert!(exists<RootOfTrust>(user_addr), ENOT_INITIALIZED);
        assert!(can_rotate(user_addr), EROTATION_WINDOW_NOT_ELAPSED);

        // Check for conflicting addresses in adds and removes
        let i = 0;
        while (i < vector::length(&adds)) {
            let addr = *vector::borrow(&adds, i);
            assert!(!vector::contains(&removes, &addr), EINVALID_ROTATION);
            i = i + 1;
        };

        let root_of_trust = borrow_global_mut<RootOfTrust>(user_addr);

        // Process removals first
        i = 0;
        while (i < vector::length(&removes)) {
            let addr = *vector::borrow(&removes, i);
            let (found, index) = vector::index_of(&root_of_trust.roots, &addr);
            if (found) {
                vector::remove(&mut root_of_trust.roots, index);
            };
            i = i + 1;
        };

        // Process additions
        i = 0;
        while (i < vector::length(&adds)) {
            let addr = *vector::borrow(&adds, i);
            if (!vector::contains(&root_of_trust.roots, &addr)) {
                vector::push_back(&mut root_of_trust.roots, addr);
            };
            i = i + 1;
        };

        root_of_trust.last_updated_secs = timestamp::now_seconds();
    }

    /// Update the minimum cohort size required
    fun update_minimum_cohort(user_sig: &signer, new_minimum: u64) acquires RootOfTrust {
        let user_addr = signer::address_of(user_sig);
        assert!(exists<RootOfTrust>(user_addr), ENOT_INITIALIZED);

        let root_of_trust = borrow_global_mut<RootOfTrust>(user_addr);
        root_of_trust.minimum_cohort = new_minimum;
        root_of_trust.last_updated_secs = timestamp::now_seconds();
    }

    #[view]
    /// Get the current set of root addresses
    public fun get_current_roots_at_registry(registry: address): vector<address> acquires RootOfTrust {
       // return empty vector if the root of trust is not initialized
        if (!exists<RootOfTrust>(registry)) {
            vector::empty<address>()
        } else {
            let root_of_trust = borrow_global<RootOfTrust>(registry);
            root_of_trust.roots
        }
    }

    #[view]
    /// For a RootOfTrust published on `registry`
    /// check if a user is a member of the root of trust.
    public fun is_root_at_registry(registry: address, account: address): bool acquires RootOfTrust {
        if (exists<RootOfTrust>(registry)) {
            let list = get_current_roots_at_registry(registry);
            vector::contains(&list, &account)
        } else {
            false
        }
    }

    #[test_only]
    /// Function for testing initialization
    fun test_init(_user_sig: &signer) {
        // TODO: Add test initialization logic
    }
}
