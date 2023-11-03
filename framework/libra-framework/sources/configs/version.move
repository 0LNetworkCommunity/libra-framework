/// Maintains the version number for the blockchain.
module diem_framework::version {
    use std::error;
    use std::signer;

    use diem_framework::reconfiguration;
    use diem_framework::system_addresses;

    friend diem_framework::genesis;

    struct Version has key {
        major: u64,
    }

    struct Git has key {
      hash: vector<u8>
    }

    struct SetVersionCapability has key {}

    /// Specified major version number must be greater than current version number.
    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 1;
    /// Account is not authorized to make this change.
    const ENOT_AUTHORIZED: u64 = 2;

    /// Only called during genesis.
    /// Publishes the Version config.
    public(friend) fun initialize(diem_framework: &signer, initial_version: u64) {
        system_addresses::assert_diem_framework(diem_framework);

        move_to(diem_framework, Version { major: initial_version});
        // Give diem framework account capability to call set version. This allows on chain governance to do it through
        // control of the diem framework account.
        move_to(diem_framework, SetVersionCapability {});
    }

    /// Updates the major version to a larger version.
    /// This can be called by on chain governance.
    public entry fun set_version(account: &signer, major: u64) acquires Version {
        assert!(exists<SetVersionCapability>(signer::address_of(account)), error::permission_denied(ENOT_AUTHORIZED));

        let old_major = borrow_global<Version>(@diem_framework).major;
        assert!(old_major < major, error::invalid_argument(EINVALID_MAJOR_VERSION_NUMBER));

        let config = borrow_global_mut<Version>(@diem_framework);
        config.major = major;

        // Need to trigger reconfiguration so validator nodes can sync on the updated version.
        reconfiguration::reconfigure();
    }

    /// set the git commit of a current upgrade.
    // NOTE: easier to troublshoot than the code.move manifests
    public fun upgrade_set_git(framework: &signer, hash: vector<u8>) acquires Git {
      if (!exists<Git>(@ol_framework)) {
        move_to(framework, Git {
          hash,
        })
      } else {
        let state = borrow_global_mut<Git>(@ol_framework);
        state.hash = hash;
      }
    }

    /// Only called in tests and testnets. This allows the core resources account, which only exists in tests/testnets,
    /// to update the version.
    fun initialize_for_test(core_resources: &signer) {
        system_addresses::assert_core_resource(core_resources);
        move_to(core_resources, SetVersionCapability {});
    }

    #[test(diem_framework = @diem_framework)]
    public entry fun test_set_version(diem_framework: signer) acquires Version {
        initialize(&diem_framework, 1);
        assert!(borrow_global<Version>(@diem_framework).major == 1, 0);
        set_version(&diem_framework, 2);
        assert!(borrow_global<Version>(@diem_framework).major == 2, 1);
    }

    #[test(diem_framework = @diem_framework, core_resources = @core_resources)]
    public entry fun test_set_version_core_resources(
        diem_framework: signer,
        core_resources: signer,
    ) acquires Version {
        initialize(&diem_framework, 1);
        assert!(borrow_global<Version>(@diem_framework).major == 1, 0);
        initialize_for_test(&core_resources);
        set_version(&core_resources, 2);
        assert!(borrow_global<Version>(@diem_framework).major == 2, 1);
    }

    #[test(diem_framework = @diem_framework, random_account = @0x123)]
    #[expected_failure(abort_code = 327682, location = Self)]
    public entry fun test_set_version_unauthorized_should_fail(
        diem_framework: signer,
        random_account: signer,
    ) acquires Version {
        initialize(&diem_framework, 1);
        set_version(&random_account, 2);
    }
}
