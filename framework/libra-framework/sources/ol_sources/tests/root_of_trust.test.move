#[test_only]
module ol_framework::root_of_trust_tests {
    // use std::vector;
    use diem_framework::system_addresses;
    use ol_framework::ancestry;
    // use ol_framework::root_of_trust;
    use ol_framework::ol_account;

    // Test addresses with descriptive names showing relationships
    const ALICE_AT_GENESIS: address = @0x123;
    const BOB_ALICES_CHILD: address = @0x456;
    const CAROL_BOBS_CHILD: address = @0x789;
    const DAVE_AT_GENESIS: address = @0x321;
    const EVE_DAVES_CHILD: address = @0x654;

    #[test_only]
    /// Sets up test environment with accounts and ancestry relationships:
    /// ALICE_AT_GENESIS -> BOB_ALICES_CHILD -> CAROL_BOBS_CHILD
    /// DAVE_AT_GENESIS -> EVE_DAVES_CHILD
    fun setup_test_ancestry(vm: &signer, framework: &signer) {
        system_addresses::assert_ol(vm);

        // Create accounts using ol_account
        ol_account::create_account(framework, ALICE_AT_GENESIS);
        ol_account::create_account(framework, BOB_ALICES_CHILD);
        ol_account::create_account(framework, CAROL_BOBS_CHILD);
        ol_account::create_account(framework, DAVE_AT_GENESIS);
        ol_account::create_account(framework, EVE_DAVES_CHILD);

        // Set up ancestry relationships - these are already handled by ol_account::create_account
        // which calls ancestry::adopt_this_child internally
    }

    #[test(vm = @vm_reserved, framework = @0x1)]
    fun test_basic_ancestry_setup(vm: signer, framework: signer) {
        setup_test_ancestry(&vm, &framework);

        // Verify direct relationships
        assert!(ancestry::is_in_tree(ALICE_AT_GENESIS, BOB_ALICES_CHILD), 1);
        assert!(ancestry::is_in_tree(BOB_ALICES_CHILD, CAROL_BOBS_CHILD), 2);
        assert!(ancestry::is_in_tree(DAVE_AT_GENESIS, EVE_DAVES_CHILD), 3);

        // Verify multi-generation relationship
        assert!(ancestry::is_in_tree(ALICE_AT_GENESIS, CAROL_BOBS_CHILD), 4);
    }
}
