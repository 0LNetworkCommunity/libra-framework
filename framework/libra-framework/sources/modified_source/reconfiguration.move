/// Publishes configuration information for validators, and issues reconfiguration events
/// to synchronize configuration changes for the validators.
module diem_framework::reconfiguration {
    use std::error;
    // use std::features;
    use std::signer;

    use diem_framework::account;
    use diem_framework::event;
    use diem_framework::stake;
    use diem_framework::system_addresses;
    use diem_framework::timestamp;
    use diem_framework::chain_status;
    use diem_framework::storage_gas;
    use ol_framework::epoch_helper;

    // use diem_std::debug::print;

    friend diem_framework::diem_governance;
    friend diem_framework::block;
    friend diem_framework::consensus_config;
    friend diem_framework::execution_config;
    friend diem_framework::gas_schedule;
    friend diem_framework::genesis;
    friend diem_framework::version;

    /// Event that signals consensus to start a new epoch,
    /// with new configuration information. This is also called a
    /// "reconfiguration event"
    struct NewEpochEvent has drop, store {
        epoch: u64,
    }

    /// Holds information about state of reconfiguration
    struct Configuration has key {
        /// Epoch number
        epoch: u64,
        /// Time of last reconfiguration. Only changes on reconfiguration events.
        last_reconfiguration_time: u64,
        /// Event handle for reconfiguration events
        events: event::EventHandle<NewEpochEvent>,
    }

    /// Reconfiguration will be disabled if this resource is published under the
    /// diem_framework system address
    struct DisableReconfiguration has key {}

    /// The `Configuration` resource is in an invalid state
    const ECONFIGURATION: u64 = 1;
    /// A `Reconfiguration` resource is in an invalid state
    const ECONFIG: u64 = 2;
    /// A `ModifyConfigCapability` is in a different state than was expected
    const EMODIFY_CAPABILITY: u64 = 3;
    /// An invalid block time was encountered.
    const EINVALID_BLOCK_TIME: u64 = 4;
    /// An invalid block time was encountered.
    const EINVALID_GUID_FOR_EVENT: u64 = 5;


    //////// 0L ////////
    #[view]
    /// Returns the current epoch number
    public fun get_current_epoch(): u64 acquires Configuration {
        let config_ref = borrow_global<Configuration>(@diem_framework);
        config_ref.epoch
    }

    /// Only called during genesis.
    /// Publishes `Configuration` resource. Can only be invoked by diem framework account, and only a single time in Genesis.
    public(friend) fun initialize(diem_framework: &signer) {
        system_addresses::assert_diem_framework(diem_framework);

        // assert it matches `new_epoch_event_key()`, otherwise the event can't be recognized
        assert!(account::get_guid_next_creation_num(signer::address_of(diem_framework)) == 2, error::invalid_state(EINVALID_GUID_FOR_EVENT));
        move_to<Configuration>(
            diem_framework,
            Configuration {
                epoch: 0,
                last_reconfiguration_time: 0,
                events: account::new_event_handle<NewEpochEvent>(diem_framework),
            }
        );
    }

    /// Private function to temporarily halt reconfiguration.
    /// This function should only be used for offline WriteSet generation purpose and should never be invoked on chain.
    fun disable_reconfiguration(diem_framework: &signer) {
        system_addresses::assert_diem_framework(diem_framework);
        assert!(reconfiguration_enabled(), error::invalid_state(ECONFIGURATION));
        move_to(diem_framework, DisableReconfiguration {})
    }

    /// Private function to resume reconfiguration.
    /// This function should only be used for offline WriteSet generation purpose and should never be invoked on chain.
    fun enable_reconfiguration(diem_framework: &signer) acquires DisableReconfiguration {
        system_addresses::assert_diem_framework(diem_framework);

        assert!(!reconfiguration_enabled(), error::invalid_state(ECONFIGURATION));
        DisableReconfiguration {} = move_from<DisableReconfiguration>(signer::address_of(diem_framework));
    }

    fun reconfiguration_enabled(): bool {
        !exists<DisableReconfiguration>(@diem_framework)
    }

    /// Signal validators to start using new configuration. Must be called from friend config modules.
    public(friend) fun reconfigure() acquires Configuration {
        // Do not do anything if genesis has not finished.
        if (chain_status::is_genesis() || timestamp::now_microseconds() == 0) {
            return
        };

        let config_ref = borrow_global_mut<Configuration>(@diem_framework);
        let current_time = timestamp::now_microseconds();


        // Do not do anything if a reconfiguration event is already emitted within this transaction.
        //
        // This is OK because:
        // - The time changes in every non-empty block
        // - A block automatically ends after a transaction that emits a reconfiguration event, which is guaranteed by
        //   VM spec that all transactions coming after a reconfiguration transaction will be returned as Retry
        //   status.
        // - Each transaction must emit at most one reconfiguration event
        //
        // Thus, this check ensures that a transaction that does multiple "reconfiguration required" actions emits only
        // one reconfiguration event.
        //
        if (current_time == config_ref.last_reconfiguration_time) {
            return
        };

        // Reconfiguration "forces the block" to end, as mentioned above. Therefore, we must process the collected fees
        // explicitly so that staking can distribute them.
        // transaction_fee::process_collected_fees();

        // Call stake to compute the new validator set and distribute rewards and transaction fees.
        // stake::on_new_epoch();
        storage_gas::on_reconfig();

        assert!(current_time > config_ref.last_reconfiguration_time, error::invalid_state(EINVALID_BLOCK_TIME));
        config_ref.last_reconfiguration_time = current_time;
        spec {
            assume config_ref.epoch + 1 <= MAX_U64;
        };
        config_ref.epoch = config_ref.epoch + 1;

        epoch_helper::set_epoch(config_ref.epoch);

        event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }

    public fun last_reconfiguration_time(): u64 acquires Configuration {
        borrow_global<Configuration>(@diem_framework).last_reconfiguration_time
    }

    public fun current_epoch(): u64 acquires Configuration {
        borrow_global<Configuration>(@diem_framework).epoch
    }

    /// Emit a `NewEpochEvent` event. This function will be invoked by genesis directly to generate the very first
    /// reconfiguration event.
    fun emit_genesis_reconfiguration_event() acquires Configuration {
        let config_ref = borrow_global_mut<Configuration>(@diem_framework);
        assert!(config_ref.epoch == 0 && config_ref.last_reconfiguration_time == 0, error::invalid_state(ECONFIGURATION));
        config_ref.epoch = 1;

        event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }

    // For tests, skips the guid validation.
    #[test_only]
    public fun initialize_for_test(account: &signer) {
        system_addresses::assert_diem_framework(account);
        move_to<Configuration>(
            account,
            Configuration {
                epoch: 0,
                last_reconfiguration_time: 0,
                events: account::new_event_handle<NewEpochEvent>(account),
            }
        );
    }

    /// For rescue missions
    public fun reconfigure_for_rescue(vm: &signer) acquires Configuration {
        system_addresses::assert_ol(vm);

        let config_ref = borrow_global_mut<Configuration>(@diem_framework);

        storage_gas::on_reconfig();

        config_ref.epoch = config_ref.epoch + 1;

        epoch_helper::set_epoch(config_ref.epoch);

        event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }

    #[test_only]
    public fun reconfigure_for_test() acquires Configuration {
        reconfigure();
    }

    // This is used together with stake::end_epoch() for testing with last_reconfiguration_time
    // It must be called each time an epoch changes
    #[test_only]
    public fun test_helper_increment_epoch_dont_reconfigure(count: u64) acquires Configuration {
        let config_ref = borrow_global_mut<Configuration>(@diem_framework);
        let current_time = timestamp::now_microseconds();
        // if (current_time == config_ref.last_reconfiguration_time) {
        //     return
        // };
        config_ref.last_reconfiguration_time = current_time;
        config_ref.epoch = config_ref.epoch + count;

        epoch_helper::set_epoch(config_ref.epoch);

    }
}
