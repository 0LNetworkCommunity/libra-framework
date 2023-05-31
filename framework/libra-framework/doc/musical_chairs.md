
<a name="0x1_musical_chairs"></a>

# Module `0x1::musical_chairs`



-  [Resource `Chairs`](#0x1_musical_chairs_Chairs)
-  [Function `initialize`](#0x1_musical_chairs_initialize)
-  [Function `stop_the_music`](#0x1_musical_chairs_stop_the_music)
-  [Function `eval_compliance_impl`](#0x1_musical_chairs_eval_compliance_impl)
-  [Function `get_current_seats`](#0x1_musical_chairs_get_current_seats)


<pre><code><b>use</b> <a href="cases.md#0x1_cases">0x1::cases</a>;
<b>use</b> <a href="chain_status.md#0x1_chain_status">0x1::chain_status</a>;
<b>use</b> <a href="">0x1::fixed_point32</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a name="0x1_musical_chairs_Chairs"></a>

## Resource `Chairs`



<pre><code><b>struct</b> <a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>current_seats: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>history: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_musical_chairs_initialize"></a>

## Function `initialize`

Called by root in genesis to initialize the GAS coin


<pre><code><b>public</b> <b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_initialize">initialize</a>(vm: &<a href="">signer</a>, genesis_seats: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_initialize">initialize</a>(
    vm: &<a href="">signer</a>,
    genesis_seats: u64,
) {
    // <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(vm);
    // TODO: replace <b>with</b> VM
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);

    <a href="chain_status.md#0x1_chain_status_is_genesis">chain_status::is_genesis</a>();
    <b>if</b> (<b>exists</b>&lt;<a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a>&gt;(@ol_framework)) {
        <b>return</b>
    };

    <b>move_to</b>(vm, <a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a> {
        current_seats: genesis_seats,
        history: <a href="_empty">vector::empty</a>&lt;u64&gt;(),
    });
}
</code></pre>



</details>

<a name="0x1_musical_chairs_stop_the_music"></a>

## Function `stop_the_music`



<pre><code><b>public</b> <b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_stop_the_music">stop_the_music</a>(vm: &<a href="">signer</a>): (<a href="">vector</a>&lt;<b>address</b>&gt;, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_stop_the_music">stop_the_music</a>( // sorry, had <b>to</b>.
  vm: &<a href="">signer</a>,
  // height_start: u64,
  // height_end: u64
): (<a href="">vector</a>&lt;<b>address</b>&gt;, u64) <b>acquires</b> <a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);

    <b>let</b> validators = <a href="stake.md#0x1_stake_get_current_validators">stake::get_current_validators</a>();


    <b>let</b> (compliant, _non, ratio) = <a href="musical_chairs.md#0x1_musical_chairs_eval_compliance_impl">eval_compliance_impl</a>(validators);



    <b>let</b> chairs = <b>borrow_global_mut</b>&lt;<a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a>&gt;(@ol_framework);

    // <b>if</b> the ratio of non-compliant nodes is between 0 and 5%
    // we can increase the number of chairs by 1.
    // otherwise (<b>if</b> more than 5% are failing) we wall back <b>to</b> the size of ther performant set.
    <b>if</b> (<a href="_is_zero">fixed_point32::is_zero</a>(*&ratio)) { // catch zeros
      chairs.current_seats = chairs.current_seats + 1;
    } <b>else</b> <b>if</b> (<a href="_multiply_u64">fixed_point32::multiply_u64</a>(100, *&ratio) &lt;= 5){
      chairs.current_seats = chairs.current_seats + 1;
    } <b>else</b> <b>if</b> (<a href="_multiply_u64">fixed_point32::multiply_u64</a>(100, *&ratio) &gt; 5) {
      // remove chairs
      // reduce the validator set <b>to</b> the size of the compliant set.
      chairs.current_seats = <a href="_length">vector::length</a>(&compliant);
    };
    // otherwise do nothing, the validator set is within a tolerable range.

    (compliant, chairs.current_seats)
}
</code></pre>



</details>

<a name="0x1_musical_chairs_eval_compliance_impl"></a>

## Function `eval_compliance_impl`



<pre><code><b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_eval_compliance_impl">eval_compliance_impl</a>(validators: <a href="">vector</a>&lt;<b>address</b>&gt;): (<a href="">vector</a>&lt;<b>address</b>&gt;, <a href="">vector</a>&lt;<b>address</b>&gt;, <a href="_FixedPoint32">fixed_point32::FixedPoint32</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_eval_compliance_impl">eval_compliance_impl</a>(
  validators: <a href="">vector</a>&lt;<b>address</b>&gt;,
) : (<a href="">vector</a>&lt;<b>address</b>&gt;, <a href="">vector</a>&lt;<b>address</b>&gt;, <a href="_FixedPoint32">fixed_point32::FixedPoint32</a>) {

    <b>let</b> val_set_len = <a href="_length">vector::length</a>(&validators);

    <b>let</b> compliant_nodes = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();
    <b>let</b> non_compliant_nodes = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();

    <b>let</b> i = 0;
    <b>while</b> (i &lt; val_set_len) {
        <b>let</b> addr = *<a href="_borrow">vector::borrow</a>(&validators, i);
        <b>if</b> (<a href="cases.md#0x1_cases_get_case">cases::get_case</a>(addr) == 1) {
            <a href="_push_back">vector::push_back</a>(&<b>mut</b> compliant_nodes, addr);
        } <b>else</b> {
            <a href="_push_back">vector::push_back</a>(&<b>mut</b> non_compliant_nodes, addr);
        };
        i = i + 1;
    };

    <b>let</b> good_len = <a href="_length">vector::length</a>(&compliant_nodes) ;
    <b>let</b> bad_len = <a href="_length">vector::length</a>(&non_compliant_nodes);

    // Note: sorry for repetition but necessary for writing tests and debugging.
    <b>let</b> null = <a href="_create_from_raw_value">fixed_point32::create_from_raw_value</a>(0);
    <b>if</b> (good_len &gt; val_set_len) { // safety
      <b>return</b> (<a href="_empty">vector::empty</a>(), <a href="_empty">vector::empty</a>(), null)
    };

    <b>if</b> (bad_len &gt; val_set_len) { // safety
      <b>return</b> (<a href="_empty">vector::empty</a>(), <a href="_empty">vector::empty</a>(), null)
    };

    <b>if</b> ((good_len + bad_len) != val_set_len) { // safety
      <b>return</b> (<a href="_empty">vector::empty</a>(), <a href="_empty">vector::empty</a>(), null)
    };


    <b>let</b> ratio = <b>if</b> (bad_len &gt; 0) {
      <a href="_create_from_rational">fixed_point32::create_from_rational</a>(bad_len, val_set_len)
    } <b>else</b> {
      null
    };

    (compliant_nodes, non_compliant_nodes, ratio)
}
</code></pre>



</details>

<a name="0x1_musical_chairs_get_current_seats"></a>

## Function `get_current_seats`



<pre><code><b>public</b> <b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_get_current_seats">get_current_seats</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="musical_chairs.md#0x1_musical_chairs_get_current_seats">get_current_seats</a>(): u64 <b>acquires</b> <a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a> {
    <b>borrow_global</b>&lt;<a href="musical_chairs.md#0x1_musical_chairs_Chairs">Chairs</a>&gt;(@ol_framework).current_seats
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
