
<a name="0x1_persistence_demo"></a>

# Module `0x1::persistence_demo`



-  [Resource `State`](#0x1_persistence_demo_State)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_persistence_demo_initialize)
-  [Function `add_stuff`](#0x1_persistence_demo_add_stuff)
-  [Function `remove_stuff`](#0x1_persistence_demo_remove_stuff)
-  [Function `isEmpty`](#0x1_persistence_demo_isEmpty)
-  [Function `length`](#0x1_persistence_demo_length)
-  [Function `contains`](#0x1_persistence_demo_contains)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="testnet.md#0x1_testnet">0x1::testnet</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_persistence_demo_State"></a>

## Resource `State`



<pre><code><b>struct</b> <a href="persistence_demo.md#0x1_persistence_demo_State">State</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>hist: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_persistence_demo_ETESTNET"></a>



<pre><code><b>const</b> <a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>: u64 = 4001;
</code></pre>



<a name="0x1_persistence_demo_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_initialize">initialize</a>(sender: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_initialize">initialize</a>(sender: &<a href="">signer</a>){
  // `<b>assert</b> can be used <b>to</b> evaluate a bool and exit the program <b>with</b>
  // an <a href="">error</a> <a href="code.md#0x1_code">code</a>, e.g. testing <b>if</b> this is being run in <a href="testnet.md#0x1_testnet">testnet</a>, and
  // throwing <a href="">error</a> 01.
  <b>assert</b>!(is_testnet(), <a href="_invalid_state">error::invalid_state</a>(<a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>));
  // In the actual <b>module</b>, must <b>assert</b> that this is the sender is the association
  <b>move_to</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(sender, <a href="persistence_demo.md#0x1_persistence_demo_State">State</a>{ hist: <a href="_empty">vector::empty</a>() });
}
</code></pre>



</details>

<a name="0x1_persistence_demo_add_stuff"></a>

## Function `add_stuff`



<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_add_stuff">add_stuff</a>(sender: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_add_stuff">add_stuff</a>(sender: &<a href="">signer</a>) <b>acquires</b> <a href="persistence_demo.md#0x1_persistence_demo_State">State</a> {
  <b>assert</b>!(is_testnet(), <a href="_invalid_state">error::invalid_state</a>(<a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>));

  // Resource Struct state is always "borrowed" and "moved" and generally
  // cannot be copied. A <b>struct</b> can be mutably borrowed, <b>if</b> it is written <b>to</b>,
  // using `<b>borrow_global_mut</b>`. Note the Type <a href="persistence_demo.md#0x1_persistence_demo_State">State</a>
  <b>let</b> st = <b>borrow_global_mut</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(<a href="_address_of">signer::address_of</a>(sender));
  // the `&` <b>as</b> in Rust makes the assignment <b>to</b> a borrowed value. Each
  // <a href="">vector</a> operation below <b>with</b> <b>use</b> a st.hist and <b>return</b> it before the
  // next one can execute.
  <b>let</b> s = &<b>mut</b> st.hist;

  // Move <b>has</b> very limited data types. <a href="">vector</a> is the most sophisticated
  // and resembles a simplified Rust <a href="">vector</a>. Can be thought of <b>as</b> an array
  // of a single type.
  <a href="_push_back">vector::push_back</a>(s, 1);
  <a href="_push_back">vector::push_back</a>(s, 2);
  <a href="_push_back">vector::push_back</a>(s, 3);
}
</code></pre>



</details>

<a name="0x1_persistence_demo_remove_stuff"></a>

## Function `remove_stuff`



<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_remove_stuff">remove_stuff</a>(sender: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_remove_stuff">remove_stuff</a>(sender: &<a href="">signer</a>) <b>acquires</b> <a href="persistence_demo.md#0x1_persistence_demo_State">State</a>{
  <b>assert</b>!(is_testnet(), <a href="_invalid_state">error::invalid_state</a>(<a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>));
  <b>let</b> st = <b>borrow_global_mut</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(<a href="_address_of">signer::address_of</a>(sender));
  <b>let</b> s = &<b>mut</b> st.hist;

  <a href="_pop_back">vector::pop_back</a>&lt;u8&gt;(s);
  <a href="_pop_back">vector::pop_back</a>&lt;u8&gt;(s);
  <a href="_remove">vector::remove</a>&lt;u8&gt;(s, 0);
}
</code></pre>



</details>

<a name="0x1_persistence_demo_isEmpty"></a>

## Function `isEmpty`



<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_isEmpty">isEmpty</a>(sender: &<a href="">signer</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_isEmpty">isEmpty</a>(sender: &<a href="">signer</a>): bool <b>acquires</b> <a href="persistence_demo.md#0x1_persistence_demo_State">State</a> {
  <b>assert</b>!(is_testnet(), <a href="_invalid_state">error::invalid_state</a>(<a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>));

  // Note this is not a mutable borrow. Read only.
  <b>let</b> st = <b>borrow_global</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(<a href="_address_of">signer::address_of</a>(sender));
  <a href="_is_empty">vector::is_empty</a>(&st.hist)
}
</code></pre>



</details>

<a name="0x1_persistence_demo_length"></a>

## Function `length`



<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_length">length</a>(sender: &<a href="">signer</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_length">length</a>(sender: &<a href="">signer</a>): u64 <b>acquires</b> <a href="persistence_demo.md#0x1_persistence_demo_State">State</a>{
  <b>assert</b>!(is_testnet(), <a href="_invalid_state">error::invalid_state</a>(<a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>));
  <b>let</b> st = <b>borrow_global</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(<a href="_address_of">signer::address_of</a>(sender));
  <a href="_length">vector::length</a>(&st.hist)
}
</code></pre>



</details>

<a name="0x1_persistence_demo_contains"></a>

## Function `contains`



<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_contains">contains</a>(sender: &<a href="">signer</a>, num: u8): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_contains">contains</a>(sender: &<a href="">signer</a>, num: u8): bool <b>acquires</b> <a href="persistence_demo.md#0x1_persistence_demo_State">State</a> {
  <b>assert</b>!(is_testnet(), <a href="_invalid_state">error::invalid_state</a>(<a href="persistence_demo.md#0x1_persistence_demo_ETESTNET">ETESTNET</a>));
  <b>let</b> st = <b>borrow_global</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(<a href="_address_of">signer::address_of</a>(sender));
  <a href="_contains">vector::contains</a>(&st.hist, &num)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification


<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="persistence_demo.md#0x1_persistence_demo_initialize">initialize</a>(sender: &<a href="">signer</a>)
</code></pre>




<pre><code><b>let</b> addr = <a href="_address_of">signer::address_of</a>(sender);
<b>let</b> init_size = 0;
<b>ensures</b> <a href="_length">vector::length</a>(<b>global</b>&lt;<a href="persistence_demo.md#0x1_persistence_demo_State">State</a>&gt;(addr).hist) == init_size;
</code></pre>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
