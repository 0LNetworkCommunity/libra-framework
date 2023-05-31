
<a name="0x1_infra_escrow"></a>

# Module `0x1::infra_escrow`



-  [Function `initialize_infra_pledge`](#0x1_infra_escrow_initialize_infra_pledge)
-  [Function `infra_pledge_withdraw`](#0x1_infra_escrow_infra_pledge_withdraw)
-  [Function `epoch_boundary_collection`](#0x1_infra_escrow_epoch_boundary_collection)
-  [Function `user_pledge_infra`](#0x1_infra_escrow_user_pledge_infra)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="pledge_accounts.md#0x1_pledge_accounts">0x1::pledge_accounts</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="transaction_fee.md#0x1_transaction_fee">0x1::transaction_fee</a>;
</code></pre>



<a name="0x1_infra_escrow_initialize_infra_pledge"></a>

## Function `initialize_infra_pledge`

for use on genesis, creates the infra escrow pledge policy struct


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_initialize_infra_pledge">initialize_infra_pledge</a>(vm: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_initialize_infra_pledge">initialize_infra_pledge</a>(vm: &<a href="">signer</a>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
    // TODO: perhaps this policy needs <b>to</b> be published <b>to</b> a different <b>address</b>?
    <a href="pledge_accounts.md#0x1_pledge_accounts_publish_beneficiary_policy">pledge_accounts::publish_beneficiary_policy</a>(
      vm, // only VM calls at <a href="genesis.md#0x1_genesis">genesis</a>
      b"infra escrow",
      90,
      <b>true</b>
    );
}
</code></pre>



</details>

<a name="0x1_infra_escrow_infra_pledge_withdraw"></a>

## Function `infra_pledge_withdraw`

VM can call down pledged funds.


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_infra_pledge_withdraw">infra_pledge_withdraw</a>(vm: &<a href="">signer</a>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_infra_pledge_withdraw">infra_pledge_withdraw</a>(vm: &<a href="">signer</a>, amount: u64): Option&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt; {
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
    <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_all_pledge_accounts">pledge_accounts::withdraw_from_all_pledge_accounts</a>(vm, amount)
}
</code></pre>



</details>

<a name="0x1_infra_escrow_epoch_boundary_collection"></a>

## Function `epoch_boundary_collection`

Helper for epoch boundaries.
Collects funds from pledge and places temporarily in network account (TransactionFee account)


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_epoch_boundary_collection">epoch_boundary_collection</a>(root: &<a href="">signer</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_epoch_boundary_collection">epoch_boundary_collection</a>(root: &<a href="">signer</a>, amount: u64) {
    <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
    <b>let</b> opt = <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_all_pledge_accounts">pledge_accounts::withdraw_from_all_pledge_accounts</a>(root, amount);

    <b>if</b> (<a href="_is_none">option::is_none</a>(&opt)) {
      <a href="_destroy_none">option::destroy_none</a>(opt);
      <b>return</b>
    };
    <b>let</b> c = <a href="_extract">option::extract</a>(&<b>mut</b> opt);
    <a href="_destroy_none">option::destroy_none</a>(opt);

    <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">transaction_fee::pay_fee</a>(root, c);
}
</code></pre>



</details>

<a name="0x1_infra_escrow_user_pledge_infra"></a>

## Function `user_pledge_infra`



<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_user_pledge_infra">user_pledge_infra</a>(user_sig: &<a href="">signer</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="infra_escrow.md#0x1_infra_escrow_user_pledge_infra">user_pledge_infra</a>(user_sig: &<a href="">signer</a>, amount: u64){

  <a href="pledge_accounts.md#0x1_pledge_accounts_user_pledge">pledge_accounts::user_pledge</a>(user_sig, @ol_framework, amount);
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
