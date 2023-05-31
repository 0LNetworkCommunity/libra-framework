
<a name="0x1_jail"></a>

# Module `0x1::jail`



-  [Resource `Jail`](#0x1_jail_Jail)
-  [Constants](#@Constants_0)
-  [Function `init`](#0x1_jail_init)
-  [Function `is_jailed`](#0x1_jail_is_jailed)
-  [Function `jail`](#0x1_jail_jail)
-  [Function `reset_consecutive_fail`](#0x1_jail_reset_consecutive_fail)
-  [Function `unjail_by_voucher`](#0x1_jail_unjail_by_voucher)
-  [Function `unjail`](#0x1_jail_unjail)
-  [Function `sort_by_jail`](#0x1_jail_sort_by_jail)
-  [Function `inc_voucher_jail`](#0x1_jail_inc_voucher_jail)
-  [Function `get_jail_reputation`](#0x1_jail_get_jail_reputation)
-  [Function `get_count_buddies_jailed`](#0x1_jail_get_count_buddies_jailed)
-  [Function `exists_jail`](#0x1_jail_exists_jail)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="vouch.md#0x1_vouch">0x1::vouch</a>;
</code></pre>



<a name="0x1_jail_Jail"></a>

## Resource `Jail`



<pre><code><b>struct</b> <a href="jail.md#0x1_jail_Jail">Jail</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>is_jailed: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>lifetime_jailed: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>lifetime_vouchees_jailed: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>consecutive_failure_to_rejoin: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_jail_EVALIDATOR_CONFIG"></a>

Validator is misconfigured cannot unjail.


<pre><code><b>const</b> <a href="jail.md#0x1_jail_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>: u64 = 10001;
</code></pre>



<a name="0x1_jail_init"></a>

## Function `init`



<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_init">init</a>(val_sig: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_init">init</a>(val_sig: &<a href="">signer</a>) {
  <b>let</b> addr = <a href="_address_of">signer::address_of</a>(val_sig);
  <b>if</b> (!<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr)) {
    <b>move_to</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(val_sig, <a href="jail.md#0x1_jail_Jail">Jail</a> {
      is_jailed: <b>false</b>,
      lifetime_jailed: 0,
      lifetime_vouchees_jailed: 0,
      consecutive_failure_to_rejoin: 0,

    });
  }
}
</code></pre>



</details>

<a name="0x1_jail_is_jailed"></a>

## Function `is_jailed`



<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_is_jailed">is_jailed</a>(validator: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_is_jailed">is_jailed</a>(validator: <b>address</b>): bool <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <b>if</b> (!<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(validator)) {
    <b>return</b> <b>false</b>
  };
  <b>borrow_global</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(validator).is_jailed
}
</code></pre>



</details>

<a name="0x1_jail_jail"></a>

## Function `jail`



<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail">jail</a>(vm: &<a href="">signer</a>, validator: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail">jail</a>(vm: &<a href="">signer</a>, validator: <b>address</b>) <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a>{
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <b>if</b> (<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(validator)) {
    <b>let</b> j = <b>borrow_global_mut</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(validator);
    j.is_jailed = <b>true</b>;
    j.lifetime_jailed = j.lifetime_jailed + 1;
    j.consecutive_failure_to_rejoin = j.consecutive_failure_to_rejoin + 1;
  };

  <a href="jail.md#0x1_jail_inc_voucher_jail">inc_voucher_jail</a>(validator);
}
</code></pre>



</details>

<a name="0x1_jail_reset_consecutive_fail"></a>

## Function `reset_consecutive_fail`

If the validator performs again after having been jailed,
then we can remove the consecutive fails.
Otherwise the lifetime counters on their account, and on buddy Voucher accounts does not get cleared.


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_reset_consecutive_fail">reset_consecutive_fail</a>(root: &<a href="">signer</a>, validator: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_reset_consecutive_fail">reset_consecutive_fail</a>(root: &<a href="">signer</a>, validator: <b>address</b>) <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <b>if</b> (<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(validator)) {
    <b>let</b> j = <b>borrow_global_mut</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(validator);
    j.consecutive_failure_to_rejoin = 0;
  }
}
</code></pre>



</details>

<a name="0x1_jail_unjail_by_voucher"></a>

## Function `unjail_by_voucher`

Only a Voucher of the validator can flip the unjail bit.
This is a way to make sure the validator is ready to rejoin.


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_unjail_by_voucher">unjail_by_voucher</a>(sender: &<a href="">signer</a>, addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_unjail_by_voucher">unjail_by_voucher</a>(sender: &<a href="">signer</a>, addr: <b>address</b>) <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <b>assert</b>!(
    <a href="stake.md#0x1_stake_is_valid">stake::is_valid</a>(addr),
    <a href="_invalid_state">error::invalid_state</a>(<a href="jail.md#0x1_jail_EVALIDATOR_CONFIG">EVALIDATOR_CONFIG</a>),
  );
  <b>let</b> voucher = <a href="_address_of">signer::address_of</a>(sender);
  <b>let</b> buddies = <a href="vouch.md#0x1_vouch_buddies_in_set">vouch::buddies_in_set</a>(addr);
  <b>let</b> (is_found, _idx) = <a href="_index_of">vector::index_of</a>(&buddies, &voucher);
  <b>assert</b>!(is_found, 100103);

  <a href="jail.md#0x1_jail_unjail">unjail</a>(addr);
}
</code></pre>



</details>

<a name="0x1_jail_unjail"></a>

## Function `unjail`



<pre><code><b>fun</b> <a href="jail.md#0x1_jail_unjail">unjail</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="jail.md#0x1_jail_unjail">unjail</a>(addr: <b>address</b>) <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr)) {
    <b>borrow_global_mut</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr).is_jailed = <b>false</b>;
  };
}
</code></pre>



</details>

<a name="0x1_jail_sort_by_jail"></a>

## Function `sort_by_jail`

gets a list of validators based on their jail reputation
this is used in the bidding process for Proof-of-Fee where
we seat the validators with the least amount of consecutive failures
to rejoin.


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_sort_by_jail">sort_by_jail</a>(vec_address: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_sort_by_jail">sort_by_jail</a>(vec_address: <a href="">vector</a>&lt;<b>address</b>&gt;): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {

  // Sorting the accounts <a href="">vector</a> based on value (weights).
  // Bubble sort algorithm
  <b>let</b> length = <a href="_length">vector::length</a>(&vec_address);

  <b>let</b> i = 0;
  <b>while</b> (i &lt; length){
    <b>let</b> j = 0;
    <b>while</b>(j &lt; length-i-1){

      <b>let</b> (_, value_j) = <a href="jail.md#0x1_jail_get_jail_reputation">get_jail_reputation</a>(*<a href="_borrow">vector::borrow</a>(&vec_address, j));
      <b>let</b> (_, value_jp1) = <a href="jail.md#0x1_jail_get_jail_reputation">get_jail_reputation</a>(*<a href="_borrow">vector::borrow</a>(&vec_address, j + 1));

      <b>if</b>(value_j &gt; value_jp1){
        <a href="_swap">vector::swap</a>&lt;<b>address</b>&gt;(&<b>mut</b> vec_address, j, j+1);
      };
      j = j + 1;
    };
    i = i + 1;
  };

  vec_address
}
</code></pre>



</details>

<a name="0x1_jail_inc_voucher_jail"></a>

## Function `inc_voucher_jail`

the Vouchers who vouched for a jailed validator
will get a reputation mark. This is informational currently not used
for any consensus admission or weight etc.


<pre><code><b>fun</b> <a href="jail.md#0x1_jail_inc_voucher_jail">inc_voucher_jail</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="jail.md#0x1_jail_inc_voucher_jail">inc_voucher_jail</a>(addr: <b>address</b>) <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <b>let</b> buddies = <a href="vouch.md#0x1_vouch_get_buddies">vouch::get_buddies</a>(addr);
  <b>let</b> i = 0;
  <b>while</b> (i &lt; <a href="_length">vector::length</a>(&buddies)) {
    <b>let</b> voucher = *<a href="_borrow">vector::borrow</a>(&buddies, i);
    <b>if</b> (<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(voucher)) {
      <b>let</b> v = <b>borrow_global_mut</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(voucher);
      v.lifetime_vouchees_jailed = v.lifetime_vouchees_jailed + 1;
    };
    i = i + 1;
  }
}
</code></pre>



</details>

<a name="0x1_jail_get_jail_reputation"></a>

## Function `get_jail_reputation`

Returns how many times has the validator failed to join the network after consecutive attempts.
Should not abort, since its used in validator admission.
Returns (lifetime_jailed, consecutive_failure_to_rejoin)


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_get_jail_reputation">get_jail_reputation</a>(addr: <b>address</b>): (u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_get_jail_reputation">get_jail_reputation</a>(addr: <b>address</b>): (u64, u64) <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr)) {
    <b>let</b> s = <b>borrow_global</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr);
    <b>return</b> (s.lifetime_jailed, s.consecutive_failure_to_rejoin)
  };
  (0, 0)
}
</code></pre>



</details>

<a name="0x1_jail_get_count_buddies_jailed"></a>

## Function `get_count_buddies_jailed`

Returns the cumulative number of times someone a validator vouched for (vouchee) was jailed. I.e. are they picking performant validators.


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_get_count_buddies_jailed">get_count_buddies_jailed</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_get_count_buddies_jailed">get_count_buddies_jailed</a>(addr: <b>address</b>): u64 <b>acquires</b> <a href="jail.md#0x1_jail_Jail">Jail</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr)) {
    <b>return</b> <b>borrow_global</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr).lifetime_vouchees_jailed
  };
  0
}
</code></pre>



</details>

<a name="0x1_jail_exists_jail"></a>

## Function `exists_jail`



<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_exists_jail">exists_jail</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="jail.md#0x1_jail_exists_jail">exists_jail</a>(addr: <b>address</b>): bool {
  <b>exists</b>&lt;<a href="jail.md#0x1_jail_Jail">Jail</a>&gt;(addr)
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
