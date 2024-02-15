
// Vendor was silly and mixed the platform implementation's feature definitions
// together with the feature flag module.
// So we are moving it to the libra_framework:: module, outside of std::

module ol_framework::ol_features {
    use std::features;

    // Helper
    public fun change_feature_flags(framework: &signer, enable: vector<u64>,
    disable: vector<u64>) {
      features::change_feature_flags(framework, enable, disable);
    }

    // --------------------------------------------------------------------------------------------
    // Code Publishing

    /// Whether validation of package dependencies is enabled, and the related native function is
    /// available. This is needed because of introduction of a new native function.
    /// Lifetime: transient
    const CODE_DEPENDENCY_CHECK: u64 = 1;
    #[view]
    public fun code_dependency_check_enabled(): bool {
        features::is_enabled(CODE_DEPENDENCY_CHECK)
    }

    /// Whether during upgrade compatibility checking, friend functions should be treated similar like
    /// private functions.
    /// Lifetime: permanent
    const TREAT_FRIEND_AS_PRIVATE: u64 = 2;
    #[view]
    public fun treat_friend_as_private(): bool {
        features::is_enabled(TREAT_FRIEND_AS_PRIVATE)
    }

    /// Whether the new SHA2-512, SHA3-512 and RIPEMD-160 hash function natives are enabled.
    /// This is needed because of the introduction of new native functions.
    /// Lifetime: transient
    const SHA_512_AND_RIPEMD_160_NATIVES: u64 = 3;

    public fun get_sha_512_and_ripemd_160_feature(): u64 { SHA_512_AND_RIPEMD_160_NATIVES }

    #[view]
    public fun sha_512_and_ripemd_160_enabled(): bool {
        features::is_enabled(SHA_512_AND_RIPEMD_160_NATIVES)
    }

    /// Whether the new `diem_stdlib::type_info::chain_id()` native for fetching the chain ID is enabled.
    /// This is needed because of the introduction of a new native function.
    /// Lifetime: transient
    const DIEM_STD_CHAIN_ID_NATIVES: u64 = 4;

    public fun get_diem_stdlib_chain_id_feature(): u64 { DIEM_STD_CHAIN_ID_NATIVES }

    #[view]
    public fun diem_stdlib_chain_id_enabled(): bool {
        features::is_enabled(DIEM_STD_CHAIN_ID_NATIVES)
    }

    /// Whether to allow the use of binary format version v6.
    /// Lifetime: transient
    const VM_BINARY_FORMAT_V6: u64 = 5;

    public fun get_vm_binary_format_v6(): u64 { VM_BINARY_FORMAT_V6 }

    #[view]
    public fun allow_vm_binary_format_v6(): bool {
        features::is_enabled(VM_BINARY_FORMAT_V6)
    }

    /// Whether gas fees are collected and distributed to the block proposers.
    /// Lifetime: transient
    const COLLECT_AND_DISTRIBUTE_GAS_FEES: u64 = 6;

    public fun get_collect_and_distribute_gas_fees_feature(): u64 { COLLECT_AND_DISTRIBUTE_GAS_FEES }

    #[view]
    public fun collect_and_distribute_gas_fees(): bool {
        features::is_enabled(COLLECT_AND_DISTRIBUTE_GAS_FEES)
    }

    /// Whether the new `diem_stdlib::multi_ed25519::public_key_validate_internal_v2()` native is enabled.
    /// This is needed because of the introduction of a new native function.
    /// Lifetime: transient
    const MULTI_ED25519_PK_VALIDATE_V2_NATIVES: u64 = 7;

    public fun multi_ed25519_pk_validate_v2_feature(): u64 { MULTI_ED25519_PK_VALIDATE_V2_NATIVES }

    #[view]
    public fun multi_ed25519_pk_validate_v2_enabled(): bool {
        features::is_enabled(MULTI_ED25519_PK_VALIDATE_V2_NATIVES)
    }

    /// Whether the new BLAKE2B-256 hash function native is enabled.
    /// This is needed because of the introduction of new native function(s).
    /// Lifetime: transient
    const BLAKE2B_256_NATIVE: u64 = 8;

    public fun get_blake2b_256_feature(): u64 { BLAKE2B_256_NATIVE }

    #[view]
    public fun blake2b_256_enabled(): bool {
        features::is_enabled(BLAKE2B_256_NATIVE)
    }

    /// Whether resource groups are enabled.
    /// This is needed because of new attributes for structs and a change in storage representation.
    const RESOURCE_GROUPS: u64 = 9;

    public fun get_resource_groups_feature(): u64 { RESOURCE_GROUPS }

    #[view]
    public fun resource_groups_enabled(): bool {
        features::is_enabled(RESOURCE_GROUPS)
    }

    /// Whether multisig accounts (different from accounts with multi-ed25519 auth keys) are enabled.
    const MULTISIG_ACCOUNTS: u64 = 10;

    public fun get_multisig_accounts_feature(): u64 { MULTISIG_ACCOUNTS }

    #[view]
    public fun multisig_accounts_enabled(): bool {
        features::is_enabled(MULTISIG_ACCOUNTS)
    }

    /// Whether delegation pools are enabled.
    /// Lifetime: transient
    const DELEGATION_POOLS: u64 = 11;

    public fun get_delegation_pools_feature(): u64 { DELEGATION_POOLS }

    #[view]
    public fun delegation_pools_enabled(): bool {
        features::is_enabled(DELEGATION_POOLS)
    }

    /// Whether generic algebra basic operation support in `crypto_algebra.move` are enabled.
    ///
    /// Lifetime: transient
    const CRYPTOGRAPHY_ALGEBRA_NATIVES: u64 = 12;
    public fun get_cryptography_algebra_natives_feature(): u64 {
    CRYPTOGRAPHY_ALGEBRA_NATIVES }
    #[view]
    public fun cryptography_algebra_enabled(): bool {
        features::is_enabled(CRYPTOGRAPHY_ALGEBRA_NATIVES)
    }

    /// Whether the generic algebra implementation for BLS12381 operations are enabled.
    ///
    /// Lifetime: transient
    const BLS12_381_STRUCTURES: u64 = 13;
    public fun get_bls12_381_strutures_feature(): u64 { BLS12_381_STRUCTURES }
    #[view]
    public fun bls12_381_structures_enabled(): bool {
        features::is_enabled(BLS12_381_STRUCTURES)
    }

    /// Whether native_public_key_validate aborts when a public key of the wrong length is given
    /// Lifetime: ephemeral
    const ED25519_PUBKEY_VALIDATE_RETURN_FALSE_WRONG_LENGTH: u64 = 14;

    /// Whether struct constructors are enabled
    ///
    /// Lifetime: transient
    const STRUCT_CONSTRUCTORS: u64 = 15;

    /// Whether reward rate decreases periodically.
    /// Lifetime: transient
    const PERIODICAL_REWARD_RATE_DECREASE: u64 = 16;
    public fun get_periodical_reward_rate_decrease_feature(): u64 {
    PERIODICAL_REWARD_RATE_DECREASE }
    #[view]
    public fun periodical_reward_rate_decrease_enabled(): bool {
        features::is_enabled(PERIODICAL_REWARD_RATE_DECREASE)
    }

    /// Whether enable paritial governance voting on diem_governance.
    /// Lifetime: transient
    const PARTIAL_GOVERNANCE_VOTING: u64 = 17;
    public fun get_partial_governance_voting(): u64 { PARTIAL_GOVERNANCE_VOTING
    }
    #[view]
    public fun partial_governance_voting_enabled(): bool {
        features::is_enabled(PARTIAL_GOVERNANCE_VOTING)
    }

    /// Charge invariant violation error.
    /// Lifetime: transient
    const CHARGE_INVARIANT_VIOLATION: u64 = 20;

    /// Whether enable paritial governance voting on delegation_pool.
    /// Lifetime: transient
    const DELEGATION_POOL_PARTIAL_GOVERNANCE_VOTING: u64 = 21;
    public fun get_delegation_pool_partial_governance_voting(): u64 {
    DELEGATION_POOL_PARTIAL_GOVERNANCE_VOTING }
    #[view]
    public fun delegation_pool_partial_governance_voting_enabled(): bool {
        features::is_enabled(DELEGATION_POOL_PARTIAL_GOVERNANCE_VOTING)
    }

    /// Whether alternate gas payer is supported
    /// Lifetime: transient
    const FEE_PAYER_ENABLED: u64 = 22;
    #[view]
    public fun fee_payer_enabled(): bool {
        features::is_enabled(FEE_PAYER_ENABLED)
    }

    /// Whether enable MOVE functions to call create_auid method to create AUIDs.
    /// Lifetime: transient
    const DIEM_UNIQUE_IDENTIFIERS: u64 = 23;
    public fun get_auids(): u64 { DIEM_UNIQUE_IDENTIFIERS }
    #[view]
    public fun auids_enabled(): bool {
        features::is_enabled(DIEM_UNIQUE_IDENTIFIERS)
    }

    /// Whether the new epoch trigger logic is enabled.
    /// Lifetime: transient
    const EPOCH_TRIGGER_ENABLED: u64 = 24;
    public fun get_epoch_trigger(): u64 { EPOCH_TRIGGER_ENABLED }
    public fun epoch_trigger_enabled(): bool {
        features::is_enabled(EPOCH_TRIGGER_ENABLED)
    }

}
