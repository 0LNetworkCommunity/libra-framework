
<a name="0x1_slow_wallet"></a>

# Module `0x1::slow_wallet`



-  [Resource `SlowWallet`](#0x1_slow_wallet_SlowWallet)
-  [Resource `SlowWalletList`](#0x1_slow_wallet_SlowWalletList)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_slow_wallet_initialize)
-  [Function `fork_migrate_slow_wallet`](#0x1_slow_wallet_fork_migrate_slow_wallet)
-  [Function `fork_migrate_slow_list`](#0x1_slow_wallet_fork_migrate_slow_list)
-  [Function `set_slow`](#0x1_slow_wallet_set_slow)
-  [Function `slow_wallet_epoch_drip`](#0x1_slow_wallet_slow_wallet_epoch_drip)
-  [Function `decrease_unlocked_tracker`](#0x1_slow_wallet_decrease_unlocked_tracker)
-  [Function `increase_unlocked_tracker`](#0x1_slow_wallet_increase_unlocked_tracker)
-  [Function `is_slow`](#0x1_slow_wallet_is_slow)
-  [Function `unlocked_amount`](#0x1_slow_wallet_unlocked_amount)
-  [Function `get_slow_list`](#0x1_slow_wallet_get_slow_list)
-  [Function `on_new_epoch`](#0x1_slow_wallet_on_new_epoch)
-  [Function `vm_multi_pay_fee`](#0x1_slow_wallet_vm_multi_pay_fee)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="globals.md#0x1_globals">0x1::globals</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_slow_wallet_SlowWallet"></a>

## Resource `SlowWallet`



<pre><code><b>struct</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>unlocked: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>transferred: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_slow_wallet_SlowWalletList"></a>

## Resource `SlowWalletList`



<pre><code><b>struct</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>list: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_slow_wallet_EGENESIS_ERROR"></a>



<pre><code><b>const</b> <a href="slow_wallet.md#0x1_slow_wallet_EGENESIS_ERROR">EGENESIS_ERROR</a>: u64 = 10001;
</code></pre>



<a name="0x1_slow_wallet_EPOCH_DRIP_CONST"></a>



<pre><code><b>const</b> <a href="slow_wallet.md#0x1_slow_wallet_EPOCH_DRIP_CONST">EPOCH_DRIP_CONST</a>: u64 = 100000;
</code></pre>



<a name="0x1_slow_wallet_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_initialize">initialize</a>(vm: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_initialize">initialize</a>(vm: &<a href="">signer</a>){
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <b>if</b> (!<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework)) {
    <b>move_to</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(vm, <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a> {
      list: <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;()
    });
  }
}
</code></pre>



</details>

<a name="0x1_slow_wallet_fork_migrate_slow_wallet"></a>

## Function `fork_migrate_slow_wallet`

private function which can only be called at genesis
must apply the coin split factor.


<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_fork_migrate_slow_wallet">fork_migrate_slow_wallet</a>(vm: &<a href="">signer</a>, user: &<a href="">signer</a>, unlocked: u64, transferred: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_fork_migrate_slow_wallet">fork_migrate_slow_wallet</a>(
  vm: &<a href="">signer</a>,
  user: &<a href="">signer</a>,
  unlocked: u64,
  transferred: u64,
) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <b>if</b> (!<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(<a href="_address_of">signer::address_of</a>(user))) {
    <b>move_to</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(vm, <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a> {
      unlocked: unlocked * <a href="globals.md#0x1_globals_get_coin_split_factor">globals::get_coin_split_factor</a>(),
      transferred: transferred * <a href="globals.md#0x1_globals_get_coin_split_factor">globals::get_coin_split_factor</a>(),
    });

    <a href="slow_wallet.md#0x1_slow_wallet_fork_migrate_slow_list">fork_migrate_slow_list</a>(vm, user);
  }
}
</code></pre>



</details>

<a name="0x1_slow_wallet_fork_migrate_slow_list"></a>

## Function `fork_migrate_slow_list`

private function which can only be called at genesis
sets the list of accounts that are slow wallets.


<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_fork_migrate_slow_list">fork_migrate_slow_list</a>(vm: &<a href="">signer</a>, user: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_fork_migrate_slow_list">fork_migrate_slow_list</a>(
  vm: &<a href="">signer</a>,
  user: &<a href="">signer</a>,
) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>{
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <b>if</b> (!<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework)) {
    <a href="slow_wallet.md#0x1_slow_wallet_initialize">initialize</a>(vm); //don't <b>abort</b>
  };
  <b>let</b> list = <b>borrow_global_mut</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework);
  <a href="_push_back">vector::push_back</a>(&<b>mut</b> list.list, <a href="_address_of">signer::address_of</a>(user));
}
</code></pre>



</details>

<a name="0x1_slow_wallet_set_slow"></a>

## Function `set_slow`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_set_slow">set_slow</a>(sig: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_set_slow">set_slow</a>(sig: &<a href="">signer</a>) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a> {
  <b>assert</b>!(<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework), <a href="_invalid_argument">error::invalid_argument</a>(<a href="slow_wallet.md#0x1_slow_wallet_EGENESIS_ERROR">EGENESIS_ERROR</a>));

    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(sig);
    <b>let</b> list = <a href="slow_wallet.md#0x1_slow_wallet_get_slow_list">get_slow_list</a>();
    <b>if</b> (!<a href="_contains">vector::contains</a>&lt;<b>address</b>&gt;(&list, &addr)) {
        <b>let</b> s = <b>borrow_global_mut</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework);
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> s.list, addr);
    };

    <b>if</b> (!<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(<a href="_address_of">signer::address_of</a>(sig))) {
      <b>move_to</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(sig, <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a> {
        unlocked: 0,
        transferred: 0,
      });
    }
}
</code></pre>



</details>

<a name="0x1_slow_wallet_slow_wallet_epoch_drip"></a>

## Function `slow_wallet_epoch_drip`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_slow_wallet_epoch_drip">slow_wallet_epoch_drip</a>(vm: &<a href="">signer</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_slow_wallet_epoch_drip">slow_wallet_epoch_drip</a>(vm: &<a href="">signer</a>, amount: u64) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>, <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>{
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <b>let</b> list = <a href="slow_wallet.md#0x1_slow_wallet_get_slow_list">get_slow_list</a>();
  <b>let</b> i = 0;
  <b>while</b> (i &lt; <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&list)) {
    <b>let</b> addr = <a href="_borrow">vector::borrow</a>&lt;<b>address</b>&gt;(&list, i);
    <b>let</b> s = <b>borrow_global_mut</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(*addr);
    s.unlocked = s.unlocked + amount;
    i = i + 1;
  }
}
</code></pre>



</details>

<a name="0x1_slow_wallet_decrease_unlocked_tracker"></a>

## Function `decrease_unlocked_tracker`



<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_decrease_unlocked_tracker">decrease_unlocked_tracker</a>(payer: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_decrease_unlocked_tracker">decrease_unlocked_tracker</a>(payer: <b>address</b>, amount: u64) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a> {
  <b>let</b> s = <b>borrow_global_mut</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(payer);
  s.transferred = s.transferred + amount;
  s.unlocked = s.unlocked - amount;
}
</code></pre>



</details>

<a name="0x1_slow_wallet_increase_unlocked_tracker"></a>

## Function `increase_unlocked_tracker`



<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_increase_unlocked_tracker">increase_unlocked_tracker</a>(recipient: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_increase_unlocked_tracker">increase_unlocked_tracker</a>(recipient: <b>address</b>, amount: u64) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a> {
  <b>let</b> s = <b>borrow_global_mut</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(recipient);
  s.unlocked = s.unlocked + amount;
}
</code></pre>



</details>

<a name="0x1_slow_wallet_is_slow"></a>

## Function `is_slow`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_is_slow">is_slow</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_is_slow">is_slow</a>(addr: <b>address</b>): bool {
  <b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_slow_wallet_unlocked_amount"></a>

## Function `unlocked_amount`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_unlocked_amount">unlocked_amount</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_unlocked_amount">unlocked_amount</a>(addr: <b>address</b>): u64 <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>{
  <b>if</b> (<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(addr)) {
    <b>let</b> s = <b>borrow_global</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>&gt;(addr);
    <b>return</b> s.unlocked
  };
  // this is a normal <a href="account.md#0x1_account">account</a>, so <b>return</b> the normal balance
  <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;GasCoin&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_slow_wallet_get_slow_list"></a>

## Function `get_slow_list`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_get_slow_list">get_slow_list</a>(): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_get_slow_list">get_slow_list</a>(): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>{
  <b>if</b> (<b>exists</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework)) {
    <b>let</b> s = <b>borrow_global</b>&lt;<a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a>&gt;(@ol_framework);
    <b>return</b> *&s.list
  } <b>else</b> {
    <b>return</b> <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;()
  }
}
</code></pre>



</details>

<a name="0x1_slow_wallet_on_new_epoch"></a>

## Function `on_new_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_on_new_epoch">on_new_epoch</a>(vm: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_on_new_epoch">on_new_epoch</a>(vm: &<a href="">signer</a>) <b>acquires</b> <a href="slow_wallet.md#0x1_slow_wallet_SlowWallet">SlowWallet</a>, <a href="slow_wallet.md#0x1_slow_wallet_SlowWalletList">SlowWalletList</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <a href="slow_wallet.md#0x1_slow_wallet_slow_wallet_epoch_drip">slow_wallet_epoch_drip</a>(vm, <a href="slow_wallet.md#0x1_slow_wallet_EPOCH_DRIP_CONST">EPOCH_DRIP_CONST</a>);
}
</code></pre>



</details>

<a name="0x1_slow_wallet_vm_multi_pay_fee"></a>

## Function `vm_multi_pay_fee`



<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_vm_multi_pay_fee">vm_multi_pay_fee</a>(_vm: &<a href="">signer</a>, _list: &<a href="">vector</a>&lt;<b>address</b>&gt;, _price: u64, _metadata: &<a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="slow_wallet.md#0x1_slow_wallet_vm_multi_pay_fee">vm_multi_pay_fee</a>(_vm: &<a href="">signer</a>, _list: &<a href="">vector</a>&lt;<b>address</b>&gt;, _price: u64, _metadata: &<a href="">vector</a>&lt;u8&gt;) {

}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
