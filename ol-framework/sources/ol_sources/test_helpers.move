/////////////////////////////////////////////////////////////////////////
// 0L Module
// Test Helpers
// 
/////////////////////////////////////////////////////////////////////////

module ol_framework::test_helpers {
    #[test_only] use std::vector;

    #[test_only] use aptos_framework::genesis;
    #[test_only] use aptos_framework::stake;

    // todo v7
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
