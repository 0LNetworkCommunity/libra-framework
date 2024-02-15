
// Vendor was silly and mixed the platform implementation's feature definitions
// together with the feature flag module.
// So we are moving it to the libra_framework:: module, outside of std::

module ol_framework::ol_features {
    use std::features;

    // Helper
    public fun change_feature_flags(framework: &signer, enable: vector<u64>,
    disable: vector<u64>) {
      features::change_feature_flags(framework, enable, disable);
    }

    // NOTE: OL features id begin numbering with 50

    /// Whether the new epoch trigger logic is enabled.
    /// Lifetime: transient
    const EPOCH_TRIGGER_ENABLED: u64 = 50;
    public fun get_epoch_trigger(): u64 { EPOCH_TRIGGER_ENABLED }
    public fun epoch_trigger_enabled(): bool {
        features::is_enabled(EPOCH_TRIGGER_ENABLED)
    }

}
