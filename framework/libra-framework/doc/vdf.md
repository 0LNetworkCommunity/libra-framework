
<a name="0x1_vdf"></a>

# Module `0x1::vdf`



-  [Function `verify`](#0x1_vdf_verify)
-  [Function `extract_address_from_challenge`](#0x1_vdf_extract_address_from_challenge)


<pre><code></code></pre>



<a name="0x1_vdf_verify"></a>

## Function `verify`



<pre><code><b>public</b> <b>fun</b> <a href="vdf.md#0x1_vdf_verify">verify</a>(challenge: &<a href="">vector</a>&lt;u8&gt;, solution: &<a href="">vector</a>&lt;u8&gt;, difficulty: &u64, security: &u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="vdf.md#0x1_vdf_verify">verify</a>(
  challenge: &<a href="">vector</a>&lt;u8&gt;,
  solution: &<a href="">vector</a>&lt;u8&gt;,
  difficulty: &u64,
  security: &u64,
): bool;
</code></pre>



</details>

<a name="0x1_vdf_extract_address_from_challenge"></a>

## Function `extract_address_from_challenge`



<pre><code><b>public</b> <b>fun</b> <a href="vdf.md#0x1_vdf_extract_address_from_challenge">extract_address_from_challenge</a>(challenge: &<a href="">vector</a>&lt;u8&gt;): (<b>address</b>, <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="vdf.md#0x1_vdf_extract_address_from_challenge">extract_address_from_challenge</a>(
  challenge: &<a href="">vector</a>&lt;u8&gt;
): (<b>address</b>, <a href="">vector</a>&lt;u8&gt;);
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
