/// Maintains the consensus config for the blockchain. The config is stored in a
/// Reconfiguration, and may be updated by root.
module diem_framework::consensus_config {
    use std::error;
    use std::vector;

    use diem_framework::reconfiguration;
    use diem_framework::system_addresses;

    friend diem_framework::genesis;

    struct ConsensusConfig has key {
        config: vector<u8>,
    }

    /// The provided on chain config bytes are empty or invalid
    const EINVALID_CONFIG: u64 = 1;

    /// Publishes the ConsensusConfig config.
    public(friend) fun initialize(diem_framework: &signer, config: vector<u8>) {
        system_addresses::assert_diem_framework(diem_framework);
        assert!(vector::length(&config) > 0, error::invalid_argument(EINVALID_CONFIG));
        move_to(diem_framework, ConsensusConfig { config });
    }

    /// This can be called by on-chain governance to update on-chain consensus configs.
    public fun set(account: &signer, config: vector<u8>) acquires ConsensusConfig {
        system_addresses::assert_diem_framework(account);
        assert!(vector::length(&config) > 0, error::invalid_argument(EINVALID_CONFIG));

        let config_ref = &mut borrow_global_mut<ConsensusConfig>(@diem_framework).config;
        *config_ref = config;

        // Need to trigger reconfiguration so validator nodes can sync on the updated configs.
        reconfiguration::reconfigure();
    }
}
