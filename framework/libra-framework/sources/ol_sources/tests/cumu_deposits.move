
#[test_only]

module ol_framework::test_cumu_deposits {
    use ol_framework::cumulative_deposits;
    use ol_framework::receipts;
    use ol_framework::mock;

    #[test(root = @ol_framework, alice = @0x1000a)]
    fun cumu_deposits_init(root: &signer, alice: &signer) {
      mock::genesis_n_vals(root, 2);
      cumulative_deposits::vm_migrate_cumulative_deposits(root, alice, false);

      assert!(cumulative_deposits::is_init_cumu_tracking(@0x1000a), 7357001);
      let t = cumulative_deposits::get_cumulative_deposits(@0x1000a);
      assert!(t == 0, 7357002);

      // also check Carol's is properly initialized
      assert!(receipts::is_init(@0x1000b), 7357003);
    }

// DiemAccount::vm_migrate_cumulative_deposits(&dr, &sender, false);

// assert!(DiemAccount::is_init_cumu_tracking(@Alice), 7357001);
// let t = DiemAccount::get_cumulative_deposits(@Alice);
// assert!(t == 0, 7357002);

// // also check Carol's is properly initialized
// assert!(Receipts::is_init(@Carol), 7357003);
}

      // // mock more funds in Carol's account
      // DiemAccount::slow_wallet_epoch_drip(&dr, 100000);

      // let carols_donation = 1000;

      // let cap = DiemAccount::extract_withdraw_capability(&sender);
      // DiemAccount::pay_from<GAS>(&cap, @Alice, carols_donation, b"thanks", b"");
      // DiemAccount::restore_withdraw_capability(cap);

      // let (a, b, c) = Receipts::read_receipt(@Carol, @Alice);
      // assert!(a == 0, 7357004); // timestamp is 0
      // assert!(b == carols_donation, 7357005); // last payment
      // assert!(c == carols_donation, 7357006); // cumulative payments

      // let alice_cumu = DiemAccount::get_cumulative_deposits(@Alice);
      // assert!(alice_cumu == carols_donation, 7357007);