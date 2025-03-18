#[test_only]
module ol_framework::test_user_vouch {
  use std::signer;
  use diem_framework::account;
  use ol_framework::ol_account;
  use ol_framework::mock;
  use ol_framework::vouch;


  #[test(bob = @0x1000b)]
  /// New accounts should not have a vouch struct
  fun vouch_user_init_without(bob: &signer) {
    let b_addr = signer::address_of(bob);
    account::create_account_for_test(b_addr);
    assert!(account::exists_at(b_addr), 735701);
    assert!(!vouch::is_init(b_addr), 735701);

  }

  #[test(framework = @0x1, bob = @0x1000b)]
  /// V7 user accounts should not have this struct
  fun v7_should_not_have(framework: &signer, bob: &signer) {
    mock::ol_test_genesis(framework);

    let b_addr = signer::address_of(bob);
    ol_account::test_create_create_v7_account(framework, b_addr);
    assert!(account::exists_at(b_addr), 735701);
    assert!(!vouch::is_init(b_addr), 735701);

  }
}
