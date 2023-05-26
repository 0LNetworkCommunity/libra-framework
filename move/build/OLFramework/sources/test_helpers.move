/////////////////////////////////////////////////////////////////////////
// 0L Module
// Test Helpers
// 
/////////////////////////////////////////////////////////////////////////

#[test_only]
module ol_framework::test_helpers {
    use aptos_framework::genesis;
    use aptos_framework::stake;
    use std::vector;

    #[test_only]
    public fun init_validators(validators: &vector<signer>) {
        genesis::setup();
        vector::for_each_ref(validators, |val| { 
            let (_, pk, pop) = stake::generate_identity();
            stake::initialize_test_validator(
                &pk, &pop, val, 100, true, true
            );
        });
    }
}
