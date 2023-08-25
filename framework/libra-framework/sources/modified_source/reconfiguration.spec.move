spec diem_framework::reconfiguration {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;

        // After genesis, `Configuration` exists.
        invariant [suspendable] chain_status::is_operating() ==> exists<Configuration>(@diem_framework);
        invariant [suspendable] chain_status::is_operating() ==>
            (timestamp::spec_now_microseconds() >= last_reconfiguration_time());
    }

    /// Make sure the signer address is @diem_framework.
    spec schema AbortsIfNotDiemFramework {
        diem_framework: &signer;

        let addr = signer::address_of(diem_framework);
        aborts_if !system_addresses::is_diem_framework_address(addr);
    }

    /// Address @diem_framework must exist resource Account and Configuration.
    /// Already exists in framework account.
    /// Guid_creation_num should be 2 according to logic.
    spec initialize(diem_framework: &signer) {
        use std::signer;
        use diem_framework::account::{Account};

        include AbortsIfNotDiemFramework;
        let addr = signer::address_of(diem_framework);
        requires exists<Account>(addr);
        aborts_if !(global<Account>(addr).guid_creation_num == 2);
        aborts_if exists<Configuration>(@diem_framework);
    }

    spec current_epoch(): u64 {
        aborts_if !exists<Configuration>(@diem_framework);
    }

    spec disable_reconfiguration(diem_framework: &signer) {
        include AbortsIfNotDiemFramework;
        aborts_if exists<DisableReconfiguration>(@diem_framework);
    }

    /// Make sure the caller is admin and check the resource DisableReconfiguration.
    spec enable_reconfiguration(diem_framework: &signer) {
        use diem_framework::reconfiguration::{DisableReconfiguration};
        include AbortsIfNotDiemFramework;
        aborts_if !exists<DisableReconfiguration>(@diem_framework);
    }

    /// When genesis_event emit the epoch and the `last_reconfiguration_time` .
    /// Should equal to 0
    spec emit_genesis_reconfiguration_event {
        use diem_framework::reconfiguration::{Configuration};

        aborts_if !exists<Configuration>(@diem_framework);
        let config_ref = global<Configuration>(@diem_framework);
        aborts_if !(config_ref.epoch == 0 && config_ref.last_reconfiguration_time == 0);
    }

    spec last_reconfiguration_time {
        aborts_if !exists<Configuration>(@diem_framework);
    }

    spec reconfigure {
        use diem_framework::coin::CoinInfo;
        use diem_framework::diem_coin::DiemCoin;
        use diem_framework::transaction_fee;
        // use diem_framework::staking_config;

        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved)

        requires exists<stake::ValidatorFees>(@diem_framework);
        requires exists<CoinInfo<DiemCoin>>(@diem_framework);

        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockDiemSupply;
        // include staking_config::StakingRewardsConfigRequirement;
        aborts_if false;
    }
}
