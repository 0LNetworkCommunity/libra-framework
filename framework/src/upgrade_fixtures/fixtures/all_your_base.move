// place this file in the libra-framework folder
// this way we test pre and post upgrade.
// pre upgrade this function will not be found.
// your automated tests should remove this post-test
module 0x1::all_your_base {
  #[view]
  public fun are_belong_to(): vector<u8> {
    b"us"
  }

  fun noop() {}
}
