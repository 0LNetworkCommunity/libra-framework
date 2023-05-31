
<a name="0x1_ol_account"></a>

# Module `0x1::ol_account`



-  [Resource `DirectTransferConfig`](#0x1_ol_account_DirectTransferConfig)
-  [Struct `DirectCoinTransferConfigUpdatedEvent`](#0x1_ol_account_DirectCoinTransferConfigUpdatedEvent)
-  [Constants](#@Constants_0)
-  [Function `user_create_account`](#0x1_ol_account_user_create_account)
-  [Function `vm_create_account_migration`](#0x1_ol_account_vm_create_account_migration)
-  [Function `get_slow_limit`](#0x1_ol_account_get_slow_limit)
-  [Function `assert_account_exists`](#0x1_ol_account_assert_account_exists)
-  [Function `assert_account_is_registered_for_gas`](#0x1_ol_account_assert_account_is_registered_for_gas)
-  [Function `set_allow_direct_coin_transfers`](#0x1_ol_account_set_allow_direct_coin_transfers)
-  [Function `can_receive_direct_coin_transfers`](#0x1_ol_account_can_receive_direct_coin_transfers)
-  [Specification](#@Specification_1)
    -  [Function `assert_account_exists`](#@Specification_1_assert_account_exists)
    -  [Function `assert_account_is_registered_for_gas`](#@Specification_1_assert_account_is_registered_for_gas)
    -  [Function `set_allow_direct_coin_transfers`](#@Specification_1_set_allow_direct_coin_transfers)
    -  [Function `can_receive_direct_coin_transfers`](#@Specification_1_can_receive_direct_coin_transfers)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="chain_status.md#0x1_chain_status">0x1::chain_status</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="slow_wallet.md#0x1_slow_wallet">0x1::slow_wallet</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a name="0x1_ol_account_DirectTransferConfig"></a>

## Resource `DirectTransferConfig`

Configuration for whether an account can receive direct transfers of coins that they have not registered.

By default, this is enabled. Users can opt-out by disabling at any time.


<pre><code><b>struct</b> <a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>allow_arbitrary_coin_transfers: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>update_coin_transfer_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="ol_account.md#0x1_ol_account_DirectCoinTransferConfigUpdatedEvent">ol_account::DirectCoinTransferConfigUpdatedEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ol_account_DirectCoinTransferConfigUpdatedEvent"></a>

## Struct `DirectCoinTransferConfigUpdatedEvent`

Event emitted when an account's direct coins transfer config is updated.


<pre><code><b>struct</b> <a href="ol_account.md#0x1_ol_account_DirectCoinTransferConfigUpdatedEvent">DirectCoinTransferConfigUpdatedEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>new_allow_direct_transfers: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ol_account_EINSUFFICIENT_BALANCE"></a>

for 0L the account which does onboarding needs to have at least 2 gas coins


<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>: u64 = 6;
</code></pre>



<a name="0x1_ol_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS"></a>

Account opted out of receiving coins that they did not register to receive.


<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS">EACCOUNT_DOES_NOT_ACCEPT_DIRECT_COIN_TRANSFERS</a>: u64 = 3;
</code></pre>



<a name="0x1_ol_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS"></a>

Account opted out of directly receiving NFT tokens.


<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS">EACCOUNT_DOES_NOT_ACCEPT_DIRECT_TOKEN_TRANSFERS</a>: u64 = 4;
</code></pre>



<a name="0x1_ol_account_EACCOUNT_NOT_FOUND"></a>

Account does not exist.


<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_EACCOUNT_NOT_FOUND">EACCOUNT_NOT_FOUND</a>: u64 = 1;
</code></pre>



<a name="0x1_ol_account_EACCOUNT_NOT_REGISTERED_FOR_APT"></a>

Account is not registered to receive GAS.


<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_EACCOUNT_NOT_REGISTERED_FOR_APT">EACCOUNT_NOT_REGISTERED_FOR_APT</a>: u64 = 2;
</code></pre>



<a name="0x1_ol_account_EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH"></a>

The lengths of the recipients and amounts lists don't match.


<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH">EMISMATCHING_RECIPIENTS_AND_AMOUNTS_LENGTH</a>: u64 = 5;
</code></pre>



<a name="0x1_ol_account_BOOTSTRAP_GAS_COIN_AMOUNT"></a>



<pre><code><b>const</b> <a href="ol_account.md#0x1_ol_account_BOOTSTRAP_GAS_COIN_AMOUNT">BOOTSTRAP_GAS_COIN_AMOUNT</a>: u64 = 1000000;
</code></pre>



<a name="0x1_ol_account_user_create_account"></a>

## Function `user_create_account`

Users with existsing accounts can onboard new accounts.


<pre><code><b>public</b> entry <b>fun</b> <a href="ol_account.md#0x1_ol_account_user_create_account">user_create_account</a>(sender: &<a href="">signer</a>, auth_key: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="ol_account.md#0x1_ol_account_user_create_account">user_create_account</a>(sender: &<a href="">signer</a>, auth_key: <b>address</b>) {
    <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sender);
    <b>assert</b>!(
        !<a href="account.md#0x1_account_exists_at">account::exists_at</a>(auth_key),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="ol_account.md#0x1_ol_account_EACCOUNT_NOT_FOUND">EACCOUNT_NOT_FOUND</a>),
    );

    <b>assert</b>!(
        (<a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;GasCoin&gt;(sender_addr) &gt; 2 * <a href="ol_account.md#0x1_ol_account_BOOTSTRAP_GAS_COIN_AMOUNT">BOOTSTRAP_GAS_COIN_AMOUNT</a>),
        <a href="_invalid_state">error::invalid_state</a>(<a href="ol_account.md#0x1_ol_account_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>),
    );

    <b>let</b> new_signer = <a href="account.md#0x1_account_create_account">account::create_account</a>(auth_key);
    <a href="coin.md#0x1_coin_register">coin::register</a>&lt;GasCoin&gt;(&new_signer);

    <a href="coin.md#0x1_coin_transfer">coin::transfer</a>&lt;GasCoin&gt;(sender, auth_key, <a href="ol_account.md#0x1_ol_account_BOOTSTRAP_GAS_COIN_AMOUNT">BOOTSTRAP_GAS_COIN_AMOUNT</a>);

}
</code></pre>



</details>

<a name="0x1_ol_account_vm_create_account_migration"></a>

## Function `vm_create_account_migration`

For migrating accounts from a legacy system


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_vm_create_account_migration">vm_create_account_migration</a>(root: &<a href="">signer</a>, new_account: <b>address</b>, auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_vm_create_account_migration">vm_create_account_migration</a>(
    root: &<a href="">signer</a>,
    new_account: <b>address</b>,
    auth_key: <a href="">vector</a>&lt;u8&gt;,
    // value: u64,
) {
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
    <a href="chain_status.md#0x1_chain_status_assert_genesis">chain_status::assert_genesis</a>();
    <b>let</b> new_signer = <a href="account.md#0x1_account_vm_create_account">account::vm_create_account</a>(root, new_account, auth_key);
    // Roles::new_user_role_with_proof(&new_signer);
    // make_account(&new_signer, auth_key);
    <a href="coin.md#0x1_coin_register">coin::register</a>&lt;GasCoin&gt;(&new_signer);
}
</code></pre>



</details>

<a name="0x1_ol_account_get_slow_limit"></a>

## Function `get_slow_limit`



<pre><code><b>fun</b> <a href="ol_account.md#0x1_ol_account_get_slow_limit">get_slow_limit</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ol_account.md#0x1_ol_account_get_slow_limit">get_slow_limit</a>(addr: <b>address</b>): u64 {
  <b>let</b> full_balance = <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;GasCoin&gt;(addr);
  // TODO: check <b>if</b> recipient is a donor directed <a href="account.md#0x1_account">account</a>.
  <b>if</b> (<b>false</b>) { <b>return</b> full_balance };
  <a href="slow_wallet.md#0x1_slow_wallet_unlocked_amount">slow_wallet::unlocked_amount</a>(addr)
}
</code></pre>



</details>

<a name="0x1_ol_account_assert_account_exists"></a>

## Function `assert_account_exists`



<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_assert_account_exists">assert_account_exists</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_assert_account_exists">assert_account_exists</a>(addr: <b>address</b>) {
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">account::exists_at</a>(addr), <a href="_not_found">error::not_found</a>(<a href="ol_account.md#0x1_ol_account_EACCOUNT_NOT_FOUND">EACCOUNT_NOT_FOUND</a>));
}
</code></pre>



</details>

<a name="0x1_ol_account_assert_account_is_registered_for_gas"></a>

## Function `assert_account_is_registered_for_gas`



<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_assert_account_is_registered_for_gas">assert_account_is_registered_for_gas</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_assert_account_is_registered_for_gas">assert_account_is_registered_for_gas</a>(addr: <b>address</b>) {
    <a href="ol_account.md#0x1_ol_account_assert_account_exists">assert_account_exists</a>(addr);
    <b>assert</b>!(<a href="coin.md#0x1_coin_is_account_registered">coin::is_account_registered</a>&lt;GasCoin&gt;(addr), <a href="_not_found">error::not_found</a>(<a href="ol_account.md#0x1_ol_account_EACCOUNT_NOT_REGISTERED_FOR_APT">EACCOUNT_NOT_REGISTERED_FOR_APT</a>));
}
</code></pre>



</details>

<a name="0x1_ol_account_set_allow_direct_coin_transfers"></a>

## Function `set_allow_direct_coin_transfers`

Set whether <code><a href="account.md#0x1_account">account</a></code> can receive direct transfers of coins that they have not explicitly registered to receive.


<pre><code><b>public</b> entry <b>fun</b> <a href="ol_account.md#0x1_ol_account_set_allow_direct_coin_transfers">set_allow_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, allow: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="ol_account.md#0x1_ol_account_set_allow_direct_coin_transfers">set_allow_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, allow: bool) <b>acquires</b> <a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a> {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>if</b> (<b>exists</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(addr)) {
        <b>let</b> direct_transfer_config = <b>borrow_global_mut</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(addr);
        // Short-circuit <b>to</b> avoid emitting an <a href="event.md#0x1_event">event</a> <b>if</b> direct transfer config is not changing.
        <b>if</b> (direct_transfer_config.allow_arbitrary_coin_transfers == allow) {
            <b>return</b>
        };

        direct_transfer_config.allow_arbitrary_coin_transfers = allow;
        emit_event(
            &<b>mut</b> direct_transfer_config.update_coin_transfer_events,
            <a href="ol_account.md#0x1_ol_account_DirectCoinTransferConfigUpdatedEvent">DirectCoinTransferConfigUpdatedEvent</a> { new_allow_direct_transfers: allow });
    } <b>else</b> {
        <b>let</b> direct_transfer_config = <a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a> {
            allow_arbitrary_coin_transfers: allow,
            update_coin_transfer_events: new_event_handle&lt;<a href="ol_account.md#0x1_ol_account_DirectCoinTransferConfigUpdatedEvent">DirectCoinTransferConfigUpdatedEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
        };
        emit_event(
            &<b>mut</b> direct_transfer_config.update_coin_transfer_events,
            <a href="ol_account.md#0x1_ol_account_DirectCoinTransferConfigUpdatedEvent">DirectCoinTransferConfigUpdatedEvent</a> { new_allow_direct_transfers: allow });
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, direct_transfer_config);
    };
}
</code></pre>



</details>

<a name="0x1_ol_account_can_receive_direct_coin_transfers"></a>

## Function `can_receive_direct_coin_transfers`

Return true if <code><a href="account.md#0x1_account">account</a></code> can receive direct transfers of coins that they have not explicitly registered to
receive.

By default, this returns true if an account has not explicitly set whether the can receive direct transfers.


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_can_receive_direct_coin_transfers">can_receive_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_can_receive_direct_coin_transfers">can_receive_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool <b>acquires</b> <a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a> {
    !<b>exists</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(<a href="account.md#0x1_account">account</a>) ||
        <b>borrow_global</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(<a href="account.md#0x1_account">account</a>).allow_arbitrary_coin_transfers
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>




<a name="0x1_ol_account_CreateAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="ol_account.md#0x1_ol_account_CreateAccountAbortsIf">CreateAccountAbortsIf</a> {
    auth_key: <b>address</b>;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(auth_key);
    <b>aborts_if</b> <a href="ol_account.md#0x1_ol_account_length_judgment">length_judgment</a>(auth_key);
    <b>aborts_if</b> auth_key == @vm_reserved || auth_key == @aptos_framework || auth_key == @aptos_token;
}
</code></pre>




<a name="0x1_ol_account_length_judgment"></a>


<pre><code><b>fun</b> <a href="ol_account.md#0x1_ol_account_length_judgment">length_judgment</a>(auth_key: <b>address</b>): bool {
   <b>use</b> std::bcs;

   <b>let</b> authentication_key = <a href="_to_bytes">bcs::to_bytes</a>(auth_key);
   len(authentication_key) != 32
}
</code></pre>



<a name="@Specification_1_assert_account_exists"></a>

### Function `assert_account_exists`


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_assert_account_exists">assert_account_exists</a>(addr: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="account.md#0x1_account_exists_at">account::exists_at</a>(addr);
</code></pre>



<a name="@Specification_1_assert_account_is_registered_for_gas"></a>

### Function `assert_account_is_registered_for_gas`


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_assert_account_is_registered_for_gas">assert_account_is_registered_for_gas</a>(addr: <b>address</b>)
</code></pre>


Check if the address existed.
Check if the GasCoin under the address existed.


<pre><code><b>aborts_if</b> !<a href="account.md#0x1_account_exists_at">account::exists_at</a>(addr);
<b>aborts_if</b> !<a href="coin.md#0x1_coin_is_account_registered">coin::is_account_registered</a>&lt;GasCoin&gt;(addr);
</code></pre>



<a name="@Specification_1_set_allow_direct_coin_transfers"></a>

### Function `set_allow_direct_coin_transfers`


<pre><code><b>public</b> entry <b>fun</b> <a href="ol_account.md#0x1_ol_account_set_allow_direct_coin_transfers">set_allow_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, allow: bool)
</code></pre>




<pre><code><b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>include</b> !<b>exists</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(addr) ==&gt; <a href="account.md#0x1_account_NewEventHandleAbortsIf">account::NewEventHandleAbortsIf</a>;
</code></pre>



<a name="@Specification_1_can_receive_direct_coin_transfers"></a>

### Function `can_receive_direct_coin_transfers`


<pre><code><b>public</b> <b>fun</b> <a href="ol_account.md#0x1_ol_account_can_receive_direct_coin_transfers">can_receive_direct_coin_transfers</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == (
    !<b>exists</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(<a href="account.md#0x1_account">account</a>) ||
        <b>global</b>&lt;<a href="ol_account.md#0x1_ol_account_DirectTransferConfig">DirectTransferConfig</a>&gt;(<a href="account.md#0x1_account">account</a>).allow_arbitrary_coin_transfers
);
</code></pre>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
