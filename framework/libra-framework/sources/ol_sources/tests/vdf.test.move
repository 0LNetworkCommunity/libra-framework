#[test_only]
// NOTE: the vdf natives live in Zapatos: /Users/user/code/rust/zapatos/aptos-move/framework/src/natives/ol_native_vdf.rs
module ol_framework::test_vdf{
    use ol_framework::vdf_fixtures;
    use ol_framework::ol_native_vdf;

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
      ol_native_vdf::verify(
        &challenge,
        &proof,
        vdf_fixtures::easy_difficulty(),
        vdf_fixtures::security(),
        weso
      ), 7357001);

    let challenge = vdf_fixtures::alice_0_hard_chal();
    // Generate solutions with:
    // cd ./verfiable_delay/vdf-cli && cargo run --release -- -l=512 aa 100 -tpietrzak
    // NOTE: the -l=512 is important because this is the security paramater of 0L miner.
    let proof = vdf_fixtures::alice_0_hard_sol();

    assert!(ol_native_vdf::verify(&challenge, &proof, vdf_fixtures::hard_difficulty(), vdf_fixtures::security(), false), 1);
  }

    #[test]
    fun reject_invalid_proof() {
    
    // sending an incorrect solution for this challenge
    let challenge = vdf_fixtures::easy_chal();

    // this is the wrong solution
    let proof = b"deadbeef";
    let weso = false;

    let is_good = ol_native_vdf::verify(
        &challenge,
        &proof,
        vdf_fixtures::easy_difficulty(),
        vdf_fixtures::security(),
        weso
      );

    assert!(!is_good, 7357001);



    // This time we'll send the incorrect difficulty
    let challenge = vdf_fixtures::easy_chal();
    // this is the CORRECT solution
    let proof = vdf_fixtures::easy_chal();
    let weso = false;

    let is_good = ol_native_vdf::verify(
        &challenge,
        &proof,
        666, // but this is the wrong difficulty
        vdf_fixtures::security(),
        weso
      );

    assert!(!is_good, 7357001);
  }


    #[test]
    #[expected_failure(abort_code = 10, location = 0x1::ol_native_vdf)]
    fun abort_large_security_param() {
    
    // sending an incorrect solution for this challenge
    // This time we'll send the incorrect difficulty
    let challenge = vdf_fixtures::easy_chal();
    // this is the CORRECT solution
    let proof = vdf_fixtures::easy_chal();
    let weso = false;

    let is_good = ol_native_vdf::verify(
        &challenge,
        &proof,
        vdf_fixtures::easy_difficulty(),
        4000, // this security param is way to large, the VM should reject and abort
        weso
      );

    assert!(!is_good, 7357001);



    // This time we'll send the incorrect difficulty
    let challenge = vdf_fixtures::easy_chal();
    // this is the CORRECT solution
    let proof = vdf_fixtures::easy_chal();
    let weso = false;

    let is_good = ol_native_vdf::verify(
        &challenge,
        &proof,
        1000000000, // difficulty too large for pietrzak
        vdf_fixtures::security(),
        weso
      );

    assert!(!is_good, 7357001);
  }


    #[test]
    #[expected_failure(abort_code = 10, location = 0x1::ol_native_vdf)]
    fun abort_large_difficulty_pietrezak() {

    // This time we'll send the incorrect difficulty
    let challenge = vdf_fixtures::easy_chal();
    // this is the CORRECT solution
    let proof = vdf_fixtures::easy_chal();
    let weso = false;

    let is_good = ol_native_vdf::verify(
        &challenge,
        &proof,
        1000000000, // difficulty too large for pietrzak
        vdf_fixtures::security(),
        weso
      );

    assert!(!is_good, 7357001);
  }


    #[test]
    #[expected_failure(abort_code = 10, location = 0x1::ol_native_vdf)]
    fun abort_large_difficulty_wesolowski() {

    // This time we'll send the incorrect difficulty
    let challenge = vdf_fixtures::easy_chal();
    // this is the CORRECT solution
    let proof = vdf_fixtures::easy_chal();
    let weso = true; // wesolowski

    let is_good = ol_native_vdf::verify(
        &challenge,
        &proof,
        4000000000, // difficulty too large for pietrzak
        vdf_fixtures::security(),
        weso
      );

    assert!(!is_good, 7357001);
  }
}