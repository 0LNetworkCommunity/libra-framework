/// Defines feature flags for ol_framework.
/// NOTE: there are additional feature flags at the std library level
/// Those should not be changed, and 0L policy level features should be kept
//separate.

/// 0L-specific feaures begin count at 100;

module ol_framework::feature_flags {
    use std::features;

    // Commit note: repurposing this feature flag as the inverse:
    // if we ever want there to be automatic epoch transitions.

    /// Whether the framework can sign an epoch transition (automatic epochs).
    const AUTO_EPOCH_TRIGGER: u64 = 100;
    public fun get_auto_epoch(): u64 { AUTO_EPOCH_TRIGGER }
    public fun auto_epoch_enabled(): bool {
        features::is_enabled(AUTO_EPOCH_TRIGGER)
    }

    /// does the dynamic vouching depend on Reputation
    const DYNAMIC_VOUCH_LIMITS: u64 = 101;
    public fun get_dynamic_vouch(): u64 { DYNAMIC_VOUCH_LIMITS }
    public fun dynamic_vouch_limits_enabled(): bool {
        features::is_enabled(DYNAMIC_VOUCH_LIMITS)
    }

    /// does the dynamic vouching depend on Reputation
    const RECURSIVE_REPUTATION: u64 = 102;
    public fun get_recursive_reputation(): u64 { RECURSIVE_REPUTATION }
    public fun recursive_reputation_enabled(): bool {
        features::is_enabled(RECURSIVE_REPUTATION)
    }
}
