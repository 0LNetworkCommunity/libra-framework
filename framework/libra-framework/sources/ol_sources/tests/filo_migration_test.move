#[test_only]
module ol_framework::test_filo_migration {
  use std::signer;
  use std::timestamp;
  use diem_framework::account;
  use ol_framework::activity;
  use ol_framework::filo_migration;
  use ol_framework::founder;
  use ol_framework::ol_account;
  use ol_framework::mock;
  use ol_framework::slow_wallet;
  use ol_framework::vouch;

  use diem_std::debug::print;

  /// two state initializations happen on first
  /// transaction
  fun simulate_transaction_validation(sender: &signer) {
    let time = timestamp::now_seconds();
    // will initialize structs if first time
    activity::increment(sender, time);

    // run migrations
    // Note, Activity and Founder struct should have been set above
    filo_migration::maybe_migrate(sender);
  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// V7 user accounts should not have this struct
  fun v7_should_not_have(framework: &signer, bob: &signer) {
    mock::ol_test_genesis(framework);

    let b_addr = signer::address_of(bob);
    ol_account::test_create_create_v7_account(framework, b_addr);
    assert!(account::exists_at(b_addr), 735701);
    assert!(!vouch::is_init(b_addr), 735701);

    simulate_transaction_validation(bob);

  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// V7 accounts get migrated to Founder on first tx
  fun v7_migrates_lazily_on_tx(framework: &signer, bob: &signer) {
    let b_addr = signer::address_of(bob);

    mock::ol_test_genesis(framework);
    // cannot be zero secs, since the activity.move needs > 0
    timestamp::fast_forward_seconds(10);

    ol_account::test_create_create_v7_account(framework, b_addr);
    assert!(account::exists_at(b_addr), 735701);
    assert!(!vouch::is_init(b_addr), 735701);

    simulate_transaction_validation(bob);
    // should not error if called again, lazy init
    simulate_transaction_validation(bob);

    // now the vouch state should exist
    assert!(vouch::is_init(b_addr), 735702);
    assert!(founder::is_founder(b_addr), 735703);
    assert!(activity::has_ever_been_touched(b_addr), 735704);
    assert!(slow_wallet::is_slow(b_addr), 735705);

    let (unlocked, total) = ol_account::balance(b_addr);
    print(&unlocked);
    print(&total);

  }
}
