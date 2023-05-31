
<a name="0x1_fifo"></a>

# Module `0x1::fifo`


<a name="@Summary_0"></a>

## Summary

Implementation of a FIFO that utilizes two vectors
Achieves Amortized O(1) cost per operation
CAUTION: In worst case this can result in O(n) cost, adjust gas allowance accordingly


-  [Summary](#@Summary_0)
-  [Struct `FIFO`](#0x1_fifo_FIFO)
-  [Constants](#@Constants_1)
-  [Function `empty`](#0x1_fifo_empty)
-  [Function `push`](#0x1_fifo_push)
-  [Function `push_LIFO`](#0x1_fifo_push_LIFO)
-  [Function `pop`](#0x1_fifo_pop)
-  [Function `peek`](#0x1_fifo_peek)
-  [Function `peek_mut`](#0x1_fifo_peek_mut)
-  [Function `len`](#0x1_fifo_len)
-  [Function `perform_swap`](#0x1_fifo_perform_swap)


<pre><code><b>use</b> <a href="">0x1::error</a>;
</code></pre>



<a name="0x1_fifo_FIFO"></a>

## Struct `FIFO`

FIFO implemented using two LIFO vectors
incoming accepts all pushes (added to the end)
when pop is requested, if outgoing is non-empty, element is popped from the end
if outgoing is empty, all elements are popped from the
end of incoming and pushed to outoing
result is a FIFO


<pre><code><b>struct</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt; <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>incoming: <a href="">vector</a>&lt;Element&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>outgoing: <a href="">vector</a>&lt;Element&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_1"></a>

## Constants


<a name="0x1_fifo_ACCESSING_EMPTY_FIFO"></a>



<pre><code><b>const</b> <a href="fifo.md#0x1_fifo_ACCESSING_EMPTY_FIFO">ACCESSING_EMPTY_FIFO</a>: u64 = 32001;
</code></pre>



<a name="0x1_fifo_empty"></a>

## Function `empty`

Create an empty FIFO of some type


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_empty">empty</a>&lt;Element&gt;(): <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_empty">empty</a>&lt;Element&gt;(): <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;{
    <b>let</b> incoming = <a href="_empty">vector::empty</a>&lt;Element&gt;();
    <b>let</b> outgoing = <a href="_empty">vector::empty</a>&lt;Element&gt;();
    <a href="fifo.md#0x1_fifo_FIFO">FIFO</a> {
        incoming: incoming,
        outgoing: outgoing,
    }
}
</code></pre>



</details>

<a name="0x1_fifo_push"></a>

## Function `push`

push an element to the FIFO


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_push">push</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;, new_item: Element)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_push">push</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;, new_item: Element){
    <a href="_push_back">vector::push_back</a>&lt;Element&gt;(&<b>mut</b> v.incoming, new_item);
}
</code></pre>



</details>

<a name="0x1_fifo_push_LIFO"></a>

## Function `push_LIFO`

push an element to the end of the FIFO (so it will be popped first)
useful if you need to give priority to some element


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_push_LIFO">push_LIFO</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;, new_item: Element)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_push_LIFO">push_LIFO</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;, new_item: Element){
    <a href="_push_back">vector::push_back</a>&lt;Element&gt;(&<b>mut</b> v.outgoing, new_item);
}
</code></pre>



</details>

<a name="0x1_fifo_pop"></a>

## Function `pop`

grab the next element from the queue, removing it


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_pop">pop</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;): Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_pop">pop</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;): Element{
    <a href="fifo.md#0x1_fifo_perform_swap">perform_swap</a>&lt;Element&gt;(v);
    //now pop from the outgoing queue
    <a href="_pop_back">vector::pop_back</a>&lt;Element&gt;(&<b>mut</b> v.outgoing)
}
</code></pre>



</details>

<a name="0x1_fifo_peek"></a>

## Function `peek`

return a ref to the next element in the queue, without removing it


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_peek">peek</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;): &Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_peek">peek</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;): & Element{
    <a href="fifo.md#0x1_fifo_perform_swap">perform_swap</a>&lt;Element&gt;(v);

    <b>let</b> len = <a href="_length">vector::length</a>&lt;Element&gt;(& v.outgoing);
    <a href="_borrow">vector::borrow</a>&lt;Element&gt;(& v.outgoing, len - 1)
}
</code></pre>



</details>

<a name="0x1_fifo_peek_mut"></a>

## Function `peek_mut`

return a mutable ref to the next element in the queue, without removing it


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_peek_mut">peek_mut</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;): &<b>mut</b> Element
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_peek_mut">peek_mut</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;): &<b>mut</b> Element{
    <a href="fifo.md#0x1_fifo_perform_swap">perform_swap</a>&lt;Element&gt;(v);

    <b>let</b> len = <a href="_length">vector::length</a>&lt;Element&gt;(& v.outgoing);
    <a href="_borrow_mut">vector::borrow_mut</a>&lt;Element&gt;(&<b>mut</b> v.outgoing, len - 1)
}
</code></pre>



</details>

<a name="0x1_fifo_len"></a>

## Function `len`

get the number of elements in the queue


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_len">len</a>&lt;Element&gt;(v: &<a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="fifo.md#0x1_fifo_len">len</a>&lt;Element&gt;(v: & <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;): u64{
    <a href="_length">vector::length</a>&lt;Element&gt;(& v.outgoing) + <a href="_length">vector::length</a>&lt;Element&gt;(& v.incoming)
}
</code></pre>



</details>

<a name="0x1_fifo_perform_swap"></a>

## Function `perform_swap`

internal function, used when peeking and/or popping to move elements
from the incoming queue to the outgoing queue. Only performs this
action if the outgoing queue is empty.


<pre><code><b>fun</b> <a href="fifo.md#0x1_fifo_perform_swap">perform_swap</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">fifo::FIFO</a>&lt;Element&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="fifo.md#0x1_fifo_perform_swap">perform_swap</a>&lt;Element&gt;(v: &<b>mut</b> <a href="fifo.md#0x1_fifo_FIFO">FIFO</a>&lt;Element&gt;) {
    <b>if</b> (<a href="_length">vector::length</a>&lt;Element&gt;(& v.outgoing) == 0) {
        <b>let</b> len = <a href="_length">vector::length</a>&lt;Element&gt;(&v.incoming);
        <b>assert</b>!(len &gt; 0, <a href="_invalid_state">error::invalid_state</a>(<a href="fifo.md#0x1_fifo_ACCESSING_EMPTY_FIFO">ACCESSING_EMPTY_FIFO</a>));
        //If outgoing is empty, pop all of incoming into outgoing
        <b>while</b> (len &gt; 0) {
            <a href="_push_back">vector::push_back</a>&lt;Element&gt;(&<b>mut</b> v.outgoing,
                <a href="_pop_back">vector::pop_back</a>&lt;Element&gt;(&<b>mut</b> v.incoming));
            len = len - 1;
        }
    };
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
