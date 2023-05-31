
<a name="0x1_pledge_accounts"></a>

# Module `0x1::pledge_accounts`



-  [Resource `MyPledges`](#0x1_pledge_accounts_MyPledges)
-  [Resource `PledgeAccount`](#0x1_pledge_accounts_PledgeAccount)
-  [Resource `BeneficiaryPolicy`](#0x1_pledge_accounts_BeneficiaryPolicy)
-  [Constants](#@Constants_0)
-  [Function `publish_beneficiary_policy`](#0x1_pledge_accounts_publish_beneficiary_policy)
-  [Function `maybe_initialize_my_pledges`](#0x1_pledge_accounts_maybe_initialize_my_pledges)
-  [Function `save_pledge`](#0x1_pledge_accounts_save_pledge)
-  [Function `vm_add_to_pledge`](#0x1_pledge_accounts_vm_add_to_pledge)
-  [Function `create_pledge_account`](#0x1_pledge_accounts_create_pledge_account)
-  [Function `add_coin_to_pledge_account`](#0x1_pledge_accounts_add_coin_to_pledge_account)
-  [Function `withdraw_from_all_pledge_accounts`](#0x1_pledge_accounts_withdraw_from_all_pledge_accounts)
-  [Function `withdraw_from_one_pledge_account`](#0x1_pledge_accounts_withdraw_from_one_pledge_account)
-  [Function `withdraw_pct_from_one_pledge_account`](#0x1_pledge_accounts_withdraw_pct_from_one_pledge_account)
-  [Function `vote_to_revoke_beneficiary_policy`](#0x1_pledge_accounts_vote_to_revoke_beneficiary_policy)
-  [Function `try_cancel_vote`](#0x1_pledge_accounts_try_cancel_vote)
-  [Function `find_index_of_vote`](#0x1_pledge_accounts_find_index_of_vote)
-  [Function `tally_vote`](#0x1_pledge_accounts_tally_vote)
-  [Function `dissolve_beneficiary_project`](#0x1_pledge_accounts_dissolve_beneficiary_project)
-  [Function `genesis_infra_escrow_pledge`](#0x1_pledge_accounts_genesis_infra_escrow_pledge)
-  [Function `user_pledge`](#0x1_pledge_accounts_user_pledge)
-  [Function `pledge_at_idx`](#0x1_pledge_accounts_pledge_at_idx)
-  [Function `get_user_pledge_amount`](#0x1_pledge_accounts_get_user_pledge_amount)
-  [Function `get_available_to_beneficiary`](#0x1_pledge_accounts_get_available_to_beneficiary)
-  [Function `get_lifetime_to_beneficiary`](#0x1_pledge_accounts_get_lifetime_to_beneficiary)
-  [Function `get_all_pledgers`](#0x1_pledge_accounts_get_all_pledgers)
-  [Function `get_revoke_vote`](#0x1_pledge_accounts_get_revoke_vote)
-  [Function `test_single_withdrawal`](#0x1_pledge_accounts_test_single_withdrawal)


<pre><code><b>use</b> <a href="chain_status.md#0x1_chain_status">0x1::chain_status</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::fixed_point32</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="reconfiguration.md#0x1_reconfiguration">0x1::reconfiguration</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="testnet.md#0x1_testnet">0x1::testnet</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_pledge_accounts_MyPledges"></a>

## Resource `MyPledges`



<pre><code><b>struct</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>list: <a href="">vector</a>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_PledgeAccount">pledge_accounts::PledgeAccount</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_pledge_accounts_PledgeAccount"></a>

## Resource `PledgeAccount`



<pre><code><b>struct</b> <a href="pledge_accounts.md#0x1_pledge_accounts_PledgeAccount">PledgeAccount</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>address_of_beneficiary: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>pledge: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>epoch_of_last_deposit: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>lifetime_pledged: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>lifetime_withdrawn: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_pledge_accounts_BeneficiaryPolicy"></a>

## Resource `BeneficiaryPolicy`



<pre><code><b>struct</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> <b>has</b> <b>copy</b>, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>purpose: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>vote_threshold_to_revoke: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>burn_funds_on_revoke: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>amount_available: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>lifetime_pledged: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>lifetime_withdrawn: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>pledgers: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>table_votes_to_revoke: <a href="">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>table_revoking_electors: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>total_revoke_vote: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>revoked: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_pledge_accounts_ENON_ZERO_BALANCE"></a>



<pre><code><b>const</b> <a href="pledge_accounts.md#0x1_pledge_accounts_ENON_ZERO_BALANCE">ENON_ZERO_BALANCE</a>: u64 = 150002;
</code></pre>



<a name="0x1_pledge_accounts_ENO_BENEFICIARY_POLICY"></a>



<pre><code><b>const</b> <a href="pledge_accounts.md#0x1_pledge_accounts_ENO_BENEFICIARY_POLICY">ENO_BENEFICIARY_POLICY</a>: u64 = 150001;
</code></pre>



<a name="0x1_pledge_accounts_ENO_PLEDGE_INIT"></a>



<pre><code><b>const</b> <a href="pledge_accounts.md#0x1_pledge_accounts_ENO_PLEDGE_INIT">ENO_PLEDGE_INIT</a>: u64 = 150003;
</code></pre>



<a name="0x1_pledge_accounts_publish_beneficiary_policy"></a>

## Function `publish_beneficiary_policy`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_publish_beneficiary_policy">publish_beneficiary_policy</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, purpose: <a href="">vector</a>&lt;u8&gt;, vote_threshold_to_revoke: u64, burn_funds_on_revoke: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_publish_beneficiary_policy">publish_beneficiary_policy</a>(
  <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>,
  purpose: <a href="">vector</a>&lt;u8&gt;,
  vote_threshold_to_revoke: u64,
  burn_funds_on_revoke: bool
) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>))) {
        <b>let</b> beneficiary_policy = <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
            purpose: purpose,
            vote_threshold_to_revoke: vote_threshold_to_revoke,
            burn_funds_on_revoke: burn_funds_on_revoke,
            amount_available: 0,
            lifetime_pledged: 0,
            lifetime_withdrawn: 0,
            pledgers: <a href="_empty">vector::empty</a>(),
            table_votes_to_revoke: <a href="_empty">vector::empty</a>(),
            table_revoking_electors: <a href="_empty">vector::empty</a>(),
            total_revoke_vote: 0,
            revoked: <b>false</b>

        };
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, beneficiary_policy);
    } <b>else</b> {
      // allow the beneficiary <b>to</b> write drafts, and modify the policy, <b>as</b> long <b>as</b> no pledge <b>has</b> been made.
      <b>let</b> b = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
      <b>if</b> (<a href="_length">vector::length</a>(&b.pledgers) == 0) {
        b.purpose = purpose;
        b.vote_threshold_to_revoke = vote_threshold_to_revoke;
        b.burn_funds_on_revoke = burn_funds_on_revoke;
      }
    }
    // no changes can be made <b>if</b> a pledge <b>has</b> been made.
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_maybe_initialize_my_pledges"></a>

## Function `maybe_initialize_my_pledges`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_maybe_initialize_my_pledges">maybe_initialize_my_pledges</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_maybe_initialize_my_pledges">maybe_initialize_my_pledges</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>) {
    <b>if</b> (!<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>))) {
        <b>let</b> my_pledges = <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a> { list: <a href="_empty">vector::empty</a>() };
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, my_pledges);
    }
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_save_pledge"></a>

## Function `save_pledge`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_save_pledge">save_pledge</a>(sig: &<a href="">signer</a>, address_of_beneficiary: <b>address</b>, pledge: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_save_pledge">save_pledge</a>(
  sig: &<a href="">signer</a>,
  address_of_beneficiary: <b>address</b>,
  pledge: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;
  ) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {

  <b>assert</b>!(<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary), <a href="_invalid_state">error::invalid_state</a>(<a href="pledge_accounts.md#0x1_pledge_accounts_ENO_BENEFICIARY_POLICY">ENO_BENEFICIARY_POLICY</a>));
  <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sig);
  <b>let</b> (found, idx) = <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(&sender_addr, &address_of_beneficiary);
  <b>if</b> (found) {
    <a href="pledge_accounts.md#0x1_pledge_accounts_add_coin_to_pledge_account">add_coin_to_pledge_account</a>(sender_addr, idx, pledge)
  } <b>else</b> {
    <a href="pledge_accounts.md#0x1_pledge_accounts_create_pledge_account">create_pledge_account</a>(sig, address_of_beneficiary, pledge)
  }
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_vm_add_to_pledge"></a>

## Function `vm_add_to_pledge`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_vm_add_to_pledge">vm_add_to_pledge</a>(vm: &<a href="">signer</a>, pledger: <b>address</b>, address_of_beneficiary: <b>address</b>, pledge: &<b>mut</b> <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_vm_add_to_pledge">vm_add_to_pledge</a>(
  vm: &<a href="">signer</a>,
  pledger: <b>address</b>,
  address_of_beneficiary: <b>address</b>,
  pledge: &<b>mut</b> <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;
) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);
  <b>assert</b>!(<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary), <a href="_invalid_state">error::invalid_state</a>(<a href="pledge_accounts.md#0x1_pledge_accounts_ENO_BENEFICIARY_POLICY">ENO_BENEFICIARY_POLICY</a>));

  <b>let</b> (found, idx) = <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(&pledger, &address_of_beneficiary);
  <b>let</b> value = <a href="coin.md#0x1_coin_value">coin::value</a>(pledge);
  <b>if</b> (found) {
    <b>let</b> c = <a href="coin.md#0x1_coin_extract">coin::extract</a>(pledge, value);
    <a href="pledge_accounts.md#0x1_pledge_accounts_add_coin_to_pledge_account">add_coin_to_pledge_account</a>(pledger, idx, c)
  }
  // caller of this function needs <b>to</b> decide what <b>to</b> do <b>if</b> the <a href="coin.md#0x1_coin">coin</a> cannot be added. Which is why its a mutable reference.
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_create_pledge_account"></a>

## Function `create_pledge_account`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_create_pledge_account">create_pledge_account</a>(sig: &<a href="">signer</a>, address_of_beneficiary: <b>address</b>, init_pledge: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_create_pledge_account">create_pledge_account</a>(
  sig: &<a href="">signer</a>,
  // project_id: <a href="">vector</a>&lt;u8&gt;,
  address_of_beneficiary: <b>address</b>,
  init_pledge: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;,
) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
    <b>let</b> <a href="account.md#0x1_account">account</a> = <a href="_address_of">signer::address_of</a>(sig);
    <a href="pledge_accounts.md#0x1_pledge_accounts_maybe_initialize_my_pledges">maybe_initialize_my_pledges</a>(sig);
    <b>let</b> my_pledges = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(<a href="account.md#0x1_account">account</a>);
    <b>let</b> value = <a href="coin.md#0x1_coin_value">coin::value</a>(&init_pledge);
    <b>let</b> new_pledge_account = <a href="pledge_accounts.md#0x1_pledge_accounts_PledgeAccount">PledgeAccount</a> {
        // project_id: project_id,
        address_of_beneficiary: address_of_beneficiary,
        amount: value,
        pledge: init_pledge,
        epoch_of_last_deposit: <a href="reconfiguration.md#0x1_reconfiguration_get_current_epoch">reconfiguration::get_current_epoch</a>(),
        lifetime_pledged: value,
        lifetime_withdrawn: 0
    };
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> my_pledges.list, new_pledge_account);

  <b>let</b> b = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary);
  <a href="_push_back">vector::push_back</a>(&<b>mut</b> b.pledgers, <a href="account.md#0x1_account">account</a>);

  b.amount_available = b.amount_available  + value;
  b.lifetime_pledged = b.lifetime_pledged + value;
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_add_coin_to_pledge_account"></a>

## Function `add_coin_to_pledge_account`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_add_coin_to_pledge_account">add_coin_to_pledge_account</a>(sender_addr: <b>address</b>, idx: u64, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_add_coin_to_pledge_account">add_coin_to_pledge_account</a>(sender_addr: <b>address</b>, idx: u64, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  // <b>let</b> sender_addr = <a href="_address_of">signer::address_of</a>(sender);
  // <b>let</b> (found, _idx) = <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(&sender_addr, &address_of_beneficiary);
  <b>let</b> amount = <a href="coin.md#0x1_coin_value">coin::value</a>(&<a href="coin.md#0x1_coin">coin</a>);
  <b>let</b> my_pledges = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(sender_addr);
  <b>let</b> pledge_account = <a href="_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> my_pledges.list, idx);

  pledge_account.amount = pledge_account.amount + amount;
  pledge_account.epoch_of_last_deposit = <a href="reconfiguration.md#0x1_reconfiguration_get_current_epoch">reconfiguration::get_current_epoch</a>();
  pledge_account.lifetime_pledged = pledge_account.lifetime_pledged + amount;

  // merge the coins in the <a href="account.md#0x1_account">account</a>
  <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> pledge_account.pledge, <a href="coin.md#0x1_coin">coin</a>);

  // must add pledger <b>address</b> the ProjectPledgers list on beneficiary <a href="account.md#0x1_account">account</a>

  <b>let</b> b = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(pledge_account.address_of_beneficiary);
  <a href="_push_back">vector::push_back</a>(&<b>mut</b> b.pledgers, sender_addr);

  b.amount_available = b.amount_available  + amount;
  b.lifetime_pledged = b.lifetime_pledged + amount;

  // exits silently <b>if</b> nothing is found.
  // this is <b>to</b> prevent halting in the <a href="event.md#0x1_event">event</a> that a VM route is calling the function and is unable <b>to</b> check the <b>return</b> value.
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_withdraw_from_all_pledge_accounts"></a>

## Function `withdraw_from_all_pledge_accounts`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_all_pledge_accounts">withdraw_from_all_pledge_accounts</a>(sig_beneficiary: &<a href="">signer</a>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_all_pledge_accounts">withdraw_from_all_pledge_accounts</a>(sig_beneficiary: &<a href="">signer</a>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt; <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
    <b>let</b> pledgers = *&<b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(<a href="_address_of">signer::address_of</a>(sig_beneficiary)).pledgers;

    <b>let</b> amount_available = *&<b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(<a href="_address_of">signer::address_of</a>(sig_beneficiary)).amount_available;



    <b>if</b> (amount_available &lt; 1) {
      <b>return</b> <a href="_none">option::none</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt;()
    };

    <b>let</b> pct_withdraw = <a href="_create_from_rational">fixed_point32::create_from_rational</a>(amount, amount_available);

    <b>let</b> address_of_beneficiary = <a href="_address_of">signer::address_of</a>(sig_beneficiary);

    <b>let</b> i = 0;
    <b>let</b> all_coins = <a href="_none">option::none</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt;();

    <b>while</b> (i &lt; <a href="_length">vector::length</a>(&pledgers)) {
        <b>let</b> pledge_account = *<a href="_borrow">vector::borrow</a>(&pledgers, i);

        // DANGER: this is a private function that changes balances.
        <b>let</b> c = <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_pct_from_one_pledge_account">withdraw_pct_from_one_pledge_account</a>(&address_of_beneficiary, &pledge_account, &pct_withdraw);



        // GROSS: dealing <b>with</b> options in Move.
        // TODO: find a better way.
        <b>if</b> (<a href="_is_none">option::is_none</a>(&all_coins) && <a href="_is_some">option::is_some</a>(&c)) {
          <b>let</b> <a href="coin.md#0x1_coin">coin</a> =  <a href="_extract">option::extract</a>(&<b>mut</b> c);
          <a href="_fill">option::fill</a>(&<b>mut</b> all_coins, <a href="coin.md#0x1_coin">coin</a>);
          <a href="_destroy_none">option::destroy_none</a>(c);
          // <a href="_destroy_none">option::destroy_none</a>(c);
        } <b>else</b> <b>if</b> (<a href="_is_some">option::is_some</a>(&c)) {
          <b>let</b> temp = <a href="_extract">option::extract</a>(&<b>mut</b> all_coins);
          <b>let</b> <a href="coin.md#0x1_coin">coin</a> =  <a href="_extract">option::extract</a>(&<b>mut</b> c);
          <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> temp, <a href="coin.md#0x1_coin">coin</a>);
          <a href="_destroy_none">option::destroy_none</a>(all_coins);
          all_coins = <a href="_some">option::some</a>(temp);
          <a href="_destroy_none">option::destroy_none</a>(c);
        } <b>else</b> {
          <a href="_destroy_none">option::destroy_none</a>(c);
        };

        i = i + 1;
    };

  all_coins
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_withdraw_from_one_pledge_account"></a>

## Function `withdraw_from_one_pledge_account`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_one_pledge_account">withdraw_from_one_pledge_account</a>(address_of_beneficiary: &<b>address</b>, payer: &<b>address</b>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_one_pledge_account">withdraw_from_one_pledge_account</a>(address_of_beneficiary: &<b>address</b>, payer: &<b>address</b>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt; <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {

    <b>let</b> (found, idx) = <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(payer, address_of_beneficiary);

    <b>if</b> (found) {
      <b>let</b> pledge_state = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(*payer);

      <b>let</b> pledge_account = <a href="_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> pledge_state.list, idx);
      <b>if</b> (
        pledge_account.amount &gt; 0 &&
        pledge_account.amount &gt;= amount

        ) {

          pledge_account.amount = pledge_account.amount - amount;

          pledge_account.lifetime_withdrawn = pledge_account.lifetime_withdrawn + amount;


          <b>let</b> <a href="coin.md#0x1_coin">coin</a> = <a href="coin.md#0x1_coin_extract">coin::extract</a>(&<b>mut</b> pledge_account.pledge, amount);

          // <b>return</b> <a href="coin.md#0x1_coin">coin</a>

          // <b>update</b> the beneficiaries state too

          <b>let</b> bp = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*address_of_beneficiary);


          bp.amount_available = bp.amount_available - amount;
          bp.lifetime_withdrawn = bp.lifetime_withdrawn + amount;

          <b>return</b> <a href="_some">option::some</a>(<a href="coin.md#0x1_coin">coin</a>)
        };
    };

    <a href="_none">option::none</a>()
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_withdraw_pct_from_one_pledge_account"></a>

## Function `withdraw_pct_from_one_pledge_account`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_pct_from_one_pledge_account">withdraw_pct_from_one_pledge_account</a>(address_of_beneficiary: &<b>address</b>, payer: &<b>address</b>, pct: &<a href="_FixedPoint32">fixed_point32::FixedPoint32</a>): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_pct_from_one_pledge_account">withdraw_pct_from_one_pledge_account</a>(address_of_beneficiary: &<b>address</b>, payer: &<b>address</b>, pct: &<a href="_FixedPoint32">fixed_point32::FixedPoint32</a>):Option&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt; <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {

    <b>let</b> (found, idx) = <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(payer, address_of_beneficiary);

    <b>if</b> (found) {
      <b>let</b> pledge_state = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(*payer);

      <b>let</b> pledge_account = <a href="_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> pledge_state.list, idx);

      <b>let</b> amount_withdraw = <a href="_multiply_u64">fixed_point32::multiply_u64</a>(pledge_account.amount, *pct);

      <b>if</b> (
        pledge_account.amount &gt; 0 &&
        pledge_account.amount &gt;= amount_withdraw

        ) {
          pledge_account.amount = pledge_account.amount - amount_withdraw;
          pledge_account.lifetime_withdrawn = pledge_account.lifetime_withdrawn + amount_withdraw;

          <b>let</b> <a href="coin.md#0x1_coin">coin</a> = <a href="coin.md#0x1_coin_extract">coin::extract</a>(&<b>mut</b> pledge_account.pledge, amount_withdraw);

          // <b>update</b> the beneficiaries state too

          <b>let</b> bp = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*address_of_beneficiary);

          bp.amount_available = bp.amount_available - amount_withdraw;

          bp.lifetime_withdrawn = bp.lifetime_withdrawn + amount_withdraw;

          <b>return</b> <a href="_some">option::some</a>(<a href="coin.md#0x1_coin">coin</a>)
        };
    };

    <a href="_none">option::none</a>()
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_vote_to_revoke_beneficiary_policy"></a>

## Function `vote_to_revoke_beneficiary_policy`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_vote_to_revoke_beneficiary_policy">vote_to_revoke_beneficiary_policy</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, address_of_beneficiary: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_vote_to_revoke_beneficiary_policy">vote_to_revoke_beneficiary_policy</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, address_of_beneficiary: <b>address</b>) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {


    // first check <b>if</b> they have already voted
    // and <b>if</b> so, cancel in one step
    <a href="pledge_accounts.md#0x1_pledge_accounts_try_cancel_vote">try_cancel_vote</a>(<a href="account.md#0x1_account">account</a>, address_of_beneficiary);

    <b>let</b> pledger = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> bp = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary);

    <a href="_push_back">vector::push_back</a>(&<b>mut</b> bp.table_revoking_electors, pledger);
    <b>let</b> user_pledge_balance = <a href="pledge_accounts.md#0x1_pledge_accounts_get_user_pledge_amount">get_user_pledge_amount</a>(&pledger, &address_of_beneficiary);
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> bp.table_votes_to_revoke, user_pledge_balance);
    bp.total_revoke_vote = bp.total_revoke_vote + user_pledge_balance;

    // The first voter <b>to</b> cross the threshold  also
    // triggers the dissolution.
    <b>if</b> (<a href="pledge_accounts.md#0x1_pledge_accounts_tally_vote">tally_vote</a>(address_of_beneficiary)) {
      <a href="pledge_accounts.md#0x1_pledge_accounts_dissolve_beneficiary_project">dissolve_beneficiary_project</a>(address_of_beneficiary);
    };
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_try_cancel_vote"></a>

## Function `try_cancel_vote`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_try_cancel_vote">try_cancel_vote</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, address_of_beneficiary: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_try_cancel_vote">try_cancel_vote</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, address_of_beneficiary: <b>address</b>) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
    <b>let</b> pledger = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> bp = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary);

    <b>let</b> idx = <a href="pledge_accounts.md#0x1_pledge_accounts_find_index_of_vote">find_index_of_vote</a>(&bp.table_revoking_electors, &pledger);

    <b>if</b> (idx == 0) {
        <b>return</b>
    };
    //adjust the running totals
    <b>let</b> prior_vote = <a href="_borrow">vector::borrow</a>(&bp.table_votes_to_revoke, idx);
    bp.total_revoke_vote = bp.total_revoke_vote - *prior_vote;

    // <b>update</b> the vote
    <a href="_remove">vector::remove</a>(&<b>mut</b> bp.table_revoking_electors, idx);
    <a href="_remove">vector::remove</a>(&<b>mut</b> bp.table_votes_to_revoke, idx);
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_find_index_of_vote"></a>

## Function `find_index_of_vote`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_find_index_of_vote">find_index_of_vote</a>(table_revoking_electors: &<a href="">vector</a>&lt;<b>address</b>&gt;, pledger: &<b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_find_index_of_vote">find_index_of_vote</a>(table_revoking_electors: &<a href="">vector</a>&lt;<b>address</b>&gt;, pledger: &<b>address</b>): u64 {
    <b>if</b> (<a href="_contains">vector::contains</a>(table_revoking_electors, pledger)) {
        <b>return</b> 0
    };

    <b>let</b> i = 0;
    <b>while</b> (i &lt; <a href="_length">vector::length</a>(table_revoking_electors)) {
        <b>if</b> (<a href="_borrow">vector::borrow</a>(table_revoking_electors, i) == pledger) {
            <b>return</b> i
        };
        i = i + 1;
    };
    0 // TODO: <b>return</b> an <a href="">option</a> type
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_tally_vote"></a>

## Function `tally_vote`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_tally_vote">tally_vote</a>(address_of_beneficiary: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_tally_vote">tally_vote</a>(address_of_beneficiary: <b>address</b>): bool <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
    <b>let</b> bp = <b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary);
    <b>let</b> amount_available = bp.amount_available;
    <b>let</b> total_revoke_vote = bp.total_revoke_vote;

    // TODO: <b>use</b> FixedPoint here.
    <b>let</b> ratio = <a href="_create_from_rational">fixed_point32::create_from_rational</a>(total_revoke_vote, amount_available);
    <b>let</b> pct = <a href="_multiply_u64">fixed_point32::multiply_u64</a>(100, ratio);
    <b>if</b> (pct &gt; bp.vote_threshold_to_revoke) {
        <b>return</b> <b>true</b>
    };
    <b>false</b>
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_dissolve_beneficiary_project"></a>

## Function `dissolve_beneficiary_project`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_dissolve_beneficiary_project">dissolve_beneficiary_project</a>(address_of_beneficiary: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_dissolve_beneficiary_project">dissolve_beneficiary_project</a>(address_of_beneficiary: <b>address</b>) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
    <b>let</b> pledgers = *&<b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary).pledgers;

    <b>let</b> is_burn = *&<b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary).burn_funds_on_revoke;

    <b>let</b> i = 0;
    <b>while</b> (i &lt; <a href="_length">vector::length</a>(&pledgers)) {
        <b>let</b> pledge_account = <a href="_borrow">vector::borrow</a>(&pledgers, i);
        <b>let</b> user_pledge_balance = <a href="pledge_accounts.md#0x1_pledge_accounts_get_user_pledge_amount">get_user_pledge_amount</a>(pledge_account, &address_of_beneficiary);
        <b>let</b> c = <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_one_pledge_account">withdraw_from_one_pledge_account</a>(&address_of_beneficiary, pledge_account, user_pledge_balance);

        // TODO: <b>if</b> burn case.
        <b>if</b> (is_burn && <a href="_is_some">option::is_some</a>(&c)) {
          <b>let</b> burn_this = <a href="_extract">option::extract</a>(&<b>mut</b> c);
          <a href="coin.md#0x1_coin_user_burn">coin::user_burn</a>(burn_this);
          <a href="_destroy_none">option::destroy_none</a>(c);
        } <b>else</b> <b>if</b> (<a href="_is_some">option::is_some</a>(&c)) {
          <b>let</b> refund_coin = <a href="_extract">option::extract</a>(&<b>mut</b> c);
          <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(
            address_of_beneficiary,
            // *pledge_account,
            refund_coin,
            // b"revoke pledge",
            // b"", // TODO: clean this up in <a href="ol_account.md#0x1_ol_account">ol_account</a>.
            // <b>false</b>, // TODO: clean this up in <a href="ol_account.md#0x1_ol_account">ol_account</a>.
          );
          <a href="_destroy_none">option::destroy_none</a>(c);
        } <b>else</b> {
          <a href="_destroy_none">option::destroy_none</a>(c);
        };


        i = i + 1;
    };

  <b>let</b> bp = <b>borrow_global_mut</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(address_of_beneficiary);
  bp.revoked = <b>true</b>;

  // otherwise leave the information <b>as</b>-is for reference purposes
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_genesis_infra_escrow_pledge"></a>

## Function `genesis_infra_escrow_pledge`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_genesis_infra_escrow_pledge">genesis_infra_escrow_pledge</a>(vm: &<a href="">signer</a>, <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_genesis_infra_escrow_pledge">genesis_infra_escrow_pledge</a>(vm: &<a href="">signer</a>, <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  // TODO: add <a href="genesis.md#0x1_genesis">genesis</a> time here, once the <a href="timestamp.md#0x1_timestamp">timestamp</a> <a href="genesis.md#0x1_genesis">genesis</a> issue is fixed.
  <a href="chain_status.md#0x1_chain_status_assert_genesis">chain_status::assert_genesis</a>();
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(vm);

  // <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);

  <b>let</b> <a href="coin.md#0x1_coin">coin</a> = <a href="coin.md#0x1_coin_withdraw">coin::withdraw</a>&lt;GasCoin&gt;(<a href="account.md#0x1_account">account</a>, 2500000);
  <a href="pledge_accounts.md#0x1_pledge_accounts_save_pledge">save_pledge</a>(<a href="account.md#0x1_account">account</a>, @ol_framework, <a href="coin.md#0x1_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_user_pledge"></a>

## Function `user_pledge`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_user_pledge">user_pledge</a>(user_sig: &<a href="">signer</a>, beneficiary: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_user_pledge">user_pledge</a>(user_sig: &<a href="">signer</a>, beneficiary: <b>address</b>, amount: u64) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a> {
  <b>let</b> <a href="coin.md#0x1_coin">coin</a> = <a href="coin.md#0x1_coin_withdraw">coin::withdraw</a>(user_sig, amount);
  <a href="pledge_accounts.md#0x1_pledge_accounts_save_pledge">save_pledge</a>(user_sig, beneficiary, <a href="coin.md#0x1_coin">coin</a>);
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_pledge_at_idx"></a>

## Function `pledge_at_idx`



<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(<a href="account.md#0x1_account">account</a>: &<b>address</b>, address_of_beneficiary: &<b>address</b>): (bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(<a href="account.md#0x1_account">account</a>: &<b>address</b>, address_of_beneficiary: &<b>address</b>): (bool, u64) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(*<a href="account.md#0x1_account">account</a>)) {
  <b>let</b> my_pledges = &<b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(*<a href="account.md#0x1_account">account</a>).list;
    <b>let</b> i = 0;
    <b>while</b> (i &lt; <a href="_length">vector::length</a>(my_pledges)) {
        <b>let</b> p = <a href="_borrow">vector::borrow</a>(my_pledges, i);
        <b>if</b> (&p.address_of_beneficiary == address_of_beneficiary) {
            <b>return</b> (<b>true</b>, i)
        };
        i = i + 1;
    };
  };
  (<b>false</b>, 0)
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_get_user_pledge_amount"></a>

## Function `get_user_pledge_amount`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_user_pledge_amount">get_user_pledge_amount</a>(<a href="account.md#0x1_account">account</a>: &<b>address</b>, address_of_beneficiary: &<b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_user_pledge_amount">get_user_pledge_amount</a>(<a href="account.md#0x1_account">account</a>: &<b>address</b>, address_of_beneficiary: &<b>address</b>): u64 <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a> {
    <b>let</b> (found, idx) = <a href="pledge_accounts.md#0x1_pledge_accounts_pledge_at_idx">pledge_at_idx</a>(<a href="account.md#0x1_account">account</a>, address_of_beneficiary);
    <b>if</b> (found) {
      <b>let</b> my_pledges = <b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>&gt;(*<a href="account.md#0x1_account">account</a>);
      <b>let</b> p = <a href="_borrow">vector::borrow</a>(&my_pledges.list, idx);
      <b>return</b> p.amount
    };
    <b>return</b> 0
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_get_available_to_beneficiary"></a>

## Function `get_available_to_beneficiary`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_available_to_beneficiary">get_available_to_beneficiary</a>(bene: &<b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_available_to_beneficiary">get_available_to_beneficiary</a>(bene: &<b>address</b>): u64 <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene)) {
    <b>let</b> bp = <b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene);
    <b>return</b> bp.amount_available
  };
  0
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_get_lifetime_to_beneficiary"></a>

## Function `get_lifetime_to_beneficiary`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_lifetime_to_beneficiary">get_lifetime_to_beneficiary</a>(bene: &<b>address</b>): (u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_lifetime_to_beneficiary">get_lifetime_to_beneficiary</a>(bene: &<b>address</b>): (u64, u64)<b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene)) {
    <b>let</b> bp = <b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene);
    <b>return</b> (bp.lifetime_pledged, bp.lifetime_withdrawn)
  };
  (0, 0)
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_get_all_pledgers"></a>

## Function `get_all_pledgers`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_all_pledgers">get_all_pledgers</a>(bene: &<b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_all_pledgers">get_all_pledgers</a>(bene: &<b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene)) {
    <b>let</b> bp = <b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene);
    <b>return</b> *&bp.pledgers
  };
  <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;()
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_get_revoke_vote"></a>

## Function `get_revoke_vote`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_revoke_vote">get_revoke_vote</a>(bene: &<b>address</b>): (bool, <a href="_FixedPoint32">fixed_point32::FixedPoint32</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_get_revoke_vote">get_revoke_vote</a>(bene: &<b>address</b>): (bool, <a href="_FixedPoint32">fixed_point32::FixedPoint32</a>) <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a> {
  <b>let</b> null = <a href="_create_from_raw_value">fixed_point32::create_from_raw_value</a>(0);
  <b>if</b> (<b>exists</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene)) {
    <b>let</b> bp = <b>borrow_global</b>&lt;<a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>&gt;(*bene);
    <b>if</b> (bp.revoked) {
      <b>return</b> (<b>true</b>, null)
    } <b>else</b> <b>if</b> (
      bp.total_revoke_vote &gt; 0 &&
      bp.amount_available &gt; 0
    ) {
      <b>return</b> (
        <b>false</b>,
        <a href="_create_from_rational">fixed_point32::create_from_rational</a>(bp.total_revoke_vote, bp.amount_available)
      )
    }
  };
  (<b>false</b>, null)
}
</code></pre>



</details>

<a name="0x1_pledge_accounts_test_single_withdrawal"></a>

## Function `test_single_withdrawal`



<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_test_single_withdrawal">test_single_withdrawal</a>(vm: &<a href="">signer</a>, bene: &<b>address</b>, donor: &<b>address</b>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;<a href="gas_coin.md#0x1_gas_coin_GasCoin">gas_coin::GasCoin</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="pledge_accounts.md#0x1_pledge_accounts_test_single_withdrawal">test_single_withdrawal</a>(vm: &<a href="">signer</a>, bene: &<b>address</b>, donor: &<b>address</b>, amount: u64): <a href="_Option">option::Option</a>&lt;<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;GasCoin&gt;&gt; <b>acquires</b> <a href="pledge_accounts.md#0x1_pledge_accounts_MyPledges">MyPledges</a>, <a href="pledge_accounts.md#0x1_pledge_accounts_BeneficiaryPolicy">BeneficiaryPolicy</a>{
  <a href="testnet.md#0x1_testnet_assert_testnet">testnet::assert_testnet</a>(vm);
  <a href="pledge_accounts.md#0x1_pledge_accounts_withdraw_from_one_pledge_account">withdraw_from_one_pledge_account</a>(bene, donor, amount)
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
