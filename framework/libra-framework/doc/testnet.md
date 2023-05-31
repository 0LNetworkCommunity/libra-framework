
<a name="0x1_testnet"></a>

# Module `0x1::testnet`



-  [Constants](#@Constants_0)
-  [Function `is_testnet`](#0x1_testnet_is_testnet)
-  [Function `assert_testnet`](#0x1_testnet_assert_testnet)
-  [Function `is_staging_net`](#0x1_testnet_is_staging_net)


<pre><code><b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::signer</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_testnet_ENOT_TESTNET"></a>



<pre><code><b>const</b> <a href="testnet.md#0x1_testnet_ENOT_TESTNET">ENOT_TESTNET</a>: u64 = 666;
</code></pre>



<a name="0x1_testnet_EWHY_U_NO_ROOT"></a>



<pre><code><b>const</b> <a href="testnet.md#0x1_testnet_EWHY_U_NO_ROOT">EWHY_U_NO_ROOT</a>: u64 = 667;
</code></pre>



<a name="0x1_testnet_is_testnet"></a>

## Function `is_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="testnet.md#0x1_testnet_is_testnet">is_testnet</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="testnet.md#0x1_testnet_is_testnet">is_testnet</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == 4
}
</code></pre>



</details>

<a name="0x1_testnet_assert_testnet"></a>

## Function `assert_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="testnet.md#0x1_testnet_assert_testnet">assert_testnet</a>(vm: &<a href="">signer</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="testnet.md#0x1_testnet_assert_testnet">assert_testnet</a>(vm: &<a href="">signer</a>): bool {
  <b>assert</b>!(
      <a href="_address_of">signer::address_of</a>(vm) == @ol_framework,
      <a href="_permission_denied">error::permission_denied</a>(<a href="testnet.md#0x1_testnet_EWHY_U_NO_ROOT">EWHY_U_NO_ROOT</a>)
  );
  <b>assert</b>!(<a href="testnet.md#0x1_testnet_is_testnet">is_testnet</a>(), <a href="_invalid_state">error::invalid_state</a>(<a href="testnet.md#0x1_testnet_ENOT_TESTNET">ENOT_TESTNET</a>));
  <b>true</b>
}
</code></pre>



</details>

<a name="0x1_testnet_is_staging_net"></a>

## Function `is_staging_net`



<pre><code><b>public</b> <b>fun</b> <a href="testnet.md#0x1_testnet_is_staging_net">is_staging_net</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="testnet.md#0x1_testnet_is_staging_net">is_staging_net</a>(): bool {
    <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == 2
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
