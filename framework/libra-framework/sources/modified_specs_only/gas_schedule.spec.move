spec diem_framework::gas_schedule {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec initialize(diem_framework: &signer, gas_schedule_blob: vector<u8>) {
        use std::signer;

        include system_addresses::AbortsIfNotDiemFramework{ account: diem_framework };
        aborts_if len(gas_schedule_blob) == 0;
        aborts_if exists<GasScheduleV2>(signer::address_of(diem_framework));
        ensures exists<GasScheduleV2>(signer::address_of(diem_framework));
    }

    spec set_gas_schedule(diem_framework: &signer, gas_schedule_blob: vector<u8>) {
        use std::signer;
        use diem_framework::util;
        use diem_framework::stake;
        use diem_framework::coin::CoinInfo;
        use diem_framework::gas_coin::LibraCoin as GasCoin;
        use diem_framework::transaction_fee;
        // use diem_framework::staking_config;

        requires exists<stake::ValidatorFees>(@diem_framework);
        requires exists<CoinInfo<GasCoin>>(@diem_framework);
        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockDiemSupply;
        // include staking_config::StakingRewardsConfigRequirement;

        include system_addresses::AbortsIfNotDiemFramework{ account: diem_framework };
        aborts_if len(gas_schedule_blob) == 0;
        let new_gas_schedule = util::spec_from_bytes<GasScheduleV2>(gas_schedule_blob);
        let gas_schedule = global<GasScheduleV2>(@diem_framework);
        aborts_if exists<GasScheduleV2>(@diem_framework) && new_gas_schedule.feature_version < gas_schedule.feature_version;
        ensures exists<GasScheduleV2>(signer::address_of(diem_framework));
    }

    spec set_storage_gas_config(diem_framework: &signer, config: StorageGasConfig) {
        use diem_framework::stake;
        use diem_framework::coin::CoinInfo;
        use diem_framework::gas_coin::LibraCoin as GasCoin;
        use diem_framework::transaction_fee;
        // use diem_framework::staking_config;

        pragma timeout = 100;

        requires exists<stake::ValidatorFees>(@diem_framework);
        requires exists<CoinInfo<GasCoin>>(@diem_framework);
        include system_addresses::AbortsIfNotDiemFramework{ account: diem_framework };
        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockDiemSupply;
        // include staking_config::StakingRewardsConfigRequirement;
        aborts_if !exists<StorageGasConfig>(@diem_framework);
    }
}
