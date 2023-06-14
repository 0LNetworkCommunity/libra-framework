#[test_only]
module ol_framework::test_vdf{
    use ol_framework::vdf_fixtures;
    use ol_framework::vdf;

    #[test]
    fun verify_valid_proof() {
    // this tests the happy case, that a proof is submitted with all three correct parameters.
    let challenge = vdf_fixtures::easy_chal();
      // Generate solutions with:
      // cd ./verfiable_delay/vdf-cli && cargo run --release -- -l=512 aa 100 -tpietrzak
      // NOTE: the -l=512 is important because this is the security paramater of 0L miner.
    let proof = vdf_fixtures::easy_sol();
    let weso = false;
    assert!(
      vdf::verify(&challenge, &proof, &vdf_fixtures::easy_difficulty(), &vdf_fixtures::security(), weso), 7357001);
  }

}