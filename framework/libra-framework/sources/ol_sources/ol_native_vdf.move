//////// 0L ////////
module ol_framework::ol_native_vdf {
    use std::vector;
    // verifies a VDF proof with security parameters.
    // For the 0th proof of a Delay Tower, this is used to check
    // the tower belongs to an authorization key and address.
    native public fun verify(
      challenge: &vector<u8>,
      solution: &vector<u8>,
      difficulty: u64,
      security: u64,
      wesolowski_algo: bool, // else it will be pietrezak (from ol V5)
    ): bool;

    native public fun extract_address_from_challenge(
      challenge: &vector<u8>
    ): (address, vector<u8>);

    #[test]
    fun test_native_vdf() {
      let r = verify(
        &vector::empty(),
        &vector::empty(),
        100,
        111,
        true
      );

      assert!(r == false, 100);
    }

}
