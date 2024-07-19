/// The `multi_action_migration` module is designed to initialize the `Offer` structure
/// in accounts that have initiated Governance or are already multisig accounts.
///
/// Due to the necessity of forging the signer to add structures to multisig accounts,
/// this module is separated for security reasons and will be deprecated once the migration
/// is completed by the account owners.
///
/// Migration will be allowed for a window of ~30 epochs. After 30 epochs, the migration
/// will no longer function. Upon completion of the migration, a PR should be made to
/// remove the migration code, including the code that forges the signer.

module ol_framework::multi_action_migration {
    use std::signer;
    use std::error;
    use diem_framework::create_signer::create_signer;
    use diem_framework::multisig_account;
    use ol_framework::multi_action;
    use ol_framework::epoch_helper;

    #[test_only]
    friend ol_framework::test_multi_action;

    /// Account already has Offer structure
    const EOFFER_ALREADY_EXISTS: u64 = 1;
    /// Signer is not in the authorities list
    const ENOT_AUTHORIZED: u64 = 2;
    /// Governance is not initialized
    const EGOV_NOT_INITIALIZED: u64 = 3;
    /// Migration expired
    const EMIGRATION_EXPIRED: u64 = 4;

    /// Epoch to expire the migration
    const MIGRATION_DEPRECATION_EPOCH: u64 = 110;

    // DANGER - may forge the signer of the multisig account is necessary here
    // TODO: remove this function after offer migration is completed
    // Migrate a legacy account to have structure Offer in order to propose authorities changes
    public entry fun migrate_offer(sig: &signer, multisig_address: address) {
        // Ensure the account does not have Offer structure
        assert!(!multi_action::exists_offer(multisig_address), error::already_exists(EOFFER_ALREADY_EXISTS));
        assert!(epoch_helper::get_current_epoch() < MIGRATION_DEPRECATION_EPOCH, error::invalid_state(EMIGRATION_EXPIRED));

        // if account is multisig, forge signer and add Offer to the multisig account
        if (multisig_account::is_multisig(multisig_address)) {
            // a) multisig account: ensure the signer is in the authorities list
            assert!(multi_action::is_authority(multisig_address, signer::address_of(sig)), error::permission_denied(ENOT_AUTHORIZED));

            // We create the signer for the multisig account here since this is required
            // to add the Offer resource.
            // This should be safe because we check that the signer is in the authorities list.
            // Also, after all accounts are migrated this function will be deprecated.
            let multisig_signer = &create_signer(multisig_address); // <<< DANGER

            // create Offer structure
            multi_action::init_offer(multisig_signer, multisig_address);
        } else {
            // b) initiated account: ensure the account is initialized with governance and add Offer to the account
            assert!(multisig_address == signer::address_of(sig), error::permission_denied(ENOT_AUTHORIZED));
            assert!(multi_action::is_gov_init(multisig_address), error::invalid_state(EGOV_NOT_INITIALIZED));
            multi_action::init_offer(sig, multisig_address);
        };
    }
}
