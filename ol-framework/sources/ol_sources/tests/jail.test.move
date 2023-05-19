#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_jail {
    use std::vector;
    use ol_framework::mock;
    use ol_framework::jail;

    #[test(root = @ol_framework, bob = @0x1000b)]
    public entry fun unjail_by_vouch(root: signer, bob: signer) {

      let vals = mock::genesis_n_vals(5);
      let alice = *vector::borrow(&vals, 0);
      assert!(!jail::is_jailed(alice), 10001);

      jail::jail(&root, alice);
      assert!(jail::is_jailed(alice), 10002);

      jail::unjail_by_voucher(&bob, alice);
      assert!(!jail::is_jailed(alice), 10003);

    }
}