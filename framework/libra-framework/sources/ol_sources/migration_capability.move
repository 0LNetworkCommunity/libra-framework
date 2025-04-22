module ol_framework::migration_capability {
    use std::signer;
    use diem_framework::create_signer;
    use diem_framework::system_addresses;

    friend ol_framework::donor_voice_migration;
    friend ol_framework::epoch_boundary;

    // cannot be dropped
    // can only be created then destroyed during the epoch boundary
    struct MigrationCapability {
      auth: address,
    }

    public(friend) fun create_cap(framework: &signer): MigrationCapability {
      system_addresses::assert_diem_framework(framework);
      let cap = MigrationCapability { auth: signer::address_of(framework) };
      cap
    }

    public(friend) fun destroy_cap(framework: &signer, cap: MigrationCapability)  {
      system_addresses::assert_diem_framework(framework);

      let MigrationCapability { auth: _ } = cap;
    }

    public fun get_auth(cap: &MigrationCapability): address {
      cap.auth
    }

    /// DANGER, allows the framework to forge a signer
    /// for the purposes of migrating data structures
    /// uses the capability patter to isolate the ability to create
    /// signers for migration purposes, and can only happen during the
    /// epoch boundary transaction
    public(friend) fun create_signer_with_cap(framework: &signer, cap: &MigrationCapability, forge_this: address): signer {
      system_addresses::assert_diem_framework(framework);
      system_addresses::is_diem_framework_address(cap.auth);

      create_signer::create_signer(forge_this)
    }


}
