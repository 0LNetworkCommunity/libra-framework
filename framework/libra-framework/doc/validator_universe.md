
<a name="0x1_validator_universe"></a>

# Module `0x1::validator_universe`



-  [Resource `ValidatorUniverse`](#0x1_validator_universe_ValidatorUniverse)
-  [Function `initialize`](#0x1_validator_universe_initialize)
-  [Function `register_validator`](#0x1_validator_universe_register_validator)
-  [Function `add`](#0x1_validator_universe_add)
-  [Function `maybe_jail`](#0x1_validator_universe_maybe_jail)
-  [Function `maybe_jail_impl`](#0x1_validator_universe_maybe_jail_impl)
-  [Function `genesis_helper_add_validator`](#0x1_validator_universe_genesis_helper_add_validator)
-  [Function `get_eligible_validators`](#0x1_validator_universe_get_eligible_validators)
-  [Function `is_in_universe`](#0x1_validator_universe_is_in_universe)


<pre><code><b>use</b> <a href="cases.md#0x1_cases">0x1::cases</a>;
<b>use</b> <a href="jail.md#0x1_jail">0x1::jail</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_validator_universe_ValidatorUniverse"></a>

## Resource `ValidatorUniverse`



<pre><code><b>struct</b> <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>validators: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_validator_universe_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_initialize">initialize</a>(vm: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_initialize">initialize</a>(vm: &<a href="">signer</a>){
  // Check for transactions sender is association
  <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(vm);
  <b>move_to</b>&lt;<a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a>&gt;(vm, <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> {
      validators: <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;()
  });
}
</code></pre>



</details>

<a name="0x1_validator_universe_register_validator"></a>

## Function `register_validator`

This is the entrypoint for a validator joining the network.
Separates the logic of registration from validator election etc. (in stake.move).
This prevents dependency cycling issues, since stake.move is a large module.


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_register_validator">register_validator</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, consensus_pubkey: <a href="">vector</a>&lt;u8&gt;, proof_of_possession: <a href="">vector</a>&lt;u8&gt;, network_addresses: <a href="">vector</a>&lt;u8&gt;, fullnode_addresses: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_register_validator">register_validator</a>(
  <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>,
  consensus_pubkey: <a href="">vector</a>&lt;u8&gt;,
  proof_of_possession: <a href="">vector</a>&lt;u8&gt;,
  network_addresses: <a href="">vector</a>&lt;u8&gt;,
  fullnode_addresses: <a href="">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> {
    <a href="stake.md#0x1_stake_initialize_validator">stake::initialize_validator</a>(<a href="account.md#0x1_account">account</a>, consensus_pubkey, proof_of_possession, network_addresses, fullnode_addresses);
    // 0L specific,
    <a href="validator_universe.md#0x1_validator_universe_add">add</a>(<a href="account.md#0x1_account">account</a>);
    <a href="jail.md#0x1_jail_init">jail::init</a>(<a href="account.md#0x1_account">account</a>);
}
</code></pre>



</details>

<a name="0x1_validator_universe_add"></a>

## Function `add`

This function is called to add validator to the validator universe.
it can only be called by <code><a href="stake.md#0x1_stake">stake</a></code> module, on validator registration.


<pre><code><b>fun</b> <a href="validator_universe.md#0x1_validator_universe_add">add</a>(sender: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="validator_universe.md#0x1_validator_universe_add">add</a>(sender: &<a href="">signer</a>) <b>acquires</b> <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> {
  <b>let</b> addr = <a href="_address_of">signer::address_of</a>(sender);
  <b>let</b> state = <b>borrow_global</b>&lt;<a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a>&gt;(@aptos_framework);
  <b>let</b> (elegible_list, _) = <a href="_index_of">vector::index_of</a>&lt;<b>address</b>&gt;(&state.validators, &addr);
  <b>if</b> (!elegible_list) {
    <b>let</b> state = <b>borrow_global_mut</b>&lt;<a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a>&gt;(@aptos_framework);
    <a href="_push_back">vector::push_back</a>&lt;<b>address</b>&gt;(&<b>mut</b> state.validators, addr);
  };
  <a href="jail.md#0x1_jail_init">jail::init</a>(sender);
}
</code></pre>



</details>

<a name="0x1_validator_universe_maybe_jail"></a>

## Function `maybe_jail`

Used at epoch boundaries to evaluate the performance of the validator.
only root can call this, and only by friend modules (reconfiguration). Belt and suspenders.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_maybe_jail">maybe_jail</a>(root: &<a href="">signer</a>, validator: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_maybe_jail">maybe_jail</a>(root: &<a href="">signer</a>, validator: <b>address</b>): bool {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <a href="validator_universe.md#0x1_validator_universe_maybe_jail_impl">maybe_jail_impl</a>(root, validator)
}
</code></pre>



</details>

<a name="0x1_validator_universe_maybe_jail_impl"></a>

## Function `maybe_jail_impl`

Common implementation for maybe_jail.


<pre><code><b>fun</b> <a href="validator_universe.md#0x1_validator_universe_maybe_jail_impl">maybe_jail_impl</a>(root: &<a href="">signer</a>, validator: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="validator_universe.md#0x1_validator_universe_maybe_jail_impl">maybe_jail_impl</a>(root: &<a href="">signer</a>, validator: <b>address</b>): bool {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);

  <b>if</b> (
    // TODO check <b>if</b> there are issues <b>with</b> config. belt and suspenders
    <a href="cases.md#0x1_cases_get_case">cases::get_case</a>(validator) == 4

  ) {
    <a href="jail.md#0x1_jail_jail">jail::jail</a>(root, validator);
    <b>return</b> <b>true</b>
  };

  <b>false</b>
}
</code></pre>



</details>

<a name="0x1_validator_universe_genesis_helper_add_validator"></a>

## Function `genesis_helper_add_validator`

For 0L genesis, initialize and add the validators
both root and validator need to sign. This is only possible at genesis.


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_genesis_helper_add_validator">genesis_helper_add_validator</a>(root: &<a href="">signer</a>, validator: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_genesis_helper_add_validator">genesis_helper_add_validator</a>(root: &<a href="">signer</a>, validator: &<a href="">signer</a>) <b>acquires</b> <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <a href="validator_universe.md#0x1_validator_universe_add">add</a>(validator);
}
</code></pre>



</details>

<a name="0x1_validator_universe_get_eligible_validators"></a>

## Function `get_eligible_validators`



<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_get_eligible_validators">get_eligible_validators</a>(): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_get_eligible_validators">get_eligible_validators</a>(): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> {
  <b>let</b> state = <b>borrow_global</b>&lt;<a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a>&gt;(@aptos_framework);
  *&state.validators
}
</code></pre>



</details>

<a name="0x1_validator_universe_is_in_universe"></a>

## Function `is_in_universe`



<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_is_in_universe">is_in_universe</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="validator_universe.md#0x1_validator_universe_is_in_universe">is_in_universe</a>(addr: <b>address</b>): bool <b>acquires</b> <a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a> {
  <b>let</b> state = <b>borrow_global</b>&lt;<a href="validator_universe.md#0x1_validator_universe_ValidatorUniverse">ValidatorUniverse</a>&gt;(@aptos_framework);
  <a href="_contains">vector::contains</a>&lt;<b>address</b>&gt;(&state.validators, &addr)
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
