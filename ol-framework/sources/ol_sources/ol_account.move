
module ol_framework::ol_account {

  // TODO: v7: just stubs

  public fun is_slow(_addr: address): bool {
    true
  }

  public fun unlocked_amount(_addr: address): u64 {
    0
  }

  public fun vm_multi_pay_fee(_vm: &signer, _list: &vector<address>, _price: u64, _metadata: &vector<u8>) {

  }
}
