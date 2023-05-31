
<a name="0x1_transaction_fee"></a>

# Module `0x1::transaction_fee`

This module provides an interface to burn or collect and redistribute transaction fees.


-  [Resource `GasCoinCapabilities`](#0x1_transaction_fee_GasCoinCapabilities)
-  [Resource `CollectedFeesPerBlock`](#0x1_transaction_fee_CollectedFeesPerBlock)
-  [Constants](#@Constants_0)
-  [Function `initialize_fee_collection_and_distribution`](#0x1_transaction_fee_initialize_fee_collection_and_distribution)
-  [Function `is_fees_collection_enabled`](#0x1_transaction_fee_is_fees_collection_enabled)
-  [Function `upgrade_burn_percentage`](#0x1_transaction_fee_upgrade_burn_percentage)
-  [Function `register_proposer_for_fee_collection`](#0x1_transaction_fee_register_proposer_for_fee_collection)
-  [Function `burn_coin_fraction`](#0x1_transaction_fee_burn_coin_fraction)
-  [Function `process_collected_fees`](#0x1_transaction_fee_process_collected_fees)
-  [Function `burn_fee`](#0x1_transaction_fee_burn_fee)
-  [Function `collect_fee`](#0x1_transaction_fee_collect_fee)
-  [Function `pay_fee`](#0x1_transaction_fee_pay_fee)
-  [Function `root_withdraw_all`](#0x1_transaction_fee_root_withdraw_all)
-  [Function `withdraw_all_impl`](#0x1_transaction_fee_withdraw_all_impl)
-  [Function `store_aptos_coin_burn_cap`](#0x1_transaction_fee_store_aptos_coin_burn_cap)
-  [Specification](#@Specification_1)
    -  [Resource `CollectedFeesPerBlock`](#@Specification_1_CollectedFeesPerBlock)
    -  [Function `initialize_fee_collection_and_distribution`](#@Specification_1_initialize_fee_collection_and_distribution)
    -  [Function `upgrade_burn_percentage`](#@Specification_1_upgrade_burn_percentage)
    -  [Function `register_proposer_for_fee_collection`](#@Specification_1_register_proposer_for_fee_collection)
    -  [Function `burn_coin_fraction`](#@Specification_1_burn_coin_fraction)
    -  [Function `process_collected_fees`](#@Specification_1_process_collected_fees)
    -  [Function `burn_fee`](#@Specification_1_burn_fee)
    -  [Function `collect_fee`](#@Specification_1_collect_fee)
    -  [Function `store_aptos_coin_burn_cap`](#@Specification_1_store_aptos_coin_burn_cap)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a name="0x1_transaction_fee_GasCoinCapabilities"></a>

## Resource `GasCoinCapabilities`

Stores burn capability to burn the gas fees.


<pre><code><b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>burn_cap: <a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_transaction_fee_CollectedFeesPerBlock"></a>

## Resource `CollectedFeesPerBlock`

Stores information about the block proposer and the amount of fees
collected when executing the block.


<pre><code><b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: <a href="coin.md#0x1_coin_AggregatableCoin">coin::AggregatableCoin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>burn_percentage: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_transaction_fee_EALREADY_COLLECTING_FEES"></a>

Gas fees are already being collected and the struct holding
information about collected amounts is already published.


<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EALREADY_COLLECTING_FEES">EALREADY_COLLECTING_FEES</a>: u64 = 1;
</code></pre>



<a name="0x1_transaction_fee_EINVALID_BURN_PERCENTAGE"></a>

The burn percentage is out of range [0, 100].


<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_EINVALID_BURN_PERCENTAGE">EINVALID_BURN_PERCENTAGE</a>: u64 = 3;
</code></pre>



<a name="0x1_transaction_fee_initialize_fee_collection_and_distribution"></a>

## Function `initialize_fee_collection_and_distribution`

Initializes the resource storing information about gas fees collection and
distribution. Should be called by on-chain governance.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">initialize_fee_collection_and_distribution</a>(aptos_framework: &<a href="">signer</a>, burn_percentage: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">initialize_fee_collection_and_distribution</a>(aptos_framework: &<a href="">signer</a>, burn_percentage: u8) {
    <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(aptos_framework);
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework),
        <a href="_already_exists">error::already_exists</a>(<a href="transaction_fee.md#0x1_transaction_fee_EALREADY_COLLECTING_FEES">EALREADY_COLLECTING_FEES</a>)
    );
    <b>assert</b>!(burn_percentage &lt;= 100, <a href="_out_of_range">error::out_of_range</a>(<a href="transaction_fee.md#0x1_transaction_fee_EINVALID_BURN_PERCENTAGE">EINVALID_BURN_PERCENTAGE</a>));

    // Make sure stakng <b>module</b> is aware of transaction fees collection.
    // stake::initialize_validator_fees(aptos_framework);

    // Initially, no fees are collected and the <a href="block.md#0x1_block">block</a> proposer is not set.
    <b>let</b> collected_fees = <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
        amount: <a href="coin.md#0x1_coin_initialize_aggregatable_coin">coin::initialize_aggregatable_coin</a>(aptos_framework),
        proposer: <a href="_none">option::none</a>(),
        burn_percentage,
    };
    <b>move_to</b>(aptos_framework, collected_fees);
}
</code></pre>



</details>

<a name="0x1_transaction_fee_is_fees_collection_enabled"></a>

## Function `is_fees_collection_enabled`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>(): bool {
    <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework)
}
</code></pre>



</details>

<a name="0x1_transaction_fee_upgrade_burn_percentage"></a>

## Function `upgrade_burn_percentage`

Sets the burn percentage for collected fees to a new value. Should be called by on-chain governance.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_upgrade_burn_percentage">upgrade_burn_percentage</a>(aptos_framework: &<a href="">signer</a>, new_burn_percentage: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_upgrade_burn_percentage">upgrade_burn_percentage</a>(
    aptos_framework: &<a href="">signer</a>,
    new_burn_percentage: u8
) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>, <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(aptos_framework);
    <b>assert</b>!(new_burn_percentage &lt;= 100, <a href="_out_of_range">error::out_of_range</a>(<a href="transaction_fee.md#0x1_transaction_fee_EINVALID_BURN_PERCENTAGE">EINVALID_BURN_PERCENTAGE</a>));

    // Prior <b>to</b> upgrading the burn percentage, make sure <b>to</b> process collected
    // fees. Otherwise we would <b>use</b> the new (incorrect) burn_percentage when
    // processing fees later!
    <a href="transaction_fee.md#0x1_transaction_fee_process_collected_fees">process_collected_fees</a>();

    <b>if</b> (<a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>()) {
        // Upgrade <b>has</b> no effect unless fees are being collected.
        <b>let</b> burn_percentage = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework).burn_percentage;
        *burn_percentage = new_burn_percentage
    }
}
</code></pre>



</details>

<a name="0x1_transaction_fee_register_proposer_for_fee_collection"></a>

## Function `register_proposer_for_fee_collection`

Registers the proposer of the block for gas fees collection. This function
can only be called at the beginning of the block.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_register_proposer_for_fee_collection">register_proposer_for_fee_collection</a>(proposer_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_register_proposer_for_fee_collection">register_proposer_for_fee_collection</a>(proposer_addr: <b>address</b>) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
    <b>if</b> (<a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>()) {
        <b>let</b> collected_fees = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework);
        <b>let</b> _ = <a href="_swap_or_fill">option::swap_or_fill</a>(&<b>mut</b> collected_fees.proposer, proposer_addr);
    }
}
</code></pre>



</details>

<a name="0x1_transaction_fee_burn_coin_fraction"></a>

## Function `burn_coin_fraction`

Burns a specified fraction of the coin.


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_coin_fraction">burn_coin_fraction</a>(<a href="coin.md#0x1_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;, burn_percentage: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_coin_fraction">burn_coin_fraction</a>(<a href="coin.md#0x1_coin">coin</a>: &<b>mut</b> Coin&lt;GasCoin&gt;, burn_percentage: u8) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a> {
    <b>assert</b>!(burn_percentage &lt;= 100, <a href="_out_of_range">error::out_of_range</a>(<a href="transaction_fee.md#0x1_transaction_fee_EINVALID_BURN_PERCENTAGE">EINVALID_BURN_PERCENTAGE</a>));

    <b>let</b> collected_amount = <a href="coin.md#0x1_coin_value">coin::value</a>(<a href="coin.md#0x1_coin">coin</a>);
    <b>spec</b> {
        // We <b>assume</b> that `burn_percentage * collected_amount` does not overflow.
        <b>assume</b> burn_percentage * collected_amount &lt;= MAX_U64;
    };
    <b>let</b> amount_to_burn = (burn_percentage <b>as</b> u64) * collected_amount / 100;
    <b>if</b> (amount_to_burn &gt; 0) {
        <b>let</b> coin_to_burn = <a href="coin.md#0x1_coin_extract">coin::extract</a>(<a href="coin.md#0x1_coin">coin</a>, amount_to_burn);
        <a href="coin.md#0x1_coin_burn">coin::burn</a>(
            coin_to_burn,
            &<b>borrow_global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(@aptos_framework).burn_cap,
        );
    }
}
</code></pre>



</details>

<a name="0x1_transaction_fee_process_collected_fees"></a>

## Function `process_collected_fees`

Calculates the fee which should be distributed to the block proposer at the
end of an epoch, and records it in the system. This function can only be called
at the beginning of the block or during reconfiguration.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_process_collected_fees">process_collected_fees</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_process_collected_fees">process_collected_fees</a>() <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>, <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
    <b>if</b> (!<a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>()) {
        <b>return</b>
    };
    <b>let</b> collected_fees = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework);

    // If there are no collected fees, only unset the proposer. See the rationale for
    // setting proposer <b>to</b> <a href="_none">option::none</a>() below.
    <b>if</b> (<a href="coin.md#0x1_coin_is_aggregatable_coin_zero">coin::is_aggregatable_coin_zero</a>(&collected_fees.amount)) {
        <b>if</b> (<a href="_is_some">option::is_some</a>(&collected_fees.proposer)) {
            <b>let</b> _ = <a href="_extract">option::extract</a>(&<b>mut</b> collected_fees.proposer);
        };
        <b>return</b>
    };

    // Otherwise get the collected fee, and check <b>if</b> it can distributed later.
    <b>let</b> <a href="coin.md#0x1_coin">coin</a> = <a href="coin.md#0x1_coin_drain_aggregatable_coin">coin::drain_aggregatable_coin</a>(&<b>mut</b> collected_fees.amount);
    // <b>if</b> (<a href="_is_some">option::is_some</a>(&collected_fees.proposer)) {
    //     // Extract the <b>address</b> of proposer here and reset it <b>to</b> <a href="_none">option::none</a>(). This
    //     // is particularly useful <b>to</b> avoid <a href="">any</a> undesired side-effects <b>where</b> coins are
    //     // collected but never distributed or distributed <b>to</b> the wrong <a href="account.md#0x1_account">account</a>.
    //     // With this design, processing collected fees enforces that all fees will be burnt
    //     // unless the proposer is specified in the <a href="block.md#0x1_block">block</a> prologue. When we have a governance
    //     // proposal that triggers <a href="reconfiguration.md#0x1_reconfiguration">reconfiguration</a>, we distribute pending fees and burn the
    //     // fee for the proposal. Otherwise, that fee would be leaked <b>to</b> the next <a href="block.md#0x1_block">block</a>.
    //     <b>let</b> proposer = <a href="_extract">option::extract</a>(&<b>mut</b> collected_fees.proposer);

    //     // Since the <a href="block.md#0x1_block">block</a> can be produced by the VM itself, we have <b>to</b> make sure we catch
    //     // this case.
    //     <b>if</b> (proposer == @vm_reserved) {
    //         <a href="transaction_fee.md#0x1_transaction_fee_burn_coin_fraction">burn_coin_fraction</a>(&<b>mut</b> <a href="coin.md#0x1_coin">coin</a>, 100);
    //         <a href="coin.md#0x1_coin_destroy_zero">coin::destroy_zero</a>(<a href="coin.md#0x1_coin">coin</a>);
    //         <b>return</b>
    //     };

    //     <a href="transaction_fee.md#0x1_transaction_fee_burn_coin_fraction">burn_coin_fraction</a>(&<b>mut</b> <a href="coin.md#0x1_coin">coin</a>, collected_fees.burn_percentage);
    //     // <a href="coin.md#0x1_coin_burn">coin::burn</a>()
    //     // stake::add_transaction_fee(proposer, <a href="coin.md#0x1_coin">coin</a>);
    //     <b>return</b>
    // };

    // If checks did not pass, simply burn all collected coins and <b>return</b> none.
    <a href="transaction_fee.md#0x1_transaction_fee_burn_coin_fraction">burn_coin_fraction</a>(&<b>mut</b> <a href="coin.md#0x1_coin">coin</a>, 100);
    <a href="coin.md#0x1_coin_destroy_zero">coin::destroy_zero</a>(<a href="coin.md#0x1_coin">coin</a>)
}
</code></pre>



</details>

<a name="0x1_transaction_fee_burn_fee"></a>

## Function `burn_fee`

Burn transaction fees in epilogue.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee">burn_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee">burn_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a> {
    <a href="coin.md#0x1_coin_burn_from">coin::burn_from</a>&lt;GasCoin&gt;(
        <a href="account.md#0x1_account">account</a>,
        fee,
        &<b>borrow_global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(@aptos_framework).burn_cap,
    );
}
</code></pre>



</details>

<a name="0x1_transaction_fee_collect_fee"></a>

## Function `collect_fee`

Collect transaction fees in epilogue.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_collect_fee">collect_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_collect_fee">collect_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
    <b>let</b> collected_fees = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework);

    // Here, we are always optimistic and always collect fees. If the proposer is not set,
    // or we cannot redistribute fees later for some reason (e.g. <a href="account.md#0x1_account">account</a> cannot receive AptoCoin)
    // we burn them all at once. This way we avoid having a check for every transaction epilogue.
    <b>let</b> collected_amount = &<b>mut</b> collected_fees.amount;
    <a href="coin.md#0x1_coin_collect_into_aggregatable_coin">coin::collect_into_aggregatable_coin</a>&lt;GasCoin&gt;(<a href="account.md#0x1_account">account</a>, fee, collected_amount);
}
</code></pre>



</details>

<a name="0x1_transaction_fee_pay_fee"></a>

## Function `pay_fee`

pay a fee


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(_sender: &<a href="">signer</a>, fee: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(_sender: &<a href="">signer</a>, fee: Coin&lt;GasCoin&gt;) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
    // TODO: need <b>to</b> track who is making payments.

    <b>let</b> collected_fees = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework);

    // Here, we are always optimistic and always collect fees. If the proposer is not set,
    // or we cannot redistribute fees later for some reason (e.g. <a href="account.md#0x1_account">account</a> cannot receive AptoCoin)
    // we burn them all at once. This way we avoid having a check for every transaction epilogue.
    <b>let</b> collected_amount = &<b>mut</b> collected_fees.amount;
    <a href="coin.md#0x1_coin_merge_aggregatable_coin">coin::merge_aggregatable_coin</a>&lt;GasCoin&gt;(collected_amount, fee);
}
</code></pre>



</details>

<a name="0x1_transaction_fee_root_withdraw_all"></a>

## Function `root_withdraw_all`

withdraw from system transaction account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_root_withdraw_all">root_withdraw_all</a>(root: &<a href="">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_root_withdraw_all">root_withdraw_all</a>(root: &<a href="">signer</a>): Coin&lt;GasCoin&gt; <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <a href="transaction_fee.md#0x1_transaction_fee_withdraw_all_impl">withdraw_all_impl</a>(root)
}
</code></pre>



</details>

<a name="0x1_transaction_fee_withdraw_all_impl"></a>

## Function `withdraw_all_impl`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_withdraw_all_impl">withdraw_all_impl</a>(root: &<a href="">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_withdraw_all_impl">withdraw_all_impl</a>(root: &<a href="">signer</a>): Coin&lt;GasCoin&gt; <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);

  <b>let</b> collected_fees = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework);

  <a href="coin.md#0x1_coin_drain_aggregatable_coin">coin::drain_aggregatable_coin</a>&lt;GasCoin&gt;(&<b>mut</b> collected_fees.amount)
}
</code></pre>



</details>

<a name="0x1_transaction_fee_store_aptos_coin_burn_cap"></a>

## Function `store_aptos_coin_burn_cap`

Only called during genesis.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_aptos_coin_burn_cap">store_aptos_coin_burn_cap</a>(aptos_framework: &<a href="">signer</a>, burn_cap: <a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_aptos_coin_burn_cap">store_aptos_coin_burn_cap</a>(aptos_framework: &<a href="">signer</a>, burn_cap: BurnCapability&lt;GasCoin&gt;) {
    <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(aptos_framework);
    <b>move_to</b>(aptos_framework, <a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a> { burn_cap })
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict;
<b>invariant</b> [suspendable] <a href="chain_status.md#0x1_chain_status_is_operating">chain_status::is_operating</a>() ==&gt; <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(@aptos_framework);
</code></pre>



<a name="@Specification_1_CollectedFeesPerBlock"></a>

### Resource `CollectedFeesPerBlock`


<pre><code><b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a> <b>has</b> key
</code></pre>



<dl>
<dt>
<code>amount: <a href="coin.md#0x1_coin_AggregatableCoin">coin::AggregatableCoin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>proposer: <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>burn_percentage: u8</code>
</dt>
<dd>

</dd>
</dl>



<pre><code><b>invariant</b> burn_percentage &lt;= 100;
</code></pre>



<a name="@Specification_1_initialize_fee_collection_and_distribution"></a>

### Function `initialize_fee_collection_and_distribution`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize_fee_collection_and_distribution">initialize_fee_collection_and_distribution</a>(aptos_framework: &<a href="">signer</a>, burn_percentage: u8)
</code></pre>




<pre><code><b>pragma</b> verify=<b>false</b>;
</code></pre>



<a name="@Specification_1_upgrade_burn_percentage"></a>

### Function `upgrade_burn_percentage`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_upgrade_burn_percentage">upgrade_burn_percentage</a>(aptos_framework: &<a href="">signer</a>, new_burn_percentage: u8)
</code></pre>




<pre><code><b>pragma</b> verify=<b>false</b>;
</code></pre>



<a name="@Specification_1_register_proposer_for_fee_collection"></a>

### Function `register_proposer_for_fee_collection`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_register_proposer_for_fee_collection">register_proposer_for_fee_collection</a>(proposer_addr: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>() ==&gt;
    <a href="_spec_borrow">option::spec_borrow</a>(<b>global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework).proposer) == proposer_addr;
</code></pre>



<a name="@Specification_1_burn_coin_fraction"></a>

### Function `burn_coin_fraction`


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_coin_fraction">burn_coin_fraction</a>(<a href="coin.md#0x1_coin">coin</a>: &<b>mut</b> <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;, burn_percentage: u8)
</code></pre>




<pre><code><b>requires</b> burn_percentage &lt;= 100;
<b>requires</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(@aptos_framework);
<b>requires</b> <b>exists</b>&lt;CoinInfo&lt;GasCoin&gt;&gt;(@aptos_framework);
<b>let</b> amount_to_burn = (burn_percentage * <a href="coin.md#0x1_coin_value">coin::value</a>(<a href="coin.md#0x1_coin">coin</a>)) / 100;
<b>let</b> maybe_supply = <a href="coin.md#0x1_coin_get_coin_supply_opt">coin::get_coin_supply_opt</a>&lt;GasCoin&gt;();
<b>aborts_if</b> amount_to_burn &gt; 0 && <a href="_is_some">option::is_some</a>(maybe_supply) && <a href="optional_aggregator.md#0x1_optional_aggregator_is_parallelizable">optional_aggregator::is_parallelizable</a>(<a href="_borrow">option::borrow</a>(maybe_supply))
    && <a href="aggregator.md#0x1_aggregator_spec_aggregator_get_val">aggregator::spec_aggregator_get_val</a>(<a href="_borrow">option::borrow</a>(<a href="_borrow">option::borrow</a>(maybe_supply).<a href="aggregator.md#0x1_aggregator">aggregator</a>)) &lt;
    amount_to_burn;
<b>aborts_if</b> <a href="_is_some">option::is_some</a>(maybe_supply) && !<a href="optional_aggregator.md#0x1_optional_aggregator_is_parallelizable">optional_aggregator::is_parallelizable</a>(<a href="_borrow">option::borrow</a>(maybe_supply))
    && <a href="_borrow">option::borrow</a>(<a href="_borrow">option::borrow</a>(maybe_supply).integer).value &lt;
    amount_to_burn;
<b>include</b> (amount_to_burn &gt; 0) ==&gt; <a href="coin.md#0x1_coin_AbortsIfNotExistCoinInfo">coin::AbortsIfNotExistCoinInfo</a>&lt;GasCoin&gt;;
</code></pre>




<a name="0x1_transaction_fee_collectedFeesAggregator"></a>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_collectedFeesAggregator">collectedFeesAggregator</a>(): AggregatableCoin&lt;GasCoin&gt; {
   <b>global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework).amount
}
</code></pre>




<a name="0x1_transaction_fee_RequiresCollectedFeesPerValueLeqBlockAptosSupply"></a>


<pre><code><b>schema</b> <a href="transaction_fee.md#0x1_transaction_fee_RequiresCollectedFeesPerValueLeqBlockAptosSupply">RequiresCollectedFeesPerValueLeqBlockAptosSupply</a> {
    <b>let</b> maybe_supply = <a href="coin.md#0x1_coin_get_coin_supply_opt">coin::get_coin_supply_opt</a>&lt;GasCoin&gt;();
    <b>requires</b>
        (<a href="transaction_fee.md#0x1_transaction_fee_is_fees_collection_enabled">is_fees_collection_enabled</a>() && <a href="_is_some">option::is_some</a>(maybe_supply)) ==&gt;
            (<a href="aggregator.md#0x1_aggregator_spec_aggregator_get_val">aggregator::spec_aggregator_get_val</a>(<b>global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework).amount.value) &lt;=
                <a href="optional_aggregator.md#0x1_optional_aggregator_optional_aggregator_value">optional_aggregator::optional_aggregator_value</a>(<a href="_spec_borrow">option::spec_borrow</a>(<a href="coin.md#0x1_coin_get_coin_supply_opt">coin::get_coin_supply_opt</a>&lt;GasCoin&gt;())));
}
</code></pre>



<a name="@Specification_1_process_collected_fees"></a>

### Function `process_collected_fees`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_process_collected_fees">process_collected_fees</a>()
</code></pre>




<pre><code><b>requires</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(@aptos_framework);
<b>requires</b> <b>exists</b>&lt;<a href="stake.md#0x1_stake_ValidatorFees">stake::ValidatorFees</a>&gt;(@aptos_framework);
<b>requires</b> <b>exists</b>&lt;CoinInfo&lt;GasCoin&gt;&gt;(@aptos_framework);
<b>include</b> <a href="transaction_fee.md#0x1_transaction_fee_RequiresCollectedFeesPerValueLeqBlockAptosSupply">RequiresCollectedFeesPerValueLeqBlockAptosSupply</a>;
</code></pre>



<a name="@Specification_1_burn_fee"></a>

### Function `burn_fee`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_burn_fee">burn_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64)
</code></pre>


<code><a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a></code> should be exists.


<pre><code><b>pragma</b> aborts_if_is_partial;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(@aptos_framework);
</code></pre>



<a name="@Specification_1_collect_fee"></a>

### Function `collect_fee`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_collect_fee">collect_fee</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, fee: u64)
</code></pre>




<pre><code><b>let</b> collected_fees = <b>global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework).amount;
<b>let</b> aggr = collected_fees.value;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_CollectedFeesPerBlock">CollectedFeesPerBlock</a>&gt;(@aptos_framework);
<b>aborts_if</b> fee &gt; 0 && !<b>exists</b>&lt;<a href="coin.md#0x1_coin_CoinStore">coin::CoinStore</a>&lt;GasCoin&gt;&gt;(<a href="account.md#0x1_account">account</a>);
<b>aborts_if</b> fee &gt; 0 && <b>global</b>&lt;<a href="coin.md#0x1_coin_CoinStore">coin::CoinStore</a>&lt;GasCoin&gt;&gt;(<a href="account.md#0x1_account">account</a>).<a href="coin.md#0x1_coin">coin</a>.value &lt; fee;
<b>aborts_if</b> fee &gt; 0 && <a href="aggregator.md#0x1_aggregator_spec_aggregator_get_val">aggregator::spec_aggregator_get_val</a>(aggr)
    + fee &gt; <a href="aggregator.md#0x1_aggregator_spec_get_limit">aggregator::spec_get_limit</a>(aggr);
<b>aborts_if</b> fee &gt; 0 && <a href="aggregator.md#0x1_aggregator_spec_aggregator_get_val">aggregator::spec_aggregator_get_val</a>(aggr)
    + fee &gt; MAX_U128;
</code></pre>



<a name="@Specification_1_store_aptos_coin_burn_cap"></a>

### Function `store_aptos_coin_burn_cap`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_store_aptos_coin_burn_cap">store_aptos_coin_burn_cap</a>(aptos_framework: &<a href="">signer</a>, burn_cap: <a href="coin.md#0x1_coin_BurnCapability">coin::BurnCapability</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>


Ensure caller is admin.
Aborts if <code><a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a></code> already exists.


<pre><code><b>let</b> addr = <a href="_address_of">signer::address_of</a>(aptos_framework);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_aptos_framework_address">system_addresses::is_aptos_framework_address</a>(addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_GasCoinCapabilities">GasCoinCapabilities</a>&gt;(addr);
</code></pre>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
