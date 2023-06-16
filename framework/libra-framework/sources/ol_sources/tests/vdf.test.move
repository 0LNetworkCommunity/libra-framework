#[test_only]
// NOTE: the vdf natives live in Zapatos: /Users/lucas/code/rust/zapatos/aptos-move/framework/src/natives/ol_native_vdf.rs
module ol_framework::test_vdf{
    use ol_framework::vdf_fixtures;
    use ol_framework::ol_native_vdf;

    // #[test]
    #[ignore]
    fun verify_valid_proof() {
    // this tests the happy case, that a proof is submitted with all three correct parameters.
    let challenge = vdf_fixtures::easy_chal();
      // Generate solutions with:
      // cd ./verfiable_delay/vdf-cli && cargo run --release -- -l=512 aa 100 -tpietrzak
      // NOTE: the -l=512 is important because this is the security paramater of 0L miner.
    let proof = vdf_fixtures::easy_sol();
    let _weso = false;
    assert!(
      ol_native_vdf::verify(&challenge, &proof, &vdf_fixtures::easy_difficulty(), &vdf_fixtures::security()), 7357001);
  }

}