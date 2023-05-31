
<a name="0x1_proof_of_fee"></a>

# Module `0x1::proof_of_fee`



-  [Resource `ProofOfFeeAuction`](#0x1_proof_of_fee_ProofOfFeeAuction)
-  [Resource `ConsensusReward`](#0x1_proof_of_fee_ConsensusReward)
-  [Constants](#@Constants_0)
-  [Function `init_genesis_baseline_reward`](#0x1_proof_of_fee_init_genesis_baseline_reward)
-  [Function `init`](#0x1_proof_of_fee_init)
-  [Function `end_epoch`](#0x1_proof_of_fee_end_epoch)
-  [Function `get_sorted_vals`](#0x1_proof_of_fee_get_sorted_vals)
-  [Function `sort_vals_impl`](#0x1_proof_of_fee_sort_vals_impl)
-  [Function `fill_seats_and_get_price`](#0x1_proof_of_fee_fill_seats_and_get_price)
-  [Function `audit_qualification`](#0x1_proof_of_fee_audit_qualification)
-  [Function `reward_thermostat`](#0x1_proof_of_fee_reward_thermostat)
-  [Function `set_history`](#0x1_proof_of_fee_set_history)
-  [Function `get_median`](#0x1_proof_of_fee_get_median)
-  [Function `get_consensus_reward`](#0x1_proof_of_fee_get_consensus_reward)
-  [Function `current_bid`](#0x1_proof_of_fee_current_bid)
-  [Function `is_already_retracted`](#0x1_proof_of_fee_is_already_retracted)
-  [Function `top_n_accounts`](#0x1_proof_of_fee_top_n_accounts)
-  [Function `set_bid`](#0x1_proof_of_fee_set_bid)
-  [Function `retract_bid`](#0x1_proof_of_fee_retract_bid)
-  [Function `init_bidding`](#0x1_proof_of_fee_init_bidding)
-  [Function `pof_update_bid`](#0x1_proof_of_fee_pof_update_bid)
-  [Function `pof_retract_bid`](#0x1_proof_of_fee_pof_retract_bid)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::fixed_point32</a>;
<b>use</b> <a href="jail.md#0x1_jail">0x1::jail</a>;
<b>use</b> <a href="reconfiguration.md#0x1_reconfiguration">0x1::reconfiguration</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="slow_wallet.md#0x1_slow_wallet">0x1::slow_wallet</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="validator_universe.md#0x1_validator_universe">0x1::validator_universe</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="vouch.md#0x1_vouch">0x1::vouch</a>;
</code></pre>



<a name="0x1_proof_of_fee_ProofOfFeeAuction"></a>

## Resource `ProofOfFeeAuction`



<pre><code><b>struct</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>bid: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>epoch_expiration: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>last_epoch_retracted: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_proof_of_fee_ConsensusReward"></a>

## Resource `ConsensusReward`



<pre><code><b>struct</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>clearing_price: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>median_win_bid: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>median_history: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_proof_of_fee_EABOVE_RETRACT_LIMIT"></a>

Retracted your bid too many times


<pre><code><b>const</b> <a href="proof_of_fee.md#0x1_proof_of_fee_EABOVE_RETRACT_LIMIT">EABOVE_RETRACT_LIMIT</a>: u64 = 190003;
</code></pre>



<a name="0x1_proof_of_fee_EBID_ABOVE_MAX_PCT"></a>

Bid is above the maximum percentage of the total reward


<pre><code><b>const</b> <a href="proof_of_fee.md#0x1_proof_of_fee_EBID_ABOVE_MAX_PCT">EBID_ABOVE_MAX_PCT</a>: u64 = 190002;
</code></pre>



<a name="0x1_proof_of_fee_ENOT_AN_ACTIVE_VALIDATOR"></a>

Not and active validator


<pre><code><b>const</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>: u64 = 190001;
</code></pre>



<a name="0x1_proof_of_fee_GENESIS_BASELINE_REWARD"></a>

The nominal reward for each validator in each epoch.


<pre><code><b>const</b> <a href="proof_of_fee.md#0x1_proof_of_fee_GENESIS_BASELINE_REWARD">GENESIS_BASELINE_REWARD</a>: u64 = 1000000;
</code></pre>



<a name="0x1_proof_of_fee_init_genesis_baseline_reward"></a>

## Function `init_genesis_baseline_reward`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_init_genesis_baseline_reward">init_genesis_baseline_reward</a>(vm: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_init_genesis_baseline_reward">init_genesis_baseline_reward</a>(vm: &<a href="">signer</a>) {
  <b>if</b> (<a href="_address_of">signer::address_of</a>(vm) != @ol_framework) <b>return</b>;

  <b>if</b> (!<b>exists</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a>&gt;(@ol_framework)) {
    <b>move_to</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a>&gt;(
      vm,
      <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
        value: <a href="proof_of_fee.md#0x1_proof_of_fee_GENESIS_BASELINE_REWARD">GENESIS_BASELINE_REWARD</a>,
        clearing_price: 0,
        median_win_bid: 0,
        median_history: <a href="_empty">vector::empty</a>&lt;u64&gt;(),
      }
    );
  }
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_init"></a>

## Function `init`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_init">init</a>(account_sig: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_init">init</a>(account_sig: &<a href="">signer</a>) {

  <b>let</b> acc = <a href="_address_of">signer::address_of</a>(account_sig);

  // TODO: V7
  // <b>assert</b>!(<a href="validator_universe.md#0x1_validator_universe_is_in_universe">validator_universe::is_in_universe</a>(acc), <a href="_permission_denied">error::permission_denied</a>(<a href="proof_of_fee.md#0x1_proof_of_fee_ENOT_AN_ACTIVE_VALIDATOR">ENOT_AN_ACTIVE_VALIDATOR</a>));

  <b>if</b> (!<b>exists</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(acc)) {
    <b>move_to</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(
    account_sig,
      <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {
        bid: 0,
        epoch_expiration: 0,
        last_epoch_retracted: 0,
      }
    );
  }
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_end_epoch"></a>

## Function `end_epoch`

consolidates all the logic for the epoch boundary


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_end_epoch">end_epoch</a>(vm: &<a href="">signer</a>, outgoing_compliant_set: &<a href="">vector</a>&lt;<b>address</b>&gt;, n_musical_chairs: u64): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_end_epoch">end_epoch</a>(
  vm: &<a href="">signer</a>,
  outgoing_compliant_set: &<a href="">vector</a>&lt;<b>address</b>&gt;,
  n_musical_chairs: u64
): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);

    <b>let</b> sorted_bids = <a href="proof_of_fee.md#0x1_proof_of_fee_get_sorted_vals">get_sorted_vals</a>(<b>false</b>);
    <b>let</b> (auction_winners, price) = <a href="proof_of_fee.md#0x1_proof_of_fee_fill_seats_and_get_price">fill_seats_and_get_price</a>(vm, n_musical_chairs, &sorted_bids, outgoing_compliant_set);

    <a href="slow_wallet.md#0x1_slow_wallet_vm_multi_pay_fee">slow_wallet::vm_multi_pay_fee</a>(vm, &auction_winners, price, &b"proof of fee");

    auction_winners
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_get_sorted_vals"></a>

## Function `get_sorted_vals`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_get_sorted_vals">get_sorted_vals</a>(remove_unqualified: bool): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_get_sorted_vals">get_sorted_vals</a>(remove_unqualified: bool): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
  <b>let</b> eligible_validators = <a href="validator_universe.md#0x1_validator_universe_get_eligible_validators">validator_universe::get_eligible_validators</a>();
  <a href="proof_of_fee.md#0x1_proof_of_fee_sort_vals_impl">sort_vals_impl</a>(eligible_validators, remove_unqualified)
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_sort_vals_impl"></a>

## Function `sort_vals_impl`



<pre><code><b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_sort_vals_impl">sort_vals_impl</a>(eligible_validators: <a href="">vector</a>&lt;<b>address</b>&gt;, remove_unqualified: bool): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_sort_vals_impl">sort_vals_impl</a>(eligible_validators: <a href="">vector</a>&lt;<b>address</b>&gt;, remove_unqualified: bool): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
  // <b>let</b> eligible_validators = <a href="validator_universe.md#0x1_validator_universe_get_eligible_validators">validator_universe::get_eligible_validators</a>();
  <b>let</b> length = <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&eligible_validators);

  // <a href="">vector</a> <b>to</b> store each <b>address</b>'s node_weight
  <b>let</b> weights = <a href="_empty">vector::empty</a>&lt;u64&gt;();
  <b>let</b> filtered_vals = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();
  <b>let</b> k = 0;
  <b>while</b> (k &lt; length) {
    // TODO: Ensure that this <b>address</b> is an active validator

    <b>let</b> cur_address = *<a href="_borrow">vector::borrow</a>&lt;<b>address</b>&gt;(&eligible_validators, k);
    <b>let</b> (bid, _expire) = <a href="proof_of_fee.md#0x1_proof_of_fee_current_bid">current_bid</a>(cur_address);
    <b>if</b> (remove_unqualified && !<a href="proof_of_fee.md#0x1_proof_of_fee_audit_qualification">audit_qualification</a>(&cur_address)) {
      k = k + 1;
      <b>continue</b>
    };
    <a href="_push_back">vector::push_back</a>&lt;u64&gt;(&<b>mut</b> weights, bid);
    <a href="_push_back">vector::push_back</a>&lt;<b>address</b>&gt;(&<b>mut</b> filtered_vals, cur_address);
    k = k + 1;
  };

  // Sorting the accounts <a href="">vector</a> based on value (weights).
  // Bubble sort algorithm
  <b>let</b> len_filtered = <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&filtered_vals);
  <b>if</b> (len_filtered &lt; 2) <b>return</b> filtered_vals;
  <b>let</b> i = 0;
  <b>while</b> (i &lt; len_filtered){
    <b>let</b> j = 0;
    <b>while</b>(j &lt; len_filtered-i-1){

      <b>let</b> value_j = *(<a href="_borrow">vector::borrow</a>&lt;u64&gt;(&weights, j));

      <b>let</b> value_jp1 = *(<a href="_borrow">vector::borrow</a>&lt;u64&gt;(&weights, j+1));
      <b>if</b>(value_j &gt; value_jp1){

        <a href="_swap">vector::swap</a>&lt;u64&gt;(&<b>mut</b> weights, j, j+1);

        <a href="_swap">vector::swap</a>&lt;<b>address</b>&gt;(&<b>mut</b> filtered_vals, j, j+1);
      };
      j = j + 1;

    };
    i = i + 1;

  };


  // Reverse <b>to</b> have sorted order - high <b>to</b> low.
  <a href="_reverse">vector::reverse</a>&lt;<b>address</b>&gt;(&<b>mut</b> filtered_vals);

  <b>return</b> filtered_vals
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_fill_seats_and_get_price"></a>

## Function `fill_seats_and_get_price`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_fill_seats_and_get_price">fill_seats_and_get_price</a>(vm: &<a href="">signer</a>, set_size: u64, sorted_vals_by_bid: &<a href="">vector</a>&lt;<b>address</b>&gt;, proven_nodes: &<a href="">vector</a>&lt;<b>address</b>&gt;): (<a href="">vector</a>&lt;<b>address</b>&gt;, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_fill_seats_and_get_price">fill_seats_and_get_price</a>(
  vm: &<a href="">signer</a>,
  set_size: u64,
  sorted_vals_by_bid: &<a href="">vector</a>&lt;<b>address</b>&gt;,
  proven_nodes: &<a href="">vector</a>&lt;<b>address</b>&gt;
): (<a href="">vector</a>&lt;<b>address</b>&gt;, u64) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
  <b>if</b> (<a href="_address_of">signer::address_of</a>(vm) != @ol_framework) <b>return</b> (<a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;(), 0);

  <b>let</b> seats_to_fill = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();

  // check the max size of the validator set.
  // there may be too few "proven" validators <b>to</b> fill the set <b>with</b> 2/3rds proven nodes of the stated set_size.
  <b>let</b> proven_len = <a href="_length">vector::length</a>(proven_nodes);

  // check <b>if</b> the proven len plus unproven quota will
  // be greater than the set size. Which is the expected.
  // Otherwise the set will need <b>to</b> be smaller than the
  // declared size, because we will have <b>to</b> fill <b>with</b> more unproven nodes.
  <b>let</b> one_third_of_max = proven_len/2;
  <b>let</b> safe_set_size = proven_len + one_third_of_max;

  <b>let</b> (set_size, max_unproven) = <b>if</b> (safe_set_size &lt; set_size) {
    (safe_set_size, safe_set_size/3)
  } <b>else</b> {
    // happy case, unproven bidders are a smaller minority
    (set_size, set_size/3)
  };

  // Now we can seat the validators based on the algo above:
  // 1. seat the proven nodes of previous epoch
  // 2. seat validators who did not participate in the previous epoch:
  // 2a. seat the vals <b>with</b> <a href="jail.md#0x1_jail">jail</a> reputation &lt; 2
  // 2b. seat the remainder of the unproven vals <b>with</b> <a href="">any</a> <a href="jail.md#0x1_jail">jail</a> reputation.

  <b>let</b> num_unproven_added = 0;
  <b>let</b> i = 0u64;
  <b>while</b> (
    (<a href="_length">vector::length</a>(&seats_to_fill) &lt; set_size) &&
    (i &lt; <a href="_length">vector::length</a>(sorted_vals_by_bid))
  ) {
    <b>let</b> val = <a href="_borrow">vector::borrow</a>(sorted_vals_by_bid, i);
    // check <b>if</b> a proven node
    <b>if</b> (<a href="_contains">vector::contains</a>(proven_nodes, val)) {
      <a href="_push_back">vector::push_back</a>(&<b>mut</b> seats_to_fill, *val);
    } <b>else</b> {
      // for unproven nodes, push it <b>to</b> list <b>if</b> we haven't hit limit
      <b>if</b> (num_unproven_added &lt; max_unproven ) {
        // TODO: check <a href="jail.md#0x1_jail">jail</a> reputation
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> seats_to_fill, *val);
        num_unproven_added = num_unproven_added + 1;
      };
    };
    i = i + 1;
  };

  // Set history
  <a href="proof_of_fee.md#0x1_proof_of_fee_set_history">set_history</a>(vm, &seats_to_fill);

  // we failed <b>to</b> seat anyone.
  // <b>let</b> EpochBoundary deal <b>with</b> this.
  <b>if</b> (<a href="_is_empty">vector::is_empty</a>(&seats_to_fill)) {


    <b>return</b> (seats_to_fill, 0)
  };

  // Find the clearing price which all validators will pay
  <b>let</b> lowest_bidder = <a href="_borrow">vector::borrow</a>(&seats_to_fill, <a href="_length">vector::length</a>(&seats_to_fill) - 1);

  <b>let</b> (lowest_bid_pct, _) = <a href="proof_of_fee.md#0x1_proof_of_fee_current_bid">current_bid</a>(*lowest_bidder);



  // <b>update</b> the clearing price
  <b>let</b> cr = <b>borrow_global_mut</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a>&gt;(@ol_framework);
  cr.clearing_price = lowest_bid_pct;

  <b>return</b> (seats_to_fill, lowest_bid_pct)
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_audit_qualification"></a>

## Function `audit_qualification`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_audit_qualification">audit_qualification</a>(val: &<b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_audit_qualification">audit_qualification</a>(val: &<b>address</b>): bool <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {

    // print(&1001);
    // Safety check: node <b>has</b> valid configs
    <b>if</b> (!<a href="stake.md#0x1_stake_stake_pool_exists">stake::stake_pool_exists</a>(*val)) <b>return</b> <b>false</b>;
// print(&1002);
    // is a slow wallet
    <b>if</b> (!<a href="slow_wallet.md#0x1_slow_wallet_is_slow">slow_wallet::is_slow</a>(*val)) <b>return</b> <b>false</b>;
// print(&1003);
    // we can't seat validators that were just jailed
    // NOTE: epoch reconfigure needs <b>to</b> reset the <a href="jail.md#0x1_jail">jail</a>
    // before calling the proof of fee.
    <b>if</b> (<a href="jail.md#0x1_jail_is_jailed">jail::is_jailed</a>(*val)) <b>return</b> <b>false</b>;
// print(&1004);
    // we can't seat validators who don't have minimum viable vouches

    <b>if</b> (!<a href="vouch.md#0x1_vouch_unrelated_buddies_above_thresh">vouch::unrelated_buddies_above_thresh</a>(*val)) <b>return</b> <b>false</b>;
    <b>let</b> (bid_pct, expire) = <a href="proof_of_fee.md#0x1_proof_of_fee_current_bid">current_bid</a>(*val);
// print(&1005);
    <b>if</b> (bid_pct &lt; 1) <b>return</b> <b>false</b>;
// print(&1006);
    // Skip <b>if</b> the bid expired. belt and suspenders, this should have been checked in the sorting above.
    // TODO: make this it's own function so it can be publicly callable, it's useful generally, and for debugging.



    <b>if</b> (<a href="reconfiguration.md#0x1_reconfiguration_get_current_epoch">reconfiguration::get_current_epoch</a>() &gt; expire) <b>return</b> <b>false</b>;
    // skip the user <b>if</b> they don't have sufficient UNLOCKED funds
    // or <b>if</b> the bid expired.
// print(&1007);
    <b>let</b> unlocked_coins = <a href="slow_wallet.md#0x1_slow_wallet_unlocked_amount">slow_wallet::unlocked_amount</a>(*val);
    <b>let</b> (baseline_reward, _, _) = <a href="proof_of_fee.md#0x1_proof_of_fee_get_consensus_reward">get_consensus_reward</a>();
    <b>let</b> coin_required = <a href="_multiply_u64">fixed_point32::multiply_u64</a>(baseline_reward, <a href="_create_from_rational">fixed_point32::create_from_rational</a>(bid_pct, 1000));
// print(&1008);
    <b>if</b> (unlocked_coins &lt; coin_required) <b>return</b> <b>false</b>;
// print(&1009);
    // <b>friend</b> of ours
    <b>true</b>
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_reward_thermostat"></a>

## Function `reward_thermostat`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_reward_thermostat">reward_thermostat</a>(vm: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_reward_thermostat">reward_thermostat</a>(vm: &<a href="">signer</a>) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
  <b>if</b> (<a href="_address_of">signer::address_of</a>(vm) != @ol_framework) {
    <b>return</b>
  };
  // check the bid history
  // <b>if</b> there are 5 days above 95% adjust the reward up by 5%
  // adjust by more <b>if</b> it <b>has</b> been 10 days then, 10%
  // <b>if</b> there are 5 days below 50% adjust the reward down.
  // adjust by more <b>if</b> it <b>has</b> been 10 days then 10%

  <b>let</b> bid_upper_bound = 0950;
  <b>let</b> bid_lower_bound = 0500;

  <b>let</b> short_window: u64 = 5;
  <b>let</b> long_window: u64 = 10;

  <b>let</b> cr = <b>borrow_global_mut</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a>&gt;(@ol_framework);


  <b>let</b> len = <a href="_length">vector::length</a>&lt;u64&gt;(&cr.median_history);
  <b>let</b> i = 0;

  <b>let</b> epochs_above = 0;
  <b>let</b> epochs_below = 0;
  <b>while</b> (i &lt; 16 && i &lt; len) { // max ten days, but may have less in history, filling set should truncate the history at 15 epochs.

    <b>let</b> avg_bid = *<a href="_borrow">vector::borrow</a>&lt;u64&gt;(&cr.median_history, i);

    <b>if</b> (avg_bid &gt; bid_upper_bound) {
      epochs_above = epochs_above + 1;
    } <b>else</b> <b>if</b> (avg_bid &lt; bid_lower_bound) {
      epochs_below = epochs_below + 1;
    };

    i = i + 1;
  };


  <b>if</b> (cr.value &gt; 0) {


    // TODO: this is an initial implementation, we need <b>to</b>
    // decide <b>if</b> we want more granularity in the reward adjustment
    // Note: making this readable for now, but we can optimize later
    <b>if</b> (epochs_above &gt; epochs_below) {

      // <b>if</b> (epochs_above &gt; short_window) {

      // check for zeros.
      // TODO: put a better safety check here

      // If the Validators are bidding near 100% that means
      // the reward is very generous, i.e. their opportunity
      // cost is met at small percentages. This means the
      // implicit bond is very high on validators. E.g.
      // at 1% median bid, the implicit bond is 100x the reward.
      // We need <b>to</b> DECREASE the reward


      <b>if</b> (epochs_above &gt; long_window) {

        // decrease the reward by 10%

        cr.value = cr.value - (cr.value / 10);
        <b>return</b> // <b>return</b> early since we can't increase and decrease simultaneously
      } <b>else</b> <b>if</b> (epochs_above &gt; short_window) {
        // decrease the reward by 5%

        cr.value = cr.value - (cr.value / 20);


        <b>return</b> // <b>return</b> early since we can't increase and decrease simultaneously
      }
    };


      // <b>if</b> validators are bidding low percentages
      // it means the nominal reward is not high enough.
      // That is the validator's opportunity cost is not met within a
      // range <b>where</b> the bond is meaningful.
      // For example: <b>if</b> the bids for the epoch's reward is 50% of the  value, that means the potential profit, is the same <b>as</b> the potential loss.
      // At a 25% bid (potential loss), the profit is thus 75% of the value, which means the implicit bond is 25/75, or 1/3 of the bond, the risk favors the validator. This means among other things, that an attacker can pay for the cost of the attack <b>with</b> the profits. See paper, for more details.

      // we need <b>to</b> INCREASE the reward, so that the bond is more meaningful.


      <b>if</b> (epochs_below &gt; long_window) {


        // increase the reward by 10%
        cr.value = cr.value + (cr.value / 10);
      } <b>else</b> <b>if</b> (epochs_below &gt; short_window) {


        // increase the reward by 5%
        cr.value = cr.value + (cr.value / 20);
      };
    // };
  };
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_set_history"></a>

## Function `set_history`

find the median bid to push to history


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_set_history">set_history</a>(vm: &<a href="">signer</a>, seats_to_fill: &<a href="">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_set_history">set_history</a>(vm: &<a href="">signer</a>, seats_to_fill: &<a href="">vector</a>&lt;<b>address</b>&gt;) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
  <b>if</b> (<a href="_address_of">signer::address_of</a>(vm) != @ol_framework) {
    <b>return</b>
  };


  <b>let</b> median_bid = <a href="proof_of_fee.md#0x1_proof_of_fee_get_median">get_median</a>(seats_to_fill);
  // push <b>to</b> history
  <b>let</b> cr = <b>borrow_global_mut</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a>&gt;(@ol_framework);
  cr.median_win_bid = median_bid;
  <b>if</b> (<a href="_length">vector::length</a>(&cr.median_history) &lt; 10) {

    <a href="_push_back">vector::push_back</a>(&<b>mut</b> cr.median_history, median_bid);
  } <b>else</b> {

    <a href="_remove">vector::remove</a>(&<b>mut</b> cr.median_history, 0);
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> cr.median_history, median_bid);
  };
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_get_median"></a>

## Function `get_median`



<pre><code><b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_get_median">get_median</a>(seats_to_fill: &<a href="">vector</a>&lt;<b>address</b>&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_get_median">get_median</a>(seats_to_fill: &<a href="">vector</a>&lt;<b>address</b>&gt;):u64 <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {
  // TODO: the list is sorted above, so
  // we <b>assume</b> the median is the middle element
  <b>let</b> len = <a href="_length">vector::length</a>(seats_to_fill);
  <b>if</b> (len == 0) {
    <b>return</b> 0
  };
  <b>let</b> median_bidder = <b>if</b> (len &gt; 2) {
    <a href="_borrow">vector::borrow</a>(seats_to_fill, len/2)
  } <b>else</b> {
    <a href="_borrow">vector::borrow</a>(seats_to_fill, 0)
  };
  <b>let</b> (median_bid, _) = <a href="proof_of_fee.md#0x1_proof_of_fee_current_bid">current_bid</a>(*median_bidder);
  <b>return</b> median_bid
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_get_consensus_reward"></a>

## Function `get_consensus_reward`

get the baseline reward from ConsensusReward
returns (reward, clearing_price, median_win_bid)


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_get_consensus_reward">get_consensus_reward</a>(): (u64, u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_get_consensus_reward">get_consensus_reward</a>(): (u64, u64, u64) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
  <b>let</b> b = <b>borrow_global</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a>&gt;(@ol_framework);
  <b>return</b> (b.value, b.clearing_price, b.median_win_bid)
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_current_bid"></a>

## Function `current_bid`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_current_bid">current_bid</a>(node_addr: <b>address</b>): (u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_current_bid">current_bid</a>(node_addr: <b>address</b>): (u64, u64) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(node_addr)) {
    <b>let</b> pof = <b>borrow_global</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(node_addr);
    <b>let</b> e = <a href="reconfiguration.md#0x1_reconfiguration_get_current_epoch">reconfiguration::get_current_epoch</a>();
    // check the expiration of the bid
    // the bid is zero <b>if</b> it expires.
    // The expiration epoch number is inclusive of the epoch.
    // i.e. the bid expires on e + 1.
    <b>if</b> (pof.epoch_expiration &gt;= e || pof.epoch_expiration == 0) {
      <b>return</b> (pof.bid, pof.epoch_expiration)
    };
    <b>return</b> (0, pof.epoch_expiration)
  };
  <b>return</b> (0, 0)
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_is_already_retracted"></a>

## Function `is_already_retracted`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_is_already_retracted">is_already_retracted</a>(node_addr: <b>address</b>): (bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_is_already_retracted">is_already_retracted</a>(node_addr: <b>address</b>): (bool, u64) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(node_addr)) {
    <b>let</b> when_retract = *&<b>borrow_global</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(node_addr).last_epoch_retracted;
    <b>return</b> (<a href="reconfiguration.md#0x1_reconfiguration_get_current_epoch">reconfiguration::get_current_epoch</a>() &gt;= when_retract,  when_retract)
  };
  <b>return</b> (<b>false</b>, 0)
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_top_n_accounts"></a>

## Function `top_n_accounts`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_top_n_accounts">top_n_accounts</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, n: u64, unfiltered: bool): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_top_n_accounts">top_n_accounts</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, n: u64, unfiltered: bool): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>, <a href="proof_of_fee.md#0x1_proof_of_fee_ConsensusReward">ConsensusReward</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(<a href="account.md#0x1_account">account</a>);

    <b>let</b> eligible_validators = <a href="proof_of_fee.md#0x1_proof_of_fee_get_sorted_vals">get_sorted_vals</a>(unfiltered);
    <b>let</b> len = <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&eligible_validators);
    <b>if</b>(len &lt;= n) <b>return</b> eligible_validators;

    <b>let</b> diff = len - n;
    <b>while</b>(diff &gt; 0){
      <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> eligible_validators);
      diff = diff - 1;
    };

    eligible_validators
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_set_bid"></a>

## Function `set_bid`



<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_set_bid">set_bid</a>(account_sig: &<a href="">signer</a>, bid: u64, expiry_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_set_bid">set_bid</a>(account_sig: &<a href="">signer</a>, bid: u64, expiry_epoch: u64) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {

  <b>let</b> acc = <a href="_address_of">signer::address_of</a>(account_sig);
  <b>if</b> (!<b>exists</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(acc)) {
    <a href="proof_of_fee.md#0x1_proof_of_fee_init">init</a>(account_sig);
  };

  // bid must be below 110%
  <b>assert</b>!(bid &lt;= 1100, <a href="_out_of_range">error::out_of_range</a>(<a href="proof_of_fee.md#0x1_proof_of_fee_EBID_ABOVE_MAX_PCT">EBID_ABOVE_MAX_PCT</a>));

  <b>let</b> pof = <b>borrow_global_mut</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(acc);
  pof.epoch_expiration = expiry_epoch;
  pof.bid = bid;
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_retract_bid"></a>

## Function `retract_bid`

Note that the validator will not be bidding on any future
epochs if they retract their bid. The must set a new bid.


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_retract_bid">retract_bid</a>(account_sig: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_retract_bid">retract_bid</a>(account_sig: &<a href="">signer</a>) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {

  <b>let</b> acc = <a href="_address_of">signer::address_of</a>(account_sig);
  <b>if</b> (!<b>exists</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(acc)) {
    <a href="proof_of_fee.md#0x1_proof_of_fee_init">init</a>(account_sig);
  };


  <b>let</b> pof = <b>borrow_global_mut</b>&lt;<a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a>&gt;(acc);
  <b>let</b> this_epoch = <a href="reconfiguration.md#0x1_reconfiguration_get_current_epoch">reconfiguration::get_current_epoch</a>();

  //////// LEAVE COMMENTED. Code for a potential upgrade. ////////
  // See above discussion for retracting of bids.
  //
  // already retracted this epoch
  // <b>assert</b>!(this_epoch &gt; pof.last_epoch_retracted, error::ol_tx(<a href="proof_of_fee.md#0x1_proof_of_fee_EABOVE_RETRACT_LIMIT">EABOVE_RETRACT_LIMIT</a>));
  //////// LEAVE COMMENTED. Code for a potential upgrade. ////////


  pof.epoch_expiration = 0;
  pof.bid = 0;
  pof.last_epoch_retracted = this_epoch;
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_init_bidding"></a>

## Function `init_bidding`



<pre><code><b>public</b> entry <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_init_bidding">init_bidding</a>(sender: <a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_init_bidding">init_bidding</a>(sender: <a href="">signer</a>) {
  <a href="proof_of_fee.md#0x1_proof_of_fee_init">init</a>(&sender);
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_pof_update_bid"></a>

## Function `pof_update_bid`



<pre><code><b>public</b> entry <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_pof_update_bid">pof_update_bid</a>(sender: <a href="">signer</a>, bid: u64, epoch_expiry: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_pof_update_bid">pof_update_bid</a>(sender: <a href="">signer</a>, bid: u64, epoch_expiry: u64) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {
  // <b>update</b> the bid, initializes <b>if</b> not already.
  <a href="proof_of_fee.md#0x1_proof_of_fee_set_bid">set_bid</a>(&sender, bid, epoch_expiry);
}
</code></pre>



</details>

<a name="0x1_proof_of_fee_pof_retract_bid"></a>

## Function `pof_retract_bid`



<pre><code><b>public</b> entry <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_pof_retract_bid">pof_retract_bid</a>(sender: <a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="proof_of_fee.md#0x1_proof_of_fee_pof_retract_bid">pof_retract_bid</a>(sender: <a href="">signer</a>) <b>acquires</b> <a href="proof_of_fee.md#0x1_proof_of_fee_ProofOfFeeAuction">ProofOfFeeAuction</a> {
  // retract a bid
  <a href="proof_of_fee.md#0x1_proof_of_fee_retract_bid">retract_bid</a>(&sender);
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
