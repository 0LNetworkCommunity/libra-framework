
<a name="0x1_rewards"></a>

# Module `0x1::rewards`



-  [Constants](#@Constants_0)
-  [Function `process_single`](#0x1_rewards_process_single)
-  [Function `process_multiple`](#0x1_rewards_process_multiple)
-  [Function `process_recipients_impl`](#0x1_rewards_process_recipients_impl)
-  [Function `pay_reward`](#0x1_rewards_pay_reward)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="stake.md#0x1_stake">0x1::stake</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_rewards_REWARD_MISC"></a>



<pre><code><b>const</b> <a href="rewards.md#0x1_rewards_REWARD_MISC">REWARD_MISC</a>: u8 = 255;
</code></pre>



<a name="0x1_rewards_REWARD_ORACLE"></a>



<pre><code><b>const</b> <a href="rewards.md#0x1_rewards_REWARD_ORACLE">REWARD_ORACLE</a>: u8 = 2;
</code></pre>



<a name="0x1_rewards_REWARD_VALIDATOR"></a>



<pre><code><b>const</b> <a href="rewards.md#0x1_rewards_REWARD_VALIDATOR">REWARD_VALIDATOR</a>: u8 = 1;
</code></pre>



<a name="0x1_rewards_process_single"></a>

## Function `process_single`

process a single reward
root needs to have an owned coin already extracted. Not a mutable borrow.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="rewards.md#0x1_rewards_process_single">process_single</a>(root: &<a href="">signer</a>, addr: <b>address</b>, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;, reward_type: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="rewards.md#0x1_rewards_process_single">process_single</a>(root: &<a href="">signer</a>, addr: <b>address</b>, <a href="coin.md#0x1_coin">coin</a>: Coin&lt;GasCoin&gt;, reward_type: u8) {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <a href="rewards.md#0x1_rewards_pay_reward">pay_reward</a>(root, addr, <a href="coin.md#0x1_coin">coin</a>, reward_type);
}
</code></pre>



</details>

<a name="0x1_rewards_process_multiple"></a>

## Function `process_multiple`

public api for processing bulk rewards
convenience function to process payment for multiple recipients
when the reward is the same for all recipients.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="rewards.md#0x1_rewards_process_multiple">process_multiple</a>(root: &<a href="">signer</a>, list: <a href="">vector</a>&lt;<b>address</b>&gt;, reward_per: u64, reward_budget: &<b>mut</b> <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;, reward_type: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="rewards.md#0x1_rewards_process_multiple">process_multiple</a>(root: &<a href="">signer</a>, list: <a href="">vector</a>&lt;<b>address</b>&gt;, reward_per: u64, reward_budget: &<b>mut</b> Coin&lt;GasCoin&gt;, reward_type: u8) {
  // note the mutable <a href="coin.md#0x1_coin">coin</a> will be retuned <b>to</b> caller for them <b>to</b> do what
  // is necessary, including destroying <b>if</b> it is zero.

  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <a href="rewards.md#0x1_rewards_process_recipients_impl">process_recipients_impl</a>(root, list, reward_per, reward_budget, reward_type);
}
</code></pre>



</details>

<a name="0x1_rewards_process_recipients_impl"></a>

## Function `process_recipients_impl`

process all the validators


<pre><code><b>fun</b> <a href="rewards.md#0x1_rewards_process_recipients_impl">process_recipients_impl</a>(root: &<a href="">signer</a>, list: <a href="">vector</a>&lt;<b>address</b>&gt;, reward_per: u64, reward_budget: &<b>mut</b> <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;, reward_type: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="rewards.md#0x1_rewards_process_recipients_impl">process_recipients_impl</a>(root: &<a href="">signer</a>, list: <a href="">vector</a>&lt;<b>address</b>&gt;, reward_per: u64, reward_budget: &<b>mut</b> Coin&lt;GasCoin&gt;, reward_type: u8) {
  // note the mutable <a href="coin.md#0x1_coin">coin</a> will be retuned <b>to</b> caller for them <b>to</b> do what
  // is necessary, including destroying <b>if</b> it is zero.
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <b>let</b> i = 0;
  <b>while</b> (i &lt; <a href="_length">vector::length</a>(&list)) {
    // split off the reward amount per validator from <a href="coin.md#0x1_coin">coin</a>
    <b>let</b> user_coin = <a href="coin.md#0x1_coin_extract">coin::extract</a>(reward_budget, reward_per);
    <a href="rewards.md#0x1_rewards_pay_reward">pay_reward</a>(root, *<a href="_borrow">vector::borrow</a>(&list, i), user_coin, reward_type);
    // TODO: emit payment <a href="event.md#0x1_event">event</a> in <a href="stake.md#0x1_stake">stake</a>.<b>move</b>
    i = i + 1;
  }

}
</code></pre>



</details>

<a name="0x1_rewards_pay_reward"></a>

## Function `pay_reward`

Pay one validator their reward
belt and suspenders


<pre><code><b>fun</b> <a href="rewards.md#0x1_rewards_pay_reward">pay_reward</a>(root: &<a href="">signer</a>, addr: <b>address</b>, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;, reward_type: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="rewards.md#0x1_rewards_pay_reward">pay_reward</a>(root: &<a href="">signer</a>, addr: <b>address</b>, <a href="coin.md#0x1_coin">coin</a>: Coin&lt;GasCoin&gt;, reward_type: u8) {
  // draw from transaction fees <a href="account.md#0x1_account">account</a>
  // transaction fees <a href="account.md#0x1_account">account</a> should have a subsidy from infra escrow
  // from start of epoch.
  // before we got here we should have checked that we have sufficient
  // funds <b>to</b> pay all members the proof-of-fee reward.
  // <b>if</b> we don't have enough funds, we should exit without <b>abort</b>.
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <b>let</b> amount = <a href="coin.md#0x1_coin_value">coin::value</a>(&<a href="coin.md#0x1_coin">coin</a>);
  <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(addr, <a href="coin.md#0x1_coin">coin</a>);

  <b>if</b> (reward_type == <a href="rewards.md#0x1_rewards_REWARD_VALIDATOR">REWARD_VALIDATOR</a>) {
    <a href="stake.md#0x1_stake_emit_distribute_reward">stake::emit_distribute_reward</a>(root, addr, amount);
  } <b>else</b> <b>if</b> (reward_type == <a href="rewards.md#0x1_rewards_REWARD_ORACLE">REWARD_ORACLE</a>) {
    // oracle::emit_reward_event(addr, <a href="coin.md#0x1_coin">coin</a>);
  } <b>else</b> <b>if</b> (reward_type == <a href="rewards.md#0x1_rewards_REWARD_MISC">REWARD_MISC</a>) {
    // misc::emit_reward_event(addr, <a href="coin.md#0x1_coin">coin</a>);
  };
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
