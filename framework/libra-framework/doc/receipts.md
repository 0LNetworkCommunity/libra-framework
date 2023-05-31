
<a name="0x1_receipts"></a>

# Module `0x1::receipts`



-  [Resource `UserReceipts`](#0x1_receipts_UserReceipts)
-  [Function `init`](#0x1_receipts_init)
-  [Function `fork_migrate`](#0x1_receipts_fork_migrate)
-  [Function `is_init`](#0x1_receipts_is_init)
-  [Function `write_receipt_vm`](#0x1_receipts_write_receipt_vm)
-  [Function `write_receipt`](#0x1_receipts_write_receipt)
-  [Function `read_receipt`](#0x1_receipts_read_receipt)


<pre><code><b>use</b> <a href="globals.md#0x1_globals">0x1::globals</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_receipts_UserReceipts"></a>

## Resource `UserReceipts`



<pre><code><b>struct</b> <a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>destination: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>cumulative: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>last_payment_timestamp: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>last_payment_value: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_receipts_init"></a>

## Function `init`



<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_init">init</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_init">init</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>) {
  <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
  <b>if</b> (!<b>exists</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(addr)) {
    <b>move_to</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(
      <a href="account.md#0x1_account">account</a>,
      <a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a> {
        destination: <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;(),
        last_payment_timestamp: <a href="_empty">vector::empty</a>&lt;u64&gt;(),
        last_payment_value: <a href="_empty">vector::empty</a>&lt;u64&gt;(),
        cumulative: <a href="_empty">vector::empty</a>&lt;u64&gt;(),
      }
    )
  };
}
</code></pre>



</details>

<a name="0x1_receipts_fork_migrate"></a>

## Function `fork_migrate`



<pre><code><b>fun</b> <a href="receipts.md#0x1_receipts_fork_migrate">fork_migrate</a>(vm: &<a href="">signer</a>, <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, destination: <b>address</b>, cumulative: u64, last_payment_timestamp: u64, last_payment_value: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="receipts.md#0x1_receipts_fork_migrate">fork_migrate</a>(
  vm: &<a href="">signer</a>,
  <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>,
  destination: <b>address</b>,
  cumulative: u64,
  last_payment_timestamp: u64,
  last_payment_value: u64,
) <b>acquires</b> <a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a> {

  <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(vm);
  <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
  <b>assert</b>!(<a href="receipts.md#0x1_receipts_is_init">is_init</a>(addr), 0);
  <b>let</b> state = <b>borrow_global_mut</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(addr);
  <a href="_push_back">vector::push_back</a>(&<b>mut</b> state.destination, destination);
  <a href="_push_back">vector::push_back</a>(
    &<b>mut</b> state.cumulative,
    cumulative * <a href="globals.md#0x1_globals_get_coin_split_factor">globals::get_coin_split_factor</a>()
  );
  <a href="_push_back">vector::push_back</a>(&<b>mut</b> state.last_payment_timestamp, last_payment_timestamp);
  <a href="_push_back">vector::push_back</a>(
    &<b>mut</b> state.last_payment_value,
    last_payment_value * <a href="globals.md#0x1_globals_get_coin_split_factor">globals::get_coin_split_factor</a>()
  );
}
</code></pre>



</details>

<a name="0x1_receipts_is_init"></a>

## Function `is_init`



<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_is_init">is_init</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_is_init">is_init</a>(addr: <b>address</b>):bool {
  <b>exists</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_receipts_write_receipt_vm"></a>

## Function `write_receipt_vm`



<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_write_receipt_vm">write_receipt_vm</a>(sender: &<a href="">signer</a>, payer: <b>address</b>, destination: <b>address</b>, value: u64): (u64, u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_write_receipt_vm">write_receipt_vm</a>(
  sender: &<a href="">signer</a>,
  payer: <b>address</b>,
  destination: <b>address</b>,
  value: u64
):(u64, u64, u64) <b>acquires</b> <a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a> {
    // TODO: make a function for user <b>to</b> write own receipt.
    <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(sender);
    <a href="receipts.md#0x1_receipts_write_receipt">write_receipt</a>(payer, destination, value)
}
</code></pre>



</details>

<a name="0x1_receipts_write_receipt"></a>

## Function `write_receipt`

Restricted to DiemAccount, we need to write receipts for certain users,
like to DonorDirected Accounts.
Core Devs: Danger: only DiemAccount can use this.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="receipts.md#0x1_receipts_write_receipt">write_receipt</a>(payer: <b>address</b>, destination: <b>address</b>, value: u64): (u64, u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="receipts.md#0x1_receipts_write_receipt">write_receipt</a>(
  payer: <b>address</b>,
  destination: <b>address</b>,
  value: u64
):(u64, u64, u64) <b>acquires</b> <a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a> {
    // TODO: make a function for user <b>to</b> write own receipt.
    <b>if</b> (!<b>exists</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(payer)) {
      <b>return</b> (0, 0, 0)
    };

    <b>let</b> r = <b>borrow_global_mut</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(payer);
    <b>let</b> (found_it, i) = <a href="_index_of">vector::index_of</a>(&r.destination, &destination);

    <b>let</b> cumu = 0;
    <b>if</b> (found_it) {
      cumu = *<a href="_borrow">vector::borrow</a>&lt;u64&gt;(&r.cumulative, i);
    };
    cumu = cumu + value;
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> r.cumulative, *&cumu);

    <b>let</b> <a href="timestamp.md#0x1_timestamp">timestamp</a> = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> r.last_payment_timestamp, *&<a href="timestamp.md#0x1_timestamp">timestamp</a>);
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> r.last_payment_value, *&value);

    <b>if</b> (found_it) { // put in same index <b>if</b> the <a href="account.md#0x1_account">account</a> was already there.
      <a href="_swap_remove">vector::swap_remove</a>(&<b>mut</b> r.last_payment_timestamp, i);
      <a href="_swap_remove">vector::swap_remove</a>(&<b>mut</b> r.last_payment_value, i);
      <a href="_swap_remove">vector::swap_remove</a>(&<b>mut</b> r.cumulative, i);
    } <b>else</b> {
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> r.destination, destination);
    };

    (<a href="timestamp.md#0x1_timestamp">timestamp</a>, value, cumu)
}
</code></pre>



</details>

<a name="0x1_receipts_read_receipt"></a>

## Function `read_receipt`



<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_read_receipt">read_receipt</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>, destination: <b>address</b>): (u64, u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="receipts.md#0x1_receipts_read_receipt">read_receipt</a>(
  <a href="account.md#0x1_account">account</a>: <b>address</b>,
  destination: <b>address</b>
):(u64, u64, u64) <b>acquires</b> <a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a> {
  <b>if</b> (!<b>exists</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(<a href="account.md#0x1_account">account</a>)) {
    <b>return</b> (0, 0, 0)
  };

  <b>let</b> receipt = <b>borrow_global</b>&lt;<a href="receipts.md#0x1_receipts_UserReceipts">UserReceipts</a>&gt;(<a href="account.md#0x1_account">account</a>);
  <b>let</b> (found_it, i) = <a href="_index_of">vector::index_of</a>(&receipt.destination, &destination);
  <b>if</b> (!found_it) <b>return</b> (0, 0, 0);

  <b>let</b> time = <a href="_borrow">vector::borrow</a>&lt;u64&gt;(&receipt.last_payment_timestamp, i);
  <b>let</b> value = <a href="_borrow">vector::borrow</a>&lt;u64&gt;(&receipt.last_payment_value, i);
  <b>let</b> cumu = <a href="_borrow">vector::borrow</a>&lt;u64&gt;(&receipt.cumulative, i);

  (*time, *value, *cumu)
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
