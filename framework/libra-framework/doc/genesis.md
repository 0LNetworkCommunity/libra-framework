
<a name="0x1_genesis"></a>

# Module `0x1::genesis`



-  [Struct `AccountMap`](#0x1_genesis_AccountMap)
-  [Struct `EmployeeAccountMap`](#0x1_genesis_EmployeeAccountMap)
-  [Struct `ValidatorConfiguration`](#0x1_genesis_ValidatorConfiguration)
-  [Struct `ValidatorConfigurationWithCommission`](#0x1_genesis_ValidatorConfigurationWithCommission)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_genesis_initialize)
-  [Function `initialize_aptos_coin`](#0x1_genesis_initialize_aptos_coin)
-  [Function `initialize_core_resources_and_aptos_coin`](#0x1_genesis_initialize_core_resources_and_aptos_coin)
-  [Function `create_accounts`](#0x1_genesis_create_accounts)
-  [Function `create_account`](#0x1_genesis_create_account)
-  [Function `create_initialize_validators_with_commission`](#0x1_genesis_create_initialize_validators_with_commission)
-  [Function `create_initialize_validators`](#0x1_genesis_create_initialize_validators)
-  [Function `create_initialize_validator`](#0x1_genesis_create_initialize_validator)
-  [Function `initialize_validator`](#0x1_genesis_initialize_validator)
-  [Function `set_genesis_end`](#0x1_genesis_set_genesis_end)
-  [Function `initialize_for_verification`](#0x1_genesis_initialize_for_verification)
-  [Specification](#@Specification_1)
    -  [Function `initialize_for_verification`](#@Specification_1_initialize_for_verification)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="aggregator_factory.md#0x1_aggregator_factory">0x1::aggregator_factory</a>;
<b>use</b> <a href="aptos_coin.md#0x1_aptos_coin">0x1::aptos_coin</a>;
<b>use</b> <a href="aptos_governance.md#0x1_aptos_governance">0x1::aptos_governance</a>;
<b>use</b> <a href="block.md#0x1_block">0x1::block</a>;
<b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="chain_status.md#0x1_chain_status">0x1::chain_status</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="consensus_config.md#0x1_consensus_config">0x1::consensus_config</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="execution_config.md#0x1_execution_config">0x1::execution_config</a>;
<b>use</b> <a href="">0x1::features</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="gas_schedule.md#0x1_gas_schedule">0x1::gas_schedule</a>;
<b>use</b> <a href="musical_chairs.md#0x1_musical_chairs">0x1::musical_chairs</a>;
<b>use</b> <a href="proof_of_fee.md#0x1_proof_of_fee">0x1::proof_of_fee</a>;
<b>use</b> <a href="reconfiguration.md#0x1_reconfiguration">0x1::reconfiguration</a>;
<b>use</b> <a href="slow_wallet.md#0x1_slow_wallet">0x1::slow_wallet</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="state_storage.md#0x1_state_storage">0x1::state_storage</a>;
<b>use</b> <a href="storage_gas.md#0x1_storage_gas">0x1::storage_gas</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="transaction_fee.md#0x1_transaction_fee">0x1::transaction_fee</a>;
<b>use</b> <a href="transaction_validation.md#0x1_transaction_validation">0x1::transaction_validation</a>;
<b>use</b> <a href="validator_universe.md#0x1_validator_universe">0x1::validator_universe</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="version.md#0x1_version">0x1::version</a>;
</code></pre>



<a name="0x1_genesis_AccountMap"></a>

## Struct `AccountMap`



<pre><code><b>struct</b> <a href="genesis.md#0x1_genesis_AccountMap">AccountMap</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>balance: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_genesis_EmployeeAccountMap"></a>

## Struct `EmployeeAccountMap`



<pre><code><b>struct</b> <a href="genesis.md#0x1_genesis_EmployeeAccountMap">EmployeeAccountMap</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>accounts: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>validator: <a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">genesis::ValidatorConfigurationWithCommission</a></code>
</dt>
<dd>

</dd>
<dt>
<code>vesting_schedule_numerator: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>vesting_schedule_denominator: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>beneficiary_resetter: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_genesis_ValidatorConfiguration"></a>

## Struct `ValidatorConfiguration`



<pre><code><b>struct</b> <a href="genesis.md#0x1_genesis_ValidatorConfiguration">ValidatorConfiguration</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owner_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>operator_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>voter_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>stake_amount: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>consensus_pubkey: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>proof_of_possession: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>network_addresses: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>full_node_network_addresses: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_genesis_ValidatorConfigurationWithCommission"></a>

## Struct `ValidatorConfigurationWithCommission`



<pre><code><b>struct</b> <a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">ValidatorConfigurationWithCommission</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>validator_config: <a href="genesis.md#0x1_genesis_ValidatorConfiguration">genesis::ValidatorConfiguration</a></code>
</dt>
<dd>

</dd>
<dt>
<code>commission_percentage: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>join_during_genesis: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_genesis_EACCOUNT_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="genesis.md#0x1_genesis_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>: u64 = 2;
</code></pre>



<a name="0x1_genesis_EDUPLICATE_ACCOUNT"></a>



<pre><code><b>const</b> <a href="genesis.md#0x1_genesis_EDUPLICATE_ACCOUNT">EDUPLICATE_ACCOUNT</a>: u64 = 1;
</code></pre>



<a name="0x1_genesis_initialize"></a>

## Function `initialize`

Genesis step 1: Initialize aptos framework account and core modules on chain.


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize">initialize</a>(<a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="">vector</a>&lt;u8&gt;, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, initial_version: u64, <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>: <a href="">vector</a>&lt;u8&gt;, <a href="execution_config.md#0x1_execution_config">execution_config</a>: <a href="">vector</a>&lt;u8&gt;, epoch_interval_microsecs: u64, _minimum_stake: u64, _maximum_stake: u64, _recurring_lockup_duration_secs: u64, _allow_validator_set_change: bool, _rewards_rate: u64, _rewards_rate_denominator: u64, _voting_power_increase_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize">initialize</a>(
    <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="">vector</a>&lt;u8&gt;,
    <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8,
    initial_version: u64,
    <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>: <a href="">vector</a>&lt;u8&gt;,
    <a href="execution_config.md#0x1_execution_config">execution_config</a>: <a href="">vector</a>&lt;u8&gt;,
    epoch_interval_microsecs: u64,
    _minimum_stake: u64,
    _maximum_stake: u64,
    _recurring_lockup_duration_secs: u64,
    _allow_validator_set_change: bool,
    _rewards_rate: u64,
    _rewards_rate_denominator: u64,
    _voting_power_increase_limit: u64,
) {
    // Initialize the aptos framework <a href="account.md#0x1_account">account</a>. This is the <a href="account.md#0x1_account">account</a> <b>where</b> system resources and modules will be
    // deployed <b>to</b>. This will be entirely managed by on-chain governance and no entities have the key or privileges
    // <b>to</b> <b>use</b> this <a href="account.md#0x1_account">account</a>.
    <b>let</b> (aptos_framework_account, aptos_framework_signer_cap) = <a href="account.md#0x1_account_create_framework_reserved_account">account::create_framework_reserved_account</a>(@aptos_framework);
    // Initialize <a href="account.md#0x1_account">account</a> configs on aptos framework <a href="account.md#0x1_account">account</a>.
    <a href="account.md#0x1_account_initialize">account::initialize</a>(&aptos_framework_account);

    <a href="transaction_validation.md#0x1_transaction_validation_initialize">transaction_validation::initialize</a>(
        &aptos_framework_account,
        b"script_prologue",
        b"module_prologue",
        b"multi_agent_script_prologue",
        b"epilogue",
    );

    // Give the decentralized on-chain governance control over the core framework <a href="account.md#0x1_account">account</a>.
    <a href="aptos_governance.md#0x1_aptos_governance_store_signer_cap">aptos_governance::store_signer_cap</a>(&aptos_framework_account, @aptos_framework, aptos_framework_signer_cap);

    // put reserved framework reserved accounts under aptos governance
    <b>let</b> framework_reserved_addresses = <a href="">vector</a>&lt;<b>address</b>&gt;[@0x2, @0x3, @0x4, @0x5, @0x6, @0x7, @0x8, @0x9, @0xa];
    <b>while</b> (!<a href="_is_empty">vector::is_empty</a>(&framework_reserved_addresses)) {
        <b>let</b> <b>address</b> = <a href="_pop_back">vector::pop_back</a>&lt;<b>address</b>&gt;(&<b>mut</b> framework_reserved_addresses);
        <b>let</b> (_, framework_signer_cap) = <a href="account.md#0x1_account_create_framework_reserved_account">account::create_framework_reserved_account</a>(<b>address</b>);
        <a href="aptos_governance.md#0x1_aptos_governance_store_signer_cap">aptos_governance::store_signer_cap</a>(&aptos_framework_account, <b>address</b>, framework_signer_cap);
    };

    <a href="consensus_config.md#0x1_consensus_config_initialize">consensus_config::initialize</a>(&aptos_framework_account, <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>);
    <a href="execution_config.md#0x1_execution_config_set">execution_config::set</a>(&aptos_framework_account, <a href="execution_config.md#0x1_execution_config">execution_config</a>);
    <a href="version.md#0x1_version_initialize">version::initialize</a>(&aptos_framework_account, initial_version);
    <a href="stake.md#0x1_stake_initialize">stake::initialize</a>(&aptos_framework_account);
    // staking_config::initialize(
    //     &aptos_framework_account,
    //     minimum_stake,
    //     maximum_stake,
    //     recurring_lockup_duration_secs,
    //     allow_validator_set_change,
    //     rewards_rate,
    //     rewards_rate_denominator,
    //     voting_power_increase_limit,
    // );
    <a href="storage_gas.md#0x1_storage_gas_initialize">storage_gas::initialize</a>(&aptos_framework_account);
    <a href="gas_schedule.md#0x1_gas_schedule_initialize">gas_schedule::initialize</a>(&aptos_framework_account, <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>);

    // Ensure we can create aggregators for supply, but not enable it for common <b>use</b> just yet.
    <a href="aggregator_factory.md#0x1_aggregator_factory_initialize_aggregator_factory">aggregator_factory::initialize_aggregator_factory</a>(&aptos_framework_account);
    <a href="coin.md#0x1_coin_initialize_supply_config">coin::initialize_supply_config</a>(&aptos_framework_account);

    <a href="chain_id.md#0x1_chain_id_initialize">chain_id::initialize</a>(&aptos_framework_account, <a href="chain_id.md#0x1_chain_id">chain_id</a>);
    <a href="reconfiguration.md#0x1_reconfiguration_initialize">reconfiguration::initialize</a>(&aptos_framework_account);
    <a href="block.md#0x1_block_initialize">block::initialize</a>(&aptos_framework_account, epoch_interval_microsecs);
    <a href="state_storage.md#0x1_state_storage_initialize">state_storage::initialize</a>(&aptos_framework_account);

    //////// 0L ////////

    <a href="validator_universe.md#0x1_validator_universe_initialize">validator_universe::initialize</a>(&aptos_framework_account);
    //todo: <a href="genesis.md#0x1_genesis">genesis</a> seats
    <b>let</b> genesis_seats = 10;
    <a href="musical_chairs.md#0x1_musical_chairs_initialize">musical_chairs::initialize</a>(&aptos_framework_account, genesis_seats);
    <a href="proof_of_fee.md#0x1_proof_of_fee_init_genesis_baseline_reward">proof_of_fee::init_genesis_baseline_reward</a>(&aptos_framework_account);
    <a href="slow_wallet.md#0x1_slow_wallet_initialize">slow_wallet::initialize</a>(&aptos_framework_account);
    // end 0L

    <a href="timestamp.md#0x1_timestamp_set_time_has_started">timestamp::set_time_has_started</a>(&aptos_framework_account);
}
</code></pre>



</details>

<a name="0x1_genesis_initialize_aptos_coin"></a>

## Function `initialize_aptos_coin`

Genesis step 2: Initialize Aptos coin.


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_aptos_coin">initialize_aptos_coin</a>(aptos_framework: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_aptos_coin">initialize_aptos_coin</a>(aptos_framework: &<a href="">signer</a>) {

    <b>let</b> (burn_cap, mint_cap) = <a href="aptos_coin.md#0x1_aptos_coin_initialize">aptos_coin::initialize</a>(aptos_framework);
    // Give <a href="stake.md#0x1_stake">stake</a> <b>module</b> MintCapability&lt;AptosCoin&gt; so it can mint <a href="rewards.md#0x1_rewards">rewards</a>.
    // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
    <a href="coin.md#0x1_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
    // Give <a href="transaction_fee.md#0x1_transaction_fee">transaction_fee</a> <b>module</b> BurnCapability&lt;AptosCoin&gt; so it can burn gas.
    // <a href="transaction_fee.md#0x1_transaction_fee_store_aptos_coin_burn_cap">transaction_fee::store_aptos_coin_burn_cap</a>(aptos_framework, burn_cap);
    <a href="coin.md#0x1_coin_destroy_burn_cap">coin::destroy_burn_cap</a>(burn_cap);

    // 0L: <a href="genesis.md#0x1_genesis">genesis</a> ceremony is calling this
    <b>let</b> (burn_cap, mint_cap) = <a href="gas_coin.md#0x1_gas_coin_initialize">gas_coin::initialize</a>(aptos_framework);
    // Give <a href="stake.md#0x1_stake">stake</a> <b>module</b> MintCapability&lt;AptosCoin&gt; so it can mint <a href="rewards.md#0x1_rewards">rewards</a>.
    // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
    <a href="coin.md#0x1_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
    <a href="coin.md#0x1_coin_destroy_burn_cap">coin::destroy_burn_cap</a>(burn_cap);
    <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">transaction_fee::initialize_fee_collection_and_distribution</a>(aptos_framework, 0);
}
</code></pre>



</details>

<a name="0x1_genesis_initialize_core_resources_and_aptos_coin"></a>

## Function `initialize_core_resources_and_aptos_coin`

Only called for testnets and e2e tests.


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_core_resources_and_aptos_coin">initialize_core_resources_and_aptos_coin</a>(aptos_framework: &<a href="">signer</a>, core_resources_auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_core_resources_and_aptos_coin">initialize_core_resources_and_aptos_coin</a>(
    aptos_framework: &<a href="">signer</a>,
    core_resources_auth_key: <a href="">vector</a>&lt;u8&gt;,
) {
    <b>let</b> (burn_cap, mint_cap) = <a href="aptos_coin.md#0x1_aptos_coin_initialize">aptos_coin::initialize</a>(aptos_framework);



    // Give <a href="stake.md#0x1_stake">stake</a> <b>module</b> MintCapability&lt;AptosCoin&gt; so it can mint <a href="rewards.md#0x1_rewards">rewards</a>.
    // stake::store_aptos_coin_mint_cap(aptos_framework, mint_cap);
    <a href="coin.md#0x1_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
    // Give <a href="transaction_fee.md#0x1_transaction_fee">transaction_fee</a> <b>module</b> BurnCapability&lt;AptosCoin&gt; so it can burn gas.
    // <a href="transaction_fee.md#0x1_transaction_fee_store_aptos_coin_burn_cap">transaction_fee::store_aptos_coin_burn_cap</a>(aptos_framework, burn_cap);
    <a href="coin.md#0x1_coin_destroy_burn_cap">coin::destroy_burn_cap</a>(burn_cap);

    <b>let</b> core_resources = <a href="account.md#0x1_account_create_account">account::create_account</a>(@core_resources);
    <a href="account.md#0x1_account_rotate_authentication_key_internal">account::rotate_authentication_key_internal</a>(&core_resources, core_resources_auth_key);
    <a href="aptos_coin.md#0x1_aptos_coin_configure_accounts_for_test">aptos_coin::configure_accounts_for_test</a>(aptos_framework, &core_resources, mint_cap);

    <b>let</b> (burn_cap, mint_cap) = <a href="gas_coin.md#0x1_gas_coin_initialize">gas_coin::initialize</a>(aptos_framework);
    <a href="coin.md#0x1_coin_destroy_mint_cap">coin::destroy_mint_cap</a>(mint_cap);
    <a href="coin.md#0x1_coin_destroy_burn_cap">coin::destroy_burn_cap</a>(burn_cap);

}
</code></pre>



</details>

<a name="0x1_genesis_create_accounts"></a>

## Function `create_accounts`



<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_accounts">create_accounts</a>(aptos_framework: &<a href="">signer</a>, accounts: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_AccountMap">genesis::AccountMap</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_accounts">create_accounts</a>(aptos_framework: &<a href="">signer</a>, accounts: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_AccountMap">AccountMap</a>&gt;) {
    <b>let</b> i = 0;
    <b>let</b> num_accounts = <a href="_length">vector::length</a>(&accounts);
    <b>let</b> unique_accounts = <a href="_empty">vector::empty</a>();

    <b>while</b> (i &lt; num_accounts) {
        <b>let</b> account_map = <a href="_borrow">vector::borrow</a>(&accounts, i);
        <b>assert</b>!(
            !<a href="_contains">vector::contains</a>(&unique_accounts, &account_map.account_address),
            <a href="_already_exists">error::already_exists</a>(<a href="genesis.md#0x1_genesis_EDUPLICATE_ACCOUNT">EDUPLICATE_ACCOUNT</a>),
        );
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> unique_accounts, account_map.account_address);

        <a href="genesis.md#0x1_genesis_create_account">create_account</a>(
            aptos_framework,
            account_map.account_address,
            account_map.balance,
        );

        i = i + 1;
    };
}
</code></pre>



</details>

<a name="0x1_genesis_create_account"></a>

## Function `create_account`

This creates an funds an account if it doesn't exist.
If it exists, it just returns the signer.


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_account">create_account</a>(aptos_framework: &<a href="">signer</a>, account_address: <b>address</b>, balance: u64): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_account">create_account</a>(aptos_framework: &<a href="">signer</a>, account_address: <b>address</b>, balance: u64): <a href="">signer</a> {
    <b>if</b> (<a href="account.md#0x1_account_exists_at">account::exists_at</a>(account_address)) {
        <a href="create_signer.md#0x1_create_signer">create_signer</a>(account_address)
    } <b>else</b> {
        <b>let</b> <a href="account.md#0x1_account">account</a> = <a href="account.md#0x1_account_create_account">account::create_account</a>(account_address);
        <a href="coin.md#0x1_coin_register">coin::register</a>&lt;AptosCoin&gt;(&<a href="account.md#0x1_account">account</a>);
        <a href="aptos_coin.md#0x1_aptos_coin_mint">aptos_coin::mint</a>(aptos_framework, account_address, balance);
        <a href="account.md#0x1_account">account</a>
    }
}
</code></pre>



</details>

<a name="0x1_genesis_create_initialize_validators_with_commission"></a>

## Function `create_initialize_validators_with_commission`



<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_initialize_validators_with_commission">create_initialize_validators_with_commission</a>(aptos_framework: &<a href="">signer</a>, use_staking_contract: bool, validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">genesis::ValidatorConfigurationWithCommission</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_initialize_validators_with_commission">create_initialize_validators_with_commission</a>(
    aptos_framework: &<a href="">signer</a>,
    use_staking_contract: bool,
    validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">ValidatorConfigurationWithCommission</a>&gt;,
) {
    <b>let</b> i = 0;
    <b>let</b> num_validators = <a href="_length">vector::length</a>(&validators);

    <b>while</b> (i &lt; num_validators) {

        <b>let</b> validator = <a href="_borrow">vector::borrow</a>(&validators, i);
        <a href="genesis.md#0x1_genesis_create_initialize_validator">create_initialize_validator</a>(aptos_framework, validator, use_staking_contract);

        i = i + 1;
    };

    // Destroy the aptos framework <a href="account.md#0x1_account">account</a>'s ability <b>to</b> mint coins now that we're done <b>with</b> setting up the initial
    // validators.
    // <a href="aptos_coin.md#0x1_aptos_coin_destroy_mint_cap">aptos_coin::destroy_mint_cap</a>(aptos_framework);

    <a href="stake.md#0x1_stake_on_new_epoch">stake::on_new_epoch</a>();

}
</code></pre>



</details>

<a name="0x1_genesis_create_initialize_validators"></a>

## Function `create_initialize_validators`

Sets up the initial validator set for the network.
The validator "owner" accounts, and their authentication
Addresses (and keys) are encoded in the <code>owners</code>
Each validator signs consensus messages with the private key corresponding to the Ed25519
public key in <code>consensus_pubkeys</code>.
Finally, each validator must specify the network address
(see types/src/network_address/mod.rs) for itself and its full nodes.

Network address fields are a vector per account, where each entry is a vector of addresses
encoded in a single BCS byte array.


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_initialize_validators">create_initialize_validators</a>(aptos_framework: &<a href="">signer</a>, validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfiguration">genesis::ValidatorConfiguration</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_initialize_validators">create_initialize_validators</a>(aptos_framework: &<a href="">signer</a>, validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfiguration">ValidatorConfiguration</a>&gt;) {
    <b>let</b> i = 0;
    <b>let</b> num_validators = <a href="_length">vector::length</a>(&validators);

    <b>let</b> validators_with_commission = <a href="_empty">vector::empty</a>();

    <b>while</b> (i &lt; num_validators) {
        <b>let</b> validator_with_commission = <a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">ValidatorConfigurationWithCommission</a> {
            validator_config: <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> validators),
            commission_percentage: 0,
            join_during_genesis: <b>true</b>,
        };
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> validators_with_commission, validator_with_commission);

        i = i + 1;
    };

    <a href="genesis.md#0x1_genesis_create_initialize_validators_with_commission">create_initialize_validators_with_commission</a>(aptos_framework, <b>false</b>, validators_with_commission);
}
</code></pre>



</details>

<a name="0x1_genesis_create_initialize_validator"></a>

## Function `create_initialize_validator`



<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_initialize_validator">create_initialize_validator</a>(aptos_framework: &<a href="">signer</a>, commission_config: &<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">genesis::ValidatorConfigurationWithCommission</a>, _use_staking_contract: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_create_initialize_validator">create_initialize_validator</a>(
    aptos_framework: &<a href="">signer</a>,
    commission_config: &<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">ValidatorConfigurationWithCommission</a>,
    _use_staking_contract: bool,
) {
    <b>let</b> validator = &commission_config.validator_config;

    <b>let</b> owner = &<a href="genesis.md#0x1_genesis_create_account">create_account</a>(aptos_framework, validator.owner_address, validator.stake_amount);
    <a href="genesis.md#0x1_genesis_create_account">create_account</a>(aptos_framework, validator.operator_address, 0);
    <a href="genesis.md#0x1_genesis_create_account">create_account</a>(aptos_framework, validator.voter_address, 0);

    // Initialize the <a href="stake.md#0x1_stake">stake</a> pool and join the validator set.
    // <b>let</b> pool_address = <b>if</b> (use_staking_contract) {

    //     staking_contract::create_staking_contract(
    //         owner,
    //         validator.operator_address,
    //         validator.voter_address,
    //         validator.stake_amount,
    //         commission_config.commission_percentage,
    //         x"",
    //     );
    //     staking_contract::stake_pool_address(validator.owner_address, validator.operator_address)
    // } <b>else</b>
    <b>let</b> pool_address = {


        <a href="stake.md#0x1_stake_initialize_stake_owner">stake::initialize_stake_owner</a>(
            owner,
            validator.stake_amount,
            validator.operator_address,
            validator.voter_address,
        );

        validator.owner_address
    };


    <b>if</b> (commission_config.join_during_genesis) {

        <a href="genesis.md#0x1_genesis_initialize_validator">initialize_validator</a>(pool_address, validator);
    };
}
</code></pre>



</details>

<a name="0x1_genesis_initialize_validator"></a>

## Function `initialize_validator`



<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_validator">initialize_validator</a>(pool_address: <b>address</b>, validator: &<a href="genesis.md#0x1_genesis_ValidatorConfiguration">genesis::ValidatorConfiguration</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_validator">initialize_validator</a>(pool_address: <b>address</b>, validator: &<a href="genesis.md#0x1_genesis_ValidatorConfiguration">ValidatorConfiguration</a>) {
    <b>let</b> operator = &<a href="create_signer.md#0x1_create_signer">create_signer</a>(validator.operator_address);

    <a href="stake.md#0x1_stake_rotate_consensus_key">stake::rotate_consensus_key</a>(
        operator,
        pool_address,
        validator.consensus_pubkey,
        validator.proof_of_possession,
    );

    <a href="stake.md#0x1_stake_update_network_and_fullnode_addresses">stake::update_network_and_fullnode_addresses</a>(
        operator,
        pool_address,
        validator.network_addresses,
        validator.full_node_network_addresses,
    );

    <a href="stake.md#0x1_stake_join_validator_set_internal">stake::join_validator_set_internal</a>(operator, pool_address);

}
</code></pre>



</details>

<a name="0x1_genesis_set_genesis_end"></a>

## Function `set_genesis_end`

The last step of genesis.


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_set_genesis_end">set_genesis_end</a>(aptos_framework: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_set_genesis_end">set_genesis_end</a>(aptos_framework: &<a href="">signer</a>) {
    <a href="chain_status.md#0x1_chain_status_set_genesis_end">chain_status::set_genesis_end</a>(aptos_framework);
}
</code></pre>



</details>

<a name="0x1_genesis_initialize_for_verification"></a>

## Function `initialize_for_verification`



<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_for_verification">initialize_for_verification</a>(<a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="">vector</a>&lt;u8&gt;, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, initial_version: u64, <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>: <a href="">vector</a>&lt;u8&gt;, <a href="execution_config.md#0x1_execution_config">execution_config</a>: <a href="">vector</a>&lt;u8&gt;, epoch_interval_microsecs: u64, minimum_stake: u64, maximum_stake: u64, recurring_lockup_duration_secs: u64, allow_validator_set_change: bool, rewards_rate: u64, rewards_rate_denominator: u64, voting_power_increase_limit: u64, aptos_framework: &<a href="">signer</a>, min_voting_threshold: u128, required_proposer_stake: u64, voting_duration_secs: u64, accounts: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_AccountMap">genesis::AccountMap</a>&gt;, _employee_vesting_start: u64, _employee_vesting_period_duration: u64, _employees: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_EmployeeAccountMap">genesis::EmployeeAccountMap</a>&gt;, validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">genesis::ValidatorConfigurationWithCommission</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_for_verification">initialize_for_verification</a>(
    <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="">vector</a>&lt;u8&gt;,
    <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8,
    initial_version: u64,
    <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>: <a href="">vector</a>&lt;u8&gt;,
    <a href="execution_config.md#0x1_execution_config">execution_config</a>: <a href="">vector</a>&lt;u8&gt;,
    epoch_interval_microsecs: u64,
    minimum_stake: u64,
    maximum_stake: u64,
    recurring_lockup_duration_secs: u64,
    allow_validator_set_change: bool,
    rewards_rate: u64,
    rewards_rate_denominator: u64,
    voting_power_increase_limit: u64,
    aptos_framework: &<a href="">signer</a>,
    min_voting_threshold: u128,
    required_proposer_stake: u64,
    voting_duration_secs: u64,
    accounts: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_AccountMap">AccountMap</a>&gt;,
    _employee_vesting_start: u64,
    _employee_vesting_period_duration: u64,
    _employees: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_EmployeeAccountMap">EmployeeAccountMap</a>&gt;,
    validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">ValidatorConfigurationWithCommission</a>&gt;
) {
    <a href="genesis.md#0x1_genesis_initialize">initialize</a>(
        <a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>,
        <a href="chain_id.md#0x1_chain_id">chain_id</a>,
        initial_version,
        <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>,
        <a href="execution_config.md#0x1_execution_config">execution_config</a>,
        epoch_interval_microsecs,
        minimum_stake,
        maximum_stake,
        recurring_lockup_duration_secs,
        allow_validator_set_change,
        rewards_rate,
        rewards_rate_denominator,
        voting_power_increase_limit
    );
    <a href="_change_feature_flags">features::change_feature_flags</a>(aptos_framework, <a href="">vector</a>[1, 2], <a href="">vector</a>[]);
    <a href="genesis.md#0x1_genesis_initialize_aptos_coin">initialize_aptos_coin</a>(aptos_framework);
    <a href="aptos_governance.md#0x1_aptos_governance_initialize_for_verification">aptos_governance::initialize_for_verification</a>(
        aptos_framework,
        min_voting_threshold,
        required_proposer_stake,
        voting_duration_secs
    );
    <a href="genesis.md#0x1_genesis_create_accounts">create_accounts</a>(aptos_framework, accounts);
    // create_employee_validators(employee_vesting_start, employee_vesting_period_duration, employees);
    <a href="genesis.md#0x1_genesis_create_initialize_validators_with_commission">create_initialize_validators_with_commission</a>(aptos_framework, <b>true</b>, validators);
    <a href="genesis.md#0x1_genesis_set_genesis_end">set_genesis_end</a>(aptos_framework);
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_initialize_for_verification"></a>

### Function `initialize_for_verification`


<pre><code><b>fun</b> <a href="genesis.md#0x1_genesis_initialize_for_verification">initialize_for_verification</a>(<a href="gas_schedule.md#0x1_gas_schedule">gas_schedule</a>: <a href="">vector</a>&lt;u8&gt;, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, initial_version: u64, <a href="consensus_config.md#0x1_consensus_config">consensus_config</a>: <a href="">vector</a>&lt;u8&gt;, <a href="execution_config.md#0x1_execution_config">execution_config</a>: <a href="">vector</a>&lt;u8&gt;, epoch_interval_microsecs: u64, minimum_stake: u64, maximum_stake: u64, recurring_lockup_duration_secs: u64, allow_validator_set_change: bool, rewards_rate: u64, rewards_rate_denominator: u64, voting_power_increase_limit: u64, aptos_framework: &<a href="">signer</a>, min_voting_threshold: u128, required_proposer_stake: u64, voting_duration_secs: u64, accounts: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_AccountMap">genesis::AccountMap</a>&gt;, _employee_vesting_start: u64, _employee_vesting_period_duration: u64, _employees: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_EmployeeAccountMap">genesis::EmployeeAccountMap</a>&gt;, validators: <a href="">vector</a>&lt;<a href="genesis.md#0x1_genesis_ValidatorConfigurationWithCommission">genesis::ValidatorConfigurationWithCommission</a>&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>true</b>;
</code></pre>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
