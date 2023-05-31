
<a name="0x1_dummy"></a>

# Module `0x1::dummy`



-  [Function `use_fn_from_aptos_framework`](#0x1_dummy_use_fn_from_aptos_framework)
-  [Function `use_fn_from_aptos_std`](#0x1_dummy_use_fn_from_aptos_std)
-  [Function `from_bytes`](#0x1_dummy_from_bytes)
-  [Function `address_from_bytes`](#0x1_dummy_address_from_bytes)


<pre><code><b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="">0x1::ed25519</a>;
</code></pre>



<a name="0x1_dummy_use_fn_from_aptos_framework"></a>

## Function `use_fn_from_aptos_framework`



<pre><code><b>public</b> entry <b>fun</b> <a href="dummy.md#0x1_dummy_use_fn_from_aptos_framework">use_fn_from_aptos_framework</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dummy.md#0x1_dummy_use_fn_from_aptos_framework">use_fn_from_aptos_framework</a>() {
    <b>let</b> _chain_id = <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>();
}
</code></pre>



</details>

<a name="0x1_dummy_use_fn_from_aptos_std"></a>

## Function `use_fn_from_aptos_std`



<pre><code><b>public</b> entry <b>fun</b> <a href="dummy.md#0x1_dummy_use_fn_from_aptos_std">use_fn_from_aptos_std</a>(account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="dummy.md#0x1_dummy_use_fn_from_aptos_std">use_fn_from_aptos_std</a>(
    account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;
) {
    <b>let</b> _pubkey = <a href="_new_unvalidated_public_key_from_bytes">ed25519::new_unvalidated_public_key_from_bytes</a>(account_public_key_bytes);
}
</code></pre>



</details>

<a name="0x1_dummy_from_bytes"></a>

## Function `from_bytes`

Native function to deserialize a type T.

Note that this function does not put any constraint on <code>T</code>. If code uses this function to
deserialized a linear value, its their responsibility that the data they deserialize is
owned.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="dummy.md#0x1_dummy_from_bytes">from_bytes</a>&lt;T&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>native</b> <b>fun</b> <a href="dummy.md#0x1_dummy_from_bytes">from_bytes</a>&lt;T&gt;(bytes: <a href="">vector</a>&lt;u8&gt;): T;
</code></pre>



</details>

<a name="0x1_dummy_address_from_bytes"></a>

## Function `address_from_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="dummy.md#0x1_dummy_address_from_bytes">address_from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="dummy.md#0x1_dummy_address_from_bytes">address_from_bytes</a>(bytes: <a href="">vector</a>&lt;u8&gt;): <b>address</b> {
    <a href="dummy.md#0x1_dummy_from_bytes">from_bytes</a>(bytes)
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
