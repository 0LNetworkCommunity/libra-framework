#[test_only]
module ol_framework::ancestry_tests {
    use ol_framework::ancestry;
    use std::vector;
    use std::signer;

    #[test(alice = @0x1, bob = @0x2)]
    public entry fun init(alice: signer, bob: signer) {
        ancestry::init(&alice, &bob);
        let tree = ancestry::get_tree(signer::address_of(&alice));
        assert!(vector::contains<address>(&tree, &signer::address_of(&bob)), 7357001);
    }

    #[test(ol_root = @ol_root, bob = @0x2)]
    public entry fun fam(ol_root: signer, bob: signer) {
        ancestry::init(&bob, &ol_root);
        let diem_addr = signer::address_of(&ol_root);
        let bob_addr = signer::address_of(&bob);
        // print(&diem_addr);
        // print(&bob_addr);
        
        let tree = ancestry::get_tree(bob_addr);
        // print(&tree);
        
        assert!(vector::contains<address>(&tree, &diem_addr), 7357001);
        let (is_family, _) = ancestry::is_family(diem_addr, bob_addr);
        // print(&is_family);
        // if (is_)
        assert!(is_family, 7357002);
    }    
}