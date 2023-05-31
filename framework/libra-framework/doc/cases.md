
<a name="0x1_cases"></a>

# Module `0x1::cases`


<a name="@Summary_0"></a>

## Summary

This module can be used by root to determine whether a validator is compliant
Validators who are no longer compliant may be kicked out of the validator
set and/or jailed. To be compliant, validators must be BOTH validating and mining.


-  [Summary](#@Summary_0)
-  [Constants](#@Constants_1)
-  [Function `node_above_thresh`](#0x1_cases_node_above_thresh)
-  [Function `get_case`](#0x1_cases_get_case)
-  [Function `get_jailed_set`](#0x1_cases_get_jailed_set)


<pre><code><b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
</code></pre>



<a name="@Constants_1"></a>

## Constants


<a name="0x1_cases_INVALID_DATA"></a>



<pre><code><b>const</b> <a href="cases.md#0x1_cases_INVALID_DATA">INVALID_DATA</a>: u64 = 0;
</code></pre>



<a name="0x1_cases_VALIDATOR_COMPLIANT"></a>



<pre><code><b>const</b> <a href="cases.md#0x1_cases_VALIDATOR_COMPLIANT">VALIDATOR_COMPLIANT</a>: u64 = 1;
</code></pre>



<a name="0x1_cases_VALIDATOR_DOUBLY_NOT_COMPLIANT"></a>



<pre><code><b>const</b> <a href="cases.md#0x1_cases_VALIDATOR_DOUBLY_NOT_COMPLIANT">VALIDATOR_DOUBLY_NOT_COMPLIANT</a>: u64 = 4;
</code></pre>



<a name="0x1_cases_node_above_thresh"></a>

## Function `node_above_thresh`



<pre><code><b>fun</b> <a href="cases.md#0x1_cases_node_above_thresh">node_above_thresh</a>(node_addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="cases.md#0x1_cases_node_above_thresh">node_above_thresh</a>(node_addr: <b>address</b>): bool {
  <b>let</b> idx = <a href="stake.md#0x1_stake_get_validator_index">stake::get_validator_index</a>(node_addr);
  <b>let</b> (proposed, failed) = <a href="stake.md#0x1_stake_get_current_epoch_proposal_counts">stake::get_current_epoch_proposal_counts</a>(idx);

  proposed &gt; failed
}
</code></pre>



</details>

<a name="0x1_cases_get_case"></a>

## Function `get_case`



<pre><code><b>public</b> <b>fun</b> <a href="cases.md#0x1_cases_get_case">get_case</a>(node_addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="cases.md#0x1_cases_get_case">get_case</a>(
    node_addr: <b>address</b>): u64 {

    // this is a failure mode. Only usually seen in rescue missions,
    // <b>where</b> epoch counters are reconfigured by writeset offline.
    // <b>if</b> (height_end &lt; height_start) <b>return</b> <a href="cases.md#0x1_cases_INVALID_DATA">INVALID_DATA</a>;

    // Roles::assert_diem_root(vm); // todo v7
    // did the validator sign blocks above threshold?
    <b>let</b> signs = <a href="cases.md#0x1_cases_node_above_thresh">node_above_thresh</a>(node_addr);

    // <b>let</b> mines = TowerState::node_above_thresh(node_addr);

    <b>if</b> (signs) {
        // compliant: in next set, gets paid, weight increments
        <a href="cases.md#0x1_cases_VALIDATOR_COMPLIANT">VALIDATOR_COMPLIANT</a>
    }
    // V6: Simplify compliance <a href="cases.md#0x1_cases">cases</a> by removing mining.

    // }
    // <b>else</b> <b>if</b> (signs && !mines) {
    //     // half compliant: not in next set, does not get paid, weight
    //     // does not increment.
    //     VALIDATOR_HALF_COMPLIANT
    // }
    // <b>else</b> <b>if</b> (!signs && mines) {
    //     // not compliant: jailed, not in next set, does not get paid,
    //     // weight increments.
    //     VALIDATOR_NOT_COMPLIANT
    // }
    <b>else</b> {
        // not compliant: jailed, not in next set, does not get paid,
        // weight does not increment.
        <a href="cases.md#0x1_cases_VALIDATOR_DOUBLY_NOT_COMPLIANT">VALIDATOR_DOUBLY_NOT_COMPLIANT</a>
    }
}
</code></pre>



</details>

<a name="0x1_cases_get_jailed_set"></a>

## Function `get_jailed_set`



<pre><code><b>public</b> <b>fun</b> <a href="cases.md#0x1_cases_get_jailed_set">get_jailed_set</a>(): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="cases.md#0x1_cases_get_jailed_set">get_jailed_set</a>(): <a href="">vector</a>&lt;<b>address</b>&gt; {
  <b>let</b> validator_set = <a href="stake.md#0x1_stake_get_current_validators">stake::get_current_validators</a>();
  <b>let</b> jailed_set = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();
  <b>let</b> k = 0;
  <b>while</b>(k &lt; <a href="_length">vector::length</a>(&validator_set)){
    <b>let</b> addr = *<a href="_borrow">vector::borrow</a>&lt;<b>address</b>&gt;(&validator_set, k);
    // consensus case 1 allow inclusion into the next validator set.
    <b>if</b> (<a href="cases.md#0x1_cases_get_case">get_case</a>(addr) == 4){
      <a href="_push_back">vector::push_back</a>&lt;<b>address</b>&gt;(&<b>mut</b> jailed_set, addr)
    };
    k = k + 1;
  };
  jailed_set
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
