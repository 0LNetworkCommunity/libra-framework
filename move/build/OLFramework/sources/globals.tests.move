#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::global_tests {
    use aptos_framework::stake;
    use aptos_framework::genesis;
    use ol_framework::globals;
    use ol_framework::testnet;
    use std::vector;
    use std::signer;

    #[test(validator_1 = @0x123)]
    fun test_genesis_0L(validator_1: &signer) {
        genesis::setup();
        let (_sk_1, pk_1, pop_1) = stake::generate_identity();
        stake::initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, true);

        let current_set = stake::get_current_validators();
        let val_1_addr = signer::address_of(validator_1);
        assert!(vector::contains(&current_set, &val_1_addr), 98);
        // val_1 should send a proof transaction here before TowerState is invoked
        // todo v7
        // assert!(TowerState::test_helper_get_height(val_1_addr) == 0u64, 73570002);  
    }

    #[test(validator_1 = @0x123)]
    fun test_constants(validator_1: &signer) {
        genesis::setup();
        let (_sk_1, pk_1, pop_1) = stake::generate_identity();
        stake::initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, true);

        let current_set = stake::get_current_validators();
        let val_1_addr = signer::address_of(validator_1);
        assert!(vector::contains(&current_set, &val_1_addr), 98);

        let val_set_size = vector::length(&current_set);
        assert!(val_set_size == 1u64, 73570001);

        let epoch_len = globals::get_epoch_length();
        if (testnet::is_testnet()){
            assert!(epoch_len == 60u64, 73570001);
        } else {
            assert!(epoch_len == 196992u64, 73570001);
        }
    }
}