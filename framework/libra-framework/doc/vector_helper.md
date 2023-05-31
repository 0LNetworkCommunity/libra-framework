
<a name="0x1_vector_helper"></a>

# Module `0x1::vector_helper`



-  [Function `compare`](#0x1_vector_helper_compare)


<pre><code></code></pre>



<a name="0x1_vector_helper_compare"></a>

## Function `compare`



<pre><code><b>public</b> <b>fun</b> <a href="vector_helper.md#0x1_vector_helper_compare">compare</a>&lt;Element&gt;(a: &<a href="">vector</a>&lt;Element&gt;, b: &<a href="">vector</a>&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="vector_helper.md#0x1_vector_helper_compare">compare</a>&lt;Element&gt;(a: &<a href="">vector</a>&lt;Element&gt;, b: &<a href="">vector</a>&lt;Element&gt;): bool {
    <b>let</b> i = 0;
    <b>let</b> len_a = length(a);
    <b>let</b> len_b = length(b);
    <b>if</b> (len_a != len_b) { <b>return</b> <b>false</b> };
    <b>while</b> (i &lt; len_a) {
        <b>let</b> num_a = borrow(a, i);
        <b>let</b> num_b = borrow(b, i);
        <b>if</b> (num_a == num_b) {
            i = i + 1;
        } <b>else</b> {
            <b>return</b> <b>false</b>
        }
    };
    <b>true</b>
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
