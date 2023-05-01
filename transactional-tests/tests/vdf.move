//# init --addresses Alice=0xf75daa73fc071f93593335eb9033da804777eb94491650dd3f095ce6f778acb6 Bob=0x9c3b634ac05d0af393e0f93b9b19b61e7cac1c519f566276aa0c6fd15dac12aa
//#      --private-keys Alice=56a26140eb233750cd14fb168c3eb4bd0782b099cde626ec8aff7f3cceb6364f Bob=952aaf3a98a27903dd078d76fc9e411740d2ae9dd9ecb87b96c7cd6b791ffc69
//#      --initial-coins 10000

//# run --script --signers Alice
script {
  use aptos_framework::vdf; // todo
  use aptos_framework::test_fixtures; // todo

  fun main() {
    // Scenario: Bob, an existing validator, is sending a transaction for Eve, 
    // with a challenge and proof not yet submitted to the chain.
    let challenge = test_fixtures::eve_0_easy_chal();
    let solution = test_fixtures::eve_0_easy_sol();
    // Parse key and check
    let (eve_addr, _auth_key) = vdf::extract_address_from_challenge(&challenge);
    assert!(eve_addr == @0x3DC18D1CF61FAAC6AC70E3A63F062E4B, 7357401001);
  }
}