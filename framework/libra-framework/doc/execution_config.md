
<a name="0x1_execution_config"></a>

# Module `0x1::execution_config`

Maintains the execution config for the blockchain. The config is stored in a
Reconfiguration, and may be updated by root.


-  [Resource `ExecutionConfig`](#0x1_execution_config_ExecutionConfig)
-  [Constants](#@Constants_0)
-  [Function `set`](#0x1_execution_config_set)
-  [Specification](#@Specification_1)
    -  [Function `set`](#@Specification_1_set)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="reconfiguration.md#0x1_reconfiguration">0x1::reconfiguration</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a name="0x1_execution_config_ExecutionConfig"></a>

## Resource `ExecutionConfig`



<pre><code><b>struct</b> <a href="execution_config.md#0x1_execution_config_ExecutionConfig">ExecutionConfig</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_execution_config_EINVALID_CONFIG"></a>

The provided on chain config bytes are empty or invalid


<pre><code><b>const</b> <a href="execution_config.md#0x1_execution_config_EINVALID_CONFIG">EINVALID_CONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_execution_config_set"></a>

## Function `set`

This can be called by on-chain governance to update on-chain execution configs.


<pre><code><b>public</b> <b>fun</b> <a href="execution_config.md#0x1_execution_config_set">set</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, config: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="execution_config.md#0x1_execution_config_set">set</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, config: <a href="">vector</a>&lt;u8&gt;) <b>acquires</b> <a href="execution_config.md#0x1_execution_config_ExecutionConfig">ExecutionConfig</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<a href="_length">vector::length</a>(&config) &gt; 0, <a href="_invalid_argument">error::invalid_argument</a>(<a href="execution_config.md#0x1_execution_config_EINVALID_CONFIG">EINVALID_CONFIG</a>));

    <b>if</b> (<b>exists</b>&lt;<a href="execution_config.md#0x1_execution_config_ExecutionConfig">ExecutionConfig</a>&gt;(@aptos_framework)) {
        <b>let</b> config_ref = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="execution_config.md#0x1_execution_config_ExecutionConfig">ExecutionConfig</a>&gt;(@aptos_framework).config;
        *config_ref = config;
    } <b>else</b> {
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="execution_config.md#0x1_execution_config_ExecutionConfig">ExecutionConfig</a> { config });
    };
    // Need <b>to</b> trigger <a href="reconfiguration.md#0x1_reconfiguration">reconfiguration</a> so validator nodes can sync on the updated configs.
    <a href="reconfiguration.md#0x1_reconfiguration_reconfigure">reconfiguration::reconfigure</a>();
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="execution_config.md#0x1_execution_config_set">set</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, config: <a href="">vector</a>&lt;u8&gt;)
</code></pre>


Ensure the caller is admin
When setting now time must be later than last_reconfiguration_time.


<pre><code><b>pragma</b> verify = <b>false</b>;
<b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_aptos_framework_address">system_addresses::is_aptos_framework_address</a>(addr);
<b>aborts_if</b> !(len(config) &gt; 0);
<b>requires</b> <a href="chain_status.md#0x1_chain_status_is_operating">chain_status::is_operating</a>();
<b>requires</b> <a href="timestamp.md#0x1_timestamp_spec_now_microseconds">timestamp::spec_now_microseconds</a>() &gt;= <a href="reconfiguration.md#0x1_reconfiguration_last_reconfiguration_time">reconfiguration::last_reconfiguration_time</a>();
</code></pre>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
