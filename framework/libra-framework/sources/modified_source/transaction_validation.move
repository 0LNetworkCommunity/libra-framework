module diem_framework::transaction_validation {
    use std::error;
    // use std::features;
    use std::signer;
    use std::vector;

    use diem_framework::account;
    // use diem_framework::diem_coin::DiemCoin;
    use ol_framework::gas_coin::GasCoin;
    use diem_framework::chain_id;
    use diem_framework::coin;
    use diem_framework::system_addresses;
    use diem_framework::timestamp;
    use diem_framework::transaction_fee;

    // use diem_std::debug::print;

    friend diem_framework::genesis;

    /// This holds information that will be picked up by the VM to call the
    /// correct chain-specific prologue and epilogue functions
    struct TransactionValidation has key {
        module_addr: address,
        module_name: vector<u8>,
        script_prologue_name: vector<u8>,
        module_prologue_name: vector<u8>,
        multi_agent_prologue_name: vector<u8>,
        user_epilogue_name: vector<u8>,
    }

    /// MSB is used to indicate a gas payer tx
    const MAX_U64: u128 = 18446744073709551615;

    /// Transaction exceeded its allocated max gas
    const EOUT_OF_GAS: u64 = 6;

    /// Prologue errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system.
    const PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY: u64 = 1001;
    const PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD: u64 = 1002;
    const PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW: u64 = 1003;
    const PROLOGUE_EACCOUNT_DOES_NOT_EXIST: u64 = 1004;
    const PROLOGUE_ECANT_PAY_GAS_DEPOSIT: u64 = 1005;
    const PROLOGUE_ETRANSACTION_EXPIRED: u64 = 1006;
    const PROLOGUE_EBAD_CHAIN_ID: u64 = 1007;
    const PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG: u64 = 1008;
    const PROLOGUE_ESECONDARY_KEYS_ADDRESSES_COUNT_MISMATCH: u64 = 1009;
    const PROLOGUE_EFEE_PAYER_NOT_ENABLED: u64 = 1010;


    /// Only called during genesis to initialize system resources for this module.
    public(friend) fun initialize(
        diem_framework: &signer,
        script_prologue_name: vector<u8>,
        module_prologue_name: vector<u8>,
        multi_agent_prologue_name: vector<u8>,
        user_epilogue_name: vector<u8>,
    ) {
        system_addresses::assert_diem_framework(diem_framework);

        move_to(diem_framework, TransactionValidation {
            module_addr: @diem_framework,
            module_name: b"transaction_validation",
            script_prologue_name,
            module_prologue_name,
            multi_agent_prologue_name,
            user_epilogue_name,
        });
    }

    fun prologue_common(
        sender: signer,
        gas_payer: address,
        txn_sequence_number: u64,
        txn_authentication_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        assert!(
            timestamp::now_seconds() < txn_expiration_time,
            error::invalid_argument(PROLOGUE_ETRANSACTION_EXPIRED),
        );
        assert!(chain_id::get() == chain_id, error::invalid_argument(PROLOGUE_EBAD_CHAIN_ID));

        let transaction_sender = signer::address_of(&sender);
        assert!(account::exists_at(transaction_sender), error::invalid_argument(PROLOGUE_EACCOUNT_DOES_NOT_EXIST));
        assert!(
            txn_authentication_key == account::get_authentication_key(transaction_sender),
            error::invalid_argument(PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY),
        );

        assert!(
            txn_sequence_number < (1u64 << 63),
            error::out_of_range(PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG)
        );

        let account_sequence_number = account::get_sequence_number(transaction_sender);
        assert!(
            txn_sequence_number >= account_sequence_number,
            error::invalid_argument(PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD)
        );

        // [PCA12]: Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new by [PCA11].
        assert!(
            txn_sequence_number == account_sequence_number,
            error::invalid_argument(PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW)
        );

        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        assert!(
            coin::is_account_registered<GasCoin>(gas_payer),
            error::invalid_argument(PROLOGUE_ECANT_PAY_GAS_DEPOSIT),
        );
        let balance = coin::balance<GasCoin>(gas_payer);
        assert!(balance >= max_transaction_fee, error::invalid_argument(PROLOGUE_ECANT_PAY_GAS_DEPOSIT));
    }

    fun module_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        let gas_payer = signer::address_of(&sender);
        prologue_common(sender, gas_payer, txn_sequence_number, txn_public_key, txn_gas_price, txn_max_gas_units, txn_expiration_time, chain_id)
    }

    fun script_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        _script_hash: vector<u8>,
    ) {
        let gas_payer = signer::address_of(&sender);
        prologue_common(sender, gas_payer, txn_sequence_number, txn_public_key, txn_gas_price, txn_max_gas_units, txn_expiration_time, chain_id)
    }

    fun multi_agent_script_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_sender_public_key: vector<u8>,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        let sender_addr = signer::address_of(&sender);
        prologue_common(
            sender,
            sender_addr,
            txn_sequence_number,
            txn_sender_public_key,
            txn_gas_price,
            txn_max_gas_units,
            txn_expiration_time,
            chain_id,
        );
        multi_agent_common_prologue(secondary_signer_addresses, secondary_signer_public_key_hashes);
    }

    fun multi_agent_common_prologue(
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
    ) {
        let num_secondary_signers = vector::length(&secondary_signer_addresses);
        assert!(
            vector::length(&secondary_signer_public_key_hashes) == num_secondary_signers,
            error::invalid_argument(PROLOGUE_ESECONDARY_KEYS_ADDRESSES_COUNT_MISMATCH),
        );

        let i = 0;
        while ({
            spec {
                invariant i <= num_secondary_signers;
                invariant forall j in 0..i:
                    account::exists_at(secondary_signer_addresses[j])
                    && secondary_signer_public_key_hashes[j]
                       == account::get_authentication_key(secondary_signer_addresses[j]);
            };
            (i < num_secondary_signers)
        }) {
            let secondary_address = *vector::borrow(&secondary_signer_addresses, i);
            assert!(account::exists_at(secondary_address), error::invalid_argument(PROLOGUE_EACCOUNT_DOES_NOT_EXIST));

            let signer_public_key_hash = *vector::borrow(&secondary_signer_public_key_hashes, i);
            assert!(
                signer_public_key_hash == account::get_authentication_key(secondary_address),
                error::invalid_argument(PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY),
            );
            i = i + 1;
        }
    }

    fun fee_payer_script_prologue(
        sender: signer,
        txn_sequence_number: u64,
        txn_sender_public_key: vector<u8>,
        secondary_signer_addresses: vector<address>,
        secondary_signer_public_key_hashes: vector<vector<u8>>,
        fee_payer_address: address,
        fee_payer_public_key_hash: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        // assert!(features::fee_payer_enabled(), error::invalid_state(PROLOGUE_EFEE_PAYER_NOT_ENABLED));
        prologue_common(
            sender,
            fee_payer_address,
            txn_sequence_number,
            txn_sender_public_key,
            txn_gas_price,
            txn_max_gas_units,
            txn_expiration_time,
            chain_id,
        );
        multi_agent_common_prologue(secondary_signer_addresses, secondary_signer_public_key_hashes);
        assert!(
            fee_payer_public_key_hash == account::get_authentication_key(fee_payer_address),
            error::invalid_argument(PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY),
        );
    }

    /// Epilogue function is run after a transaction is successfully executed.
    /// Called by the Adapter
    fun epilogue(
        account: signer,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) {
        let addr = signer::address_of(&account);
        epilogue_gas_payer(account, addr, txn_sequence_number, txn_gas_price, txn_max_gas_units, gas_units_remaining);
    }

    /// Epilogue function with explicit gas payer specified, is run after a transaction is successfully executed.
    /// Called by the Adapter
    fun epilogue_gas_payer(
        account: signer,
        gas_payer: address,
        _txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) {
        assert!(txn_max_gas_units >= gas_units_remaining, error::invalid_argument(EOUT_OF_GAS));
        let gas_used = txn_max_gas_units - gas_units_remaining;

        assert!(
            (txn_gas_price as u128) * (gas_used as u128) <= MAX_U64,
            error::out_of_range(EOUT_OF_GAS)
        );
        let transaction_fee_amount = txn_gas_price * gas_used;
        // it's important to maintain the error code consistent with vm
        // to do failed transaction cleanup.
        assert!(
            coin::balance<GasCoin>(gas_payer) >= transaction_fee_amount,
            error::out_of_range(PROLOGUE_ECANT_PAY_GAS_DEPOSIT),
        );

        // if (features::collect_and_distribute_gas_fees()) {
        //     // If transaction fees are redistributed to validators, collect them here for
        //     // later redistribution.
            transaction_fee::collect_fee(gas_payer, transaction_fee_amount);
        // } else {
            // Otherwise, just burn the fee.
            // TODO: this branch should be removed completely when transaction fee collection
            // is tested and is fully proven to work well.
            // transaction_fee::burn_fee(gas_payer, transaction_fee_amount);
        // };

        // Increment sequence number
        let addr = signer::address_of(&account);
        account::increment_sequence_number(addr);
    }
}
