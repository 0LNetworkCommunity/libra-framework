
<a name="0x1_epoch_boundary"></a>

# Module `0x1::epoch_boundary`



-  [Function `epoch_boundary`](#0x1_epoch_boundary_epoch_boundary)


<pre><code><b>use</b> <a href="musical_chairs.md#0x1_musical_chairs">0x1::musical_chairs</a>;
<b>use</b> <a href="proof_of_fee.md#0x1_proof_of_fee">0x1::proof_of_fee</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="slow_wallet.md#0x1_slow_wallet">0x1::slow_wallet</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
</code></pre>



<a name="0x1_epoch_boundary_epoch_boundary"></a>

## Function `epoch_boundary`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="epoch_boundary.md#0x1_epoch_boundary">epoch_boundary</a>(root: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="epoch_boundary.md#0x1_epoch_boundary">epoch_boundary</a>(root: &<a href="">signer</a>) {
    <b>if</b> (<a href="_address_of">signer::address_of</a>(root) != @ol_framework) {
        <b>return</b>
    };

    // TODO: this needs <b>to</b> be a <b>friend</b> function, but it's in a different namespace, so we are gating it <b>with</b> vm <a href="">signer</a>, which is what was done previously. Which means hacking <a href="block.md#0x1_block">block</a>.<b>move</b>
    <a href="slow_wallet.md#0x1_slow_wallet_on_new_epoch">slow_wallet::on_new_epoch</a>(root);

    <b>let</b> (compliant, n_seats) = <a href="musical_chairs.md#0x1_musical_chairs_stop_the_music">musical_chairs::stop_the_music</a>(root);

    <b>let</b> validators = <a href="proof_of_fee.md#0x1_proof_of_fee_end_epoch">proof_of_fee::end_epoch</a>(root, &compliant, n_seats);

    <a href="stake.md#0x1_stake_ol_on_new_epoch">stake::ol_on_new_epoch</a>(root, validators);

}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
