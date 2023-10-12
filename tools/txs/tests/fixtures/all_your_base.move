// NOTE: place this file in stdlib to generate the appropriate
// fixtures for testing framework upgrades
module std::all_your_base {
  #[view]
  public fun are_belong_to(): vector<u8> {
    b"us"
  }
}