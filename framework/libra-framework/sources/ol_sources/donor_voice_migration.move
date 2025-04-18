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

module ol_framework::donor_voice_migration {
    use std::vector;
    use diem_framework::multisig_account;
    use diem_framework::system_addresses;
    use ol_framework::multi_action;
    use ol_framework::migration_capability::{Self, MigrationCapability};
    use ol_framework::donor_voice;
    use ol_framework::donor_voice_governance;

    friend ol_framework::migrations;

    #[test_only]
    friend ol_framework::test_multi_action;

    public(friend) fun v8_state_migration(framework: &signer, migration_cap: &MigrationCapability) {
        system_addresses::assert_diem_framework(framework);
        system_addresses::is_diem_framework_address(migration_capability::get_auth(migration_cap));

        // get registry of donor voice accounts
        let list = donor_voice::get_root_registry();

        vector::for_each_ref(&list, |multisig_address| {
            // if account is multisig, forge signer and add Offer to the multisig account
            each_migration(framework, migration_cap, *multisig_address);
        });
    }

    // DANGER - must forge the signer of the multisig account here
    // we use the capability pattern to isolate the ability to create
    // signers for migration purposes, and can only happen during the
    // epoch boundary transaction
    fun each_migration(framework: &signer, migration_cap: &MigrationCapability, multisig_address: address) {
        system_addresses::assert_diem_framework(framework);
        system_addresses::is_diem_framework_address(migration_capability::get_auth(migration_cap));
        // if account is multisig, forge signer and add Offer to the multisig account
        if (multisig_account::is_multisig(multisig_address) &&
        multi_action::is_multi_action(multisig_address) &&
        donor_voice::is_donor_voice(multisig_address)) {
            ///////////
            // DANGER
            // should only acquire capability in during the epoch boundary
            let multisig_signer = migration_capability::create_signer_with_cap(framework, migration_cap, multisig_address);

            //////////

            // create Offer structure
            multi_action::maybe_init_auth_offer(&multisig_signer, multisig_address);

            donor_voice_governance::maybe_init_dv_governance(&multisig_signer);

        }
    }
}
