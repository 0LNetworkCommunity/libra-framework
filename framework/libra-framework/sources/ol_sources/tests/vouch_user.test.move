#[test_only]
module ol_framework::test_user_vouch {
  use std::signer;
  use diem_framework::account;
  use ol_framework::vouch;

  #[test(bob = @0x1000b)]
  /// New accounts should not have a vouch struct
  /// by default. It needs to be created
  fun vouch_user_init(bob: &signer) {
    let b_addr = signer::address_of(bob);
    account::create_account_for_test(b_addr);
    assert!(account::exists_at(b_addr), 735701);
    assert!(!vouch::is_init(b_addr), 735701);

  }

}
