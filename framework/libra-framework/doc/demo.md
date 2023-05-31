
<a name="0x1_demo"></a>

# Module `0x1::demo`



-  [Resource `MessageHolder`](#0x1_demo_MessageHolder)
-  [Struct `MessageChangeEvent`](#0x1_demo_MessageChangeEvent)
-  [Constants](#@Constants_0)
-  [Function `get_message`](#0x1_demo_get_message)
-  [Function `set_message`](#0x1_demo_set_message)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::string</a>;
</code></pre>



<a name="0x1_demo_MessageHolder"></a>

## Resource `MessageHolder`



<pre><code><b>struct</b> <a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>message: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>message_change_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="demo.md#0x1_demo_MessageChangeEvent">demo::MessageChangeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_demo_MessageChangeEvent"></a>

## Struct `MessageChangeEvent`



<pre><code><b>struct</b> <a href="demo.md#0x1_demo_MessageChangeEvent">MessageChangeEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>from_message: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>to_message: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_demo_ENO_MESSAGE"></a>

There is no message present


<pre><code><b>const</b> <a href="demo.md#0x1_demo_ENO_MESSAGE">ENO_MESSAGE</a>: u64 = 0;
</code></pre>



<a name="0x1_demo_get_message"></a>

## Function `get_message`



<pre><code><b>public</b> <b>fun</b> <a href="demo.md#0x1_demo_get_message">get_message</a>(addr: <b>address</b>): <a href="_String">string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="demo.md#0x1_demo_get_message">get_message</a>(addr: <b>address</b>): <a href="_String">string::String</a> <b>acquires</b> <a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a>&gt;(addr), <a href="_not_found">error::not_found</a>(<a href="demo.md#0x1_demo_ENO_MESSAGE">ENO_MESSAGE</a>));
    *&<b>borrow_global</b>&lt;<a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a>&gt;(addr).message
}
</code></pre>



</details>

<a name="0x1_demo_set_message"></a>

## Function `set_message`



<pre><code><b>public</b> entry <b>fun</b> <a href="demo.md#0x1_demo_set_message">set_message</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, message: <a href="_String">string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="demo.md#0x1_demo_set_message">set_message</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, message: <a href="_String">string::String</a>)
<b>acquires</b> <a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a> {
    <b>let</b> account_addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>if</b> (!<b>exists</b>&lt;<a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a>&gt;(account_addr)) {
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a> {
            message,
            message_change_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="demo.md#0x1_demo_MessageChangeEvent">MessageChangeEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
        })
    } <b>else</b> {
        <b>let</b> old_message_holder = <b>borrow_global_mut</b>&lt;<a href="demo.md#0x1_demo_MessageHolder">MessageHolder</a>&gt;(account_addr);
        <b>let</b> from_message = *&old_message_holder.message;
        <a href="event.md#0x1_event_emit_event">event::emit_event</a>(&<b>mut</b> old_message_holder.message_change_events, <a href="demo.md#0x1_demo_MessageChangeEvent">MessageChangeEvent</a> {
            from_message,
            to_message: <b>copy</b> message,
        });
        old_message_holder.message = message;
    }
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
