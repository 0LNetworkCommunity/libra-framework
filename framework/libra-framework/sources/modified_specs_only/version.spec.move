spec diem_framework::version {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec set_version(account: &signer, major: u64) {
        use std::signer;
        use diem_framework::chain_status;
        use diem_framework::timestamp;
        use diem_framework::stake;
        use diem_framework::coin::CoinInfo;
        use diem_framework::gas_coin::GasCoin;
        use diem_framework::transaction_fee;
        // use diem_framework::staking_config;
        // Not verified when verify_duration_estimate > vc_timeout
        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved).
        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockDiemSupply;
        // include staking_config::StakingRewardsConfigRequirement;
        requires chain_status::is_operating();
        requires timestamp::spec_now_microseconds() >= reconfiguration::last_reconfiguration_time();
        requires exists<stake::ValidatorFees>(@diem_framework);
        requires exists<CoinInfo<GasCoin>>(@diem_framework);

        aborts_if !exists<SetVersionCapability>(signer::address_of(account));
        aborts_if !exists<Version>(@diem_framework);

        let old_major = global<Version>(@diem_framework).major;
        aborts_if !(old_major < major);
    }

    /// Abort if resource already exists in `@diem_framwork` when initializing.
    spec initialize(diem_framework: &signer, initial_version: u64) {
        use std::signer;

        aborts_if signer::address_of(diem_framework) != @diem_framework;
        aborts_if exists<Version>(@diem_framework);
        aborts_if exists<SetVersionCapability>(@diem_framework);
    }

    /// This module turns on `aborts_if_is_strict`, so need to add spec for test function `initialize_for_test`.
    spec initialize_for_test {
        // Don't verify test functions.
        pragma verify = false;
    }
}
