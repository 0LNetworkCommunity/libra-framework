#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_jail {
    use std::vector;
    use ol_framework::mock;
    use ol_framework::jail;

    #[test(root = @ol_framework)]
    public entry fun jail_reputation(root: signer) {

      let vals = mock::genesis_n_vals(5);
      let alice = *vector::borrow(&vals, 0);
      let bob_the_buddy = *vector::borrow(&vals, 1);

      assert!(!jail::is_jailed(alice), 10001);
      // Alice fails to perform, is jailed the first time.
      jail::jail(&root, alice);
      assert!(jail::is_jailed(alice), 10002);

      let (lifetime, consecutive) = jail::get_jail_reputation(alice);
      assert!(lifetime == 1, 10003);
      assert!(consecutive == 1, 10004);

      // A voucher unjailed the account (not needed in test here)
      // and Alice fails to perform again.

      jail::jail(&root, alice);

      let (lifetime, consecutive) = jail::get_jail_reputation(alice);
      assert!(lifetime == 2, 10005);
      assert!(consecutive == 2, 10006);

      // A voucher unjailed the account (not needed in test here)
      // and Alice fails to perform again.
      jail::jail(&root, alice);

      let (lifetime, consecutive) = jail::get_jail_reputation(alice);
      assert!(lifetime == 3, 10007);
      assert!(consecutive == 3, 10008);

      let rep = jail::get_count_buddies_jailed(bob_the_buddy);

      assert!(rep == 3, 10009);
      // Now we say that Alice returned to the set successfully
      // so we can reset the consecutive fail. Other reputation 
      // marks on the account and on buddy Voucher accounts should fail.
      jail::reset_consecutive_fail(&root, alice);
      let (lifetime, consecutive) = jail::get_jail_reputation(alice);
      assert!(lifetime == 3, 10007);
      assert!(consecutive == 0, 10008);
      let rep = jail::get_count_buddies_jailed(bob_the_buddy);
      assert!(rep == 3, 10009);
    }

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