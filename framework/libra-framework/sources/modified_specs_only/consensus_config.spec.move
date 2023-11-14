spec diem_framework::consensus_config {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    /// Ensure caller is admin.
    /// Aborts if StateStorageUsage already exists.
    spec initialize(diem_framework: &signer, config: vector<u8>) {
        use std::signer;
        let addr = signer::address_of(diem_framework);
        aborts_if !system_addresses::is_diem_framework_address(addr);
        aborts_if exists<ConsensusConfig>(@diem_framework);
        aborts_if !(len(config) > 0);
    }

    /// Ensure the caller is admin and `ConsensusConfig` should be existed.
    /// When setting now time must be later than last_reconfiguration_time.
    spec set(account: &signer, config: vector<u8>) {
        use diem_framework::chain_status;
        use diem_framework::timestamp;
        use std::signer;
        use diem_framework::stake;
        use diem_framework::coin::CoinInfo;
        use diem_framework::libra_coin::LibraCoin;
        use diem_framework::transaction_fee;
        // use diem_framework::staking_config;

        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved)

        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockDiemSupply;
        // include staking_config::StakingRewardsConfigRequirement;
        let addr = signer::address_of(account);
        aborts_if !system_addresses::is_diem_framework_address(addr);
        aborts_if !exists<ConsensusConfig>(@diem_framework);
        aborts_if !(len(config) > 0);

        requires chain_status::is_operating();
        requires timestamp::spec_now_microseconds() >= reconfiguration::last_reconfiguration_time();
        requires exists<stake::ValidatorFees>(@diem_framework);
        requires exists<CoinInfo<LibraCoin>>(@diem_framework);
    }
}
