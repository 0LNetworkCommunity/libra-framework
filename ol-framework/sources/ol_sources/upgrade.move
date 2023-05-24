///////////////////////////////////////////////////////////////////////////
// Upgrade payload
// File Prefix for errors: 2100
///////////////////////////////////////////////////////////////////////////

module ol_framework::upgrade {
    use std::error;
    use std::signer;
    use std::vector;
    use ol_framework::epoch;
    // use DiemFramework::DiemConfig; // todo v7
    use aptos_framework::system_addresses;

    #[test_only] use aptos_framework::genesis;
    #[test_only] use aptos_framework::stake;
    #[test_only] use ol_framework::vector_helper;

    /// Structs for UpgradePayload resource
    struct UpgradePayload has key {
        payload: vector<u8>, 
    }

    /// Structs for UpgradeHistory resource
    struct UpgradeBlobs has store {
        upgraded_version: u64,
        upgraded_payload: vector<u8>,
        validators_signed: vector<address>,
        consensus_height: u64,
    }

    struct UpgradeHistory has key {
        records: vector<UpgradeBlobs>, 
    }

    // Function code: 01
    public fun initialize(account: &signer) {
        assert!(signer::address_of(account) == @ol_root, error::permission_denied(210001)); 
        move_to(account, UpgradePayload{payload: x""});
        move_to(account, UpgradeHistory{
            records: vector::empty<UpgradeBlobs>()},
        );
    }

    // Function code: 02
    public fun set_update(account: &signer, payload: vector<u8>) acquires UpgradePayload {
        assert!(signer::address_of(account) == @ol_root, error::permission_denied(210002)); 
        assert!(exists<UpgradePayload>(@ol_root), error::not_found(210002)); 
        let temp = borrow_global_mut<UpgradePayload>(@ol_root);
        temp.payload = payload;
    }

    // Can only be called by the VM
    // making public so that we can use in admin scripts of writeset-transaction-generator.
    public fun upgrade_reconfig(vm: &signer) acquires UpgradePayload {
        system_addresses::assert_vm(vm);
        reset_payload(vm);
        // This is janky, but there's no other way to get the current block height,
        // unless the prologue gives it to us.
        // The upgrade reconfigure happens on round 2, so we'll increment the
        // new start by 2 from previous.        
        let new_epoch_height = epoch::get_timer_height_start() + 2; 
        epoch::reset_timer(vm, new_epoch_height);

        // TODO: check if this has any impact.
        // Update global time by 1 to escape the timestamps check (for deduplication) 
        // of DiemConfig::reconfig_
        // that check prevents offline writsets from being written during emergency
        // offline recovery.
        // let timenow = DiemTimestamp::now_microseconds() + 100;
        // use any address except for 0x0 for updating.
        // DiemTimestamp::update_global_time(vm, @0x6, timenow);
        // DiemConfig::upgrade_reconfig(vm); // todo v7
    }    

    // Function code: 03
    public fun reset_payload(account: &signer) acquires UpgradePayload {
        assert!(signer::address_of(account) == @ol_root, error::permission_denied(210003)); 
        assert!(exists<UpgradePayload>(@ol_root), error::not_found(210003)); 
        let temp = borrow_global_mut<UpgradePayload>(@ol_root);
        temp.payload = vector::empty<u8>();
    }

    // Function code: 04
    public fun record_history(
        account: &signer, 
        upgraded_version: u64, 
        upgraded_payload: vector<u8>, 
        validators_signed: vector<address>,
        consensus_height: u64,
    ) acquires UpgradeHistory {
        assert!(signer::address_of(account) == @ol_root, error::permission_denied(210004)); 
        let new_record = UpgradeBlobs {
            upgraded_version: upgraded_version,
            upgraded_payload: upgraded_payload,
            validators_signed: validators_signed,
            consensus_height: consensus_height,
        };
        let history = borrow_global_mut<UpgradeHistory>(@ol_root);
        vector::push_back(&mut history.records, new_record);
    }

    // Function code: 05
    public fun retrieve_latest_history(): (u64, vector<u8>, vector<address>, u64) acquires UpgradeHistory {
        let history = borrow_global<UpgradeHistory>(@ol_root);
        let len = vector::length<UpgradeBlobs>(&history.records);
        if (len == 0) {
            return (0, vector::empty<u8>(), vector::empty<address>(), 0)
        };
        let entry = vector::borrow<UpgradeBlobs>(&history.records, len-1);
        (entry.upgraded_version, *&entry.upgraded_payload, *&entry.validators_signed, entry.consensus_height)
    }

    // Function code: 06
    public fun has_upgrade(): bool acquires UpgradePayload {
        assert!(exists<UpgradePayload>(@ol_root), error::permission_denied(210005)); 
        !vector::is_empty(&borrow_global<UpgradePayload>(@ol_root).payload)
    }

    // Function code: 07
    public fun get_payload(): vector<u8> acquires UpgradePayload {
        assert!(exists<UpgradePayload>(@ol_root), error::permission_denied(210006));
        *&borrow_global<UpgradePayload>(@ol_root).payload
    }

    //// ================ Tests ================

    // val: validator
    #[test(ol_root = @ol_root, val_1 = @0x11, val_2 = @0x22)]
    fun test_record_history(ol_root: &signer, val_1: &signer, val_2: &signer) 
    acquires UpgradeHistory {
        genesis::setup();
        let (_sk_1, pk_1, pop_1) = stake::generate_identity();
        let (_sk_2, pk_2, pop_2) = stake::generate_identity();
        stake::initialize_test_validator(&pk_1, &pop_1, val_1, 100, true, true);
        stake::initialize_test_validator(&pk_2, &pop_2, val_2, 100, true, true);
        initialize(ol_root);
        let val_1_addr = signer::address_of(val_1);
        let val_2_addr = signer::address_of(val_2);  

        let validators = vector::empty<address>();
        vector::push_back(&mut validators, val_1_addr);
        vector::push_back(&mut validators, val_2_addr);

        record_history(ol_root, 0, x"1234", *&validators, 200);
        
        let (upgraded_version, payload, voters, height) = retrieve_latest_history();
        assert!(upgraded_version == 0, 1);
        assert!(payload == x"1234", 1);
        assert!(vector_helper::compare(&voters, &validators), 1);
        assert!(height == 200, 1);
    }

    #[test(ol_root = @ol_root)]
    fun test_toggle_upgrade_flag(ol_root: &signer) 
    acquires UpgradePayload {
        initialize(ol_root);
        assert!(has_upgrade() == false, 1);
        set_update(ol_root, x"1234");
        assert!(has_upgrade() == true, 1);
        assert!(get_payload() == x"1234", 1);
    }
}