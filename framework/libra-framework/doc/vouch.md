
<a name="0x1_vouch"></a>

# Module `0x1::vouch`



-  [Resource `Vouch`](#0x1_vouch_Vouch)
-  [Constants](#@Constants_0)
-  [Function `init`](#0x1_vouch_init)
-  [Function `is_init`](#0x1_vouch_is_init)
-  [Function `vouch_for`](#0x1_vouch_vouch_for)
-  [Function `revoke`](#0x1_vouch_revoke)
-  [Function `vm_migrate`](#0x1_vouch_vm_migrate)
-  [Function `bulk_set`](#0x1_vouch_bulk_set)
-  [Function `get_buddies`](#0x1_vouch_get_buddies)
-  [Function `buddies_in_set`](#0x1_vouch_buddies_in_set)
-  [Function `unrelated_buddies`](#0x1_vouch_unrelated_buddies)
-  [Function `unrelated_buddies_above_thresh`](#0x1_vouch_unrelated_buddies_above_thresh)


<pre><code><b>use</b> <a href="ancestry.md#0x1_ancestry">0x1::ancestry</a>;
<b>use</b> <a href="globals.md#0x1_globals">0x1::globals</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="testnet.md#0x1_testnet">0x1::testnet</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_vouch_Vouch"></a>

## Resource `Vouch`



<pre><code><b>struct</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>my_buddies: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_vouch_ETRY_SELF_VOUCH_REALLY"></a>

Trying to vouch for yourself?


<pre><code><b>const</b> <a href="vouch.md#0x1_vouch_ETRY_SELF_VOUCH_REALLY">ETRY_SELF_VOUCH_REALLY</a>: u64 = 12345;
</code></pre>



<a name="0x1_vouch_init"></a>

## Function `init`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_init">init</a>(new_account_sig: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_init">init</a>(new_account_sig: &<a href="">signer</a> ) {
  <b>let</b> acc = <a href="_address_of">signer::address_of</a>(new_account_sig);

  <b>if</b> (!<a href="vouch.md#0x1_vouch_is_init">is_init</a>(acc)) {
    <b>move_to</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(new_account_sig, <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {
        my_buddies: <a href="_empty">vector::empty</a>(),
      });
  }
}
</code></pre>



</details>

<a name="0x1_vouch_is_init"></a>

## Function `is_init`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_is_init">is_init</a>(acc: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_is_init">is_init</a>(acc: <b>address</b> ):bool {
  <b>exists</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(acc)
}
</code></pre>



</details>

<a name="0x1_vouch_vouch_for"></a>

## Function `vouch_for`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_vouch_for">vouch_for</a>(buddy: &<a href="">signer</a>, val: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_vouch_for">vouch_for</a>(buddy: &<a href="">signer</a>, val: <b>address</b>) <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {
  <b>let</b> buddy_acc = <a href="_address_of">signer::address_of</a>(buddy);
  <b>assert</b>!(buddy_acc!=val, <a href="vouch.md#0x1_vouch_ETRY_SELF_VOUCH_REALLY">ETRY_SELF_VOUCH_REALLY</a>);

  // <b>if</b> (!<a href="validator_universe.md#0x1_validator_universe_is_in_universe">validator_universe::is_in_universe</a>(buddy_acc)) <b>return</b>;
  <b>if</b> (!<b>exists</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val)) <b>return</b>;

  <b>let</b> v = <b>borrow_global_mut</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val);
  <b>if</b> (!<a href="_contains">vector::contains</a>(&v.my_buddies, &buddy_acc)) { // prevent duplicates
    <a href="_push_back">vector::push_back</a>&lt;<b>address</b>&gt;(&<b>mut</b> v.my_buddies, buddy_acc);
  }
}
</code></pre>



</details>

<a name="0x1_vouch_revoke"></a>

## Function `revoke`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_revoke">revoke</a>(buddy: &<a href="">signer</a>, val: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_revoke">revoke</a>(buddy: &<a href="">signer</a>, val: <b>address</b>) <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {
  <b>let</b> buddy_acc = <a href="_address_of">signer::address_of</a>(buddy);
  <b>assert</b>!(buddy_acc!=val, <a href="vouch.md#0x1_vouch_ETRY_SELF_VOUCH_REALLY">ETRY_SELF_VOUCH_REALLY</a>);

  <b>if</b> (!<b>exists</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val)) <b>return</b>;

  <b>let</b> v = <b>borrow_global_mut</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val);
  <b>let</b> (found, i) = <a href="_index_of">vector::index_of</a>(&v.my_buddies, &buddy_acc);
  <b>if</b> (found) {
    <a href="_remove">vector::remove</a>(&<b>mut</b> v.my_buddies, i);
  };
}
</code></pre>



</details>

<a name="0x1_vouch_vm_migrate"></a>

## Function `vm_migrate`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_vm_migrate">vm_migrate</a>(vm: &<a href="">signer</a>, val: <b>address</b>, buddy_list: <a href="">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_vm_migrate">vm_migrate</a>(vm: &<a href="">signer</a>, val: <b>address</b>, buddy_list: <a href="">vector</a>&lt;<b>address</b>&gt;) <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(vm);
  <a href="vouch.md#0x1_vouch_bulk_set">bulk_set</a>(val, buddy_list);

}
</code></pre>



</details>

<a name="0x1_vouch_bulk_set"></a>

## Function `bulk_set`



<pre><code><b>fun</b> <a href="vouch.md#0x1_vouch_bulk_set">bulk_set</a>(val: <b>address</b>, buddy_list: <a href="">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="vouch.md#0x1_vouch_bulk_set">bulk_set</a>(val: <b>address</b>, buddy_list: <a href="">vector</a>&lt;<b>address</b>&gt;) <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {

  // <b>if</b> (!<a href="validator_universe.md#0x1_validator_universe_is_in_universe">validator_universe::is_in_universe</a>(val)) <b>return</b>;
  <b>if</b> (!<b>exists</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val)) <b>return</b>;

  <b>let</b> v = <b>borrow_global_mut</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val);

  // take self out of list
  <b>let</b> (is_found, i) = <a href="_index_of">vector::index_of</a>(&buddy_list, &val);

  <b>if</b> (is_found) {
    <a href="_swap_remove">vector::swap_remove</a>&lt;<b>address</b>&gt;(&<b>mut</b> buddy_list, i);
  };

  v.my_buddies = buddy_list;
}
</code></pre>



</details>

<a name="0x1_vouch_get_buddies"></a>

## Function `get_buddies`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_get_buddies">get_buddies</a>(val: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_get_buddies">get_buddies</a>(val: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a>{
  <b>if</b> (<a href="vouch.md#0x1_vouch_is_init">is_init</a>(val)) {
    <b>return</b> *&<b>borrow_global</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val).my_buddies
  };
  <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;()
}
</code></pre>



</details>

<a name="0x1_vouch_buddies_in_set"></a>

## Function `buddies_in_set`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_buddies_in_set">buddies_in_set</a>(val: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_buddies_in_set">buddies_in_set</a>(val: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {
  <b>let</b> current_set = <a href="stake.md#0x1_stake_get_current_validators">stake::get_current_validators</a>();
  <b>if</b> (!<b>exists</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val)) <b>return</b> <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();

  <b>let</b> v = <b>borrow_global</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val);

  <b>let</b> buddies_in_set = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();
  <b>let</b>  i = 0;
  <b>while</b> (i &lt; <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&v.my_buddies)) {
    <b>let</b> a = <a href="_borrow">vector::borrow</a>&lt;<b>address</b>&gt;(&v.my_buddies, i);
    <b>if</b> (<a href="_contains">vector::contains</a>(&current_set, a)) {
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> buddies_in_set, *a);
    };
    i = i + 1;
  };

  buddies_in_set
}
</code></pre>



</details>

<a name="0x1_vouch_unrelated_buddies"></a>

## Function `unrelated_buddies`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_unrelated_buddies">unrelated_buddies</a>(val: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_unrelated_buddies">unrelated_buddies</a>(val: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a> {
  // start our list empty
  <b>let</b> unrelated_buddies = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();

  // find all our buddies in this validator set
  <b>let</b> buddies_in_set = <a href="vouch.md#0x1_vouch_buddies_in_set">buddies_in_set</a>(val);
  <b>let</b> len = <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&buddies_in_set);
  <b>let</b>  i = 0;
  <b>while</b> (i &lt; len) {

    <b>let</b> target_acc = <a href="_borrow">vector::borrow</a>&lt;<b>address</b>&gt;(&buddies_in_set, i);

    // now <b>loop</b> through all the accounts again, and check <b>if</b> this target
    // <a href="account.md#0x1_account">account</a> is related <b>to</b> anyone.
    <b>let</b>  k = 0;
    <b>while</b> (k &lt; <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&buddies_in_set)) {
      <b>let</b> comparison_acc = <a href="_borrow">vector::borrow</a>(&buddies_in_set, k);
      // skip <b>if</b> you're the same person
      <b>if</b> (comparison_acc != target_acc) {
        // check <a href="ancestry.md#0x1_ancestry">ancestry</a> algo
        <b>let</b> (is_fam, _) = <a href="ancestry.md#0x1_ancestry_is_family">ancestry::is_family</a>(*comparison_acc, *target_acc);
        <b>if</b> (!is_fam) {
          <b>if</b> (!<a href="_contains">vector::contains</a>(&unrelated_buddies, target_acc)) {
            <a href="_push_back">vector::push_back</a>&lt;<b>address</b>&gt;(&<b>mut</b> unrelated_buddies, *target_acc)
          }
        }
      };
      k = k + 1;
    };

    // <b>if</b> (<a href="_contains">vector::contains</a>(&current_set, a)) {
    //   <a href="_push_back">vector::push_back</a>(&<b>mut</b> buddies_in_set, *a);
    // };
    i = i + 1;
  };

  unrelated_buddies
}
</code></pre>



</details>

<a name="0x1_vouch_unrelated_buddies_above_thresh"></a>

## Function `unrelated_buddies_above_thresh`



<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_unrelated_buddies_above_thresh">unrelated_buddies_above_thresh</a>(val: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vouch.md#0x1_vouch_unrelated_buddies_above_thresh">unrelated_buddies_above_thresh</a>(val: <b>address</b>): bool <b>acquires</b> <a href="vouch.md#0x1_vouch_Vouch">Vouch</a>{
  <b>if</b> (!<b>exists</b>&lt;<a href="vouch.md#0x1_vouch_Vouch">Vouch</a>&gt;(val)) <b>return</b> <b>false</b>;

  <b>if</b> (<a href="testnet.md#0x1_testnet_is_testnet">testnet::is_testnet</a>()) <b>return</b> <b>true</b>;

  <b>let</b> len = <a href="_length">vector::length</a>(&<a href="vouch.md#0x1_vouch_unrelated_buddies">unrelated_buddies</a>(val));
  (len &gt;= <a href="globals.md#0x1_globals_get_vouch_threshold">globals::get_vouch_threshold</a>())
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
