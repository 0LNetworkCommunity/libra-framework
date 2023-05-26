#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::ancestry_tests {
    use aptos_framework::genesis;
    use aptos_framework::stake;
    use ol_framework::ancestry;
    use std::vector;
    use std::signer;

    #[test(validator = @0x1, bob = @0x2)]
    fun test_init(validator: &signer, bob: &signer) {
        genesis::setup();
        let (_sk_1, pk_1, pop_1) = stake::generate_identity();
        stake::initialize_test_validator(&pk_1, &pop_1, validator, 100, true, true);

        ancestry::init(validator, bob);
        let tree = ancestry::get_tree(signer::address_of(validator));
        assert!(vector::contains<address>(&tree, &signer::address_of(bob)), 7357001);
    }

    #[test(ol_root = @ol_root, bob = @0x2)]
    fun test_fam(ol_root: signer, bob: signer) {
        ancestry::init(&bob, &ol_root);
        let ol_root_addr = signer::address_of(&ol_root);
        let bob_addr = signer::address_of(&bob);
        // print(&ol_root_addr);
        // print(&bob_addr);        
        let tree = ancestry::get_tree(bob_addr);
        // print(&tree);        
        assert!(vector::contains<address>(&tree, &ol_root_addr), 7357001);
        let (is_family, _) = ancestry::is_family(ol_root_addr, bob_addr);
        // print(&is_family);
        // if (is_)
        assert!(is_family, 7357002);
    }

    #[test(vm = @vm_reserved, bob = @0x2, carol = @0x3)]
    fun test_migrate(vm: signer, bob: signer, carol: signer) {
        let carol_addr = signer::address_of(&carol);
        ancestry::fork_migrate(&vm, &bob, vector::singleton(carol_addr));
        let tree = ancestry::get_tree(signer::address_of(&bob));
        assert!(vector::contains<address>(&tree, &carol_addr), 7357001);
    }    
}