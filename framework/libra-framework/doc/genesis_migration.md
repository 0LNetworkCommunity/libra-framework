
<a name="0x1_genesis_migration"></a>

# Module `0x1::genesis_migration`



-  [Constants](#@Constants_0)
-  [Function `fork_migrate_account`](#0x1_genesis_migration_fork_migrate_account)
-  [Function `is_genesis_val`](#0x1_genesis_migration_is_genesis_val)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="gas_coin.md#0x1_gas_coin">0x1::gas_coin</a>;
<b>use</b> <a href="globals.md#0x1_globals">0x1::globals</a>;
<b>use</b> <a href="infra_escrow.md#0x1_infra_escrow">0x1::infra_escrow</a>;
<b>use</b> <a href="ol_account.md#0x1_ol_account">0x1::ol_account</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="validator_universe.md#0x1_validator_universe">0x1::validator_universe</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_genesis_migration_EBALANCE_MISMATCH"></a>



<pre><code><b>const</b> <a href="genesis_migration.md#0x1_genesis_migration_EBALANCE_MISMATCH">EBALANCE_MISMATCH</a>: u64 = 0;
</code></pre>



<a name="0x1_genesis_migration_VAL_ESCROW_PCT"></a>



<pre><code><b>const</b> <a href="genesis_migration.md#0x1_genesis_migration_VAL_ESCROW_PCT">VAL_ESCROW_PCT</a>: u64 = 80;
</code></pre>



<a name="0x1_genesis_migration_fork_migrate_account"></a>

## Function `fork_migrate_account`

Called by root in genesis to initialize the GAS coin


<pre><code><b>public</b> <b>fun</b> <a href="genesis_migration.md#0x1_genesis_migration_fork_migrate_account">fork_migrate_account</a>(vm: &<a href="">signer</a>, user_sig: &<a href="">signer</a>, auth_key: <a href="">vector</a>&lt;u8&gt;, balance: u64, is_validator: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="genesis_migration.md#0x1_genesis_migration_fork_migrate_account">fork_migrate_account</a>(
    vm: &<a href="">signer</a>,
    user_sig: &<a href="">signer</a>,
    // user_addr: <b>address</b>,
    auth_key: <a href="">vector</a>&lt;u8&gt;,
    balance: u64,
    is_validator: bool,
) {
  <b>let</b> user_addr = <a href="_address_of">signer::address_of</a>(user_sig);
  // <b>if</b> not a validator OR operator of a validator, create a new <a href="account.md#0x1_account">account</a>
  // previously during <a href="genesis.md#0x1_genesis">genesis</a> validator and oper accounts were already created
  <b>if</b> (!<a href="genesis_migration.md#0x1_genesis_migration_is_genesis_val">is_genesis_val</a>(user_addr)) {
    <a href="ol_account.md#0x1_ol_account_vm_create_account_migration">ol_account::vm_create_account_migration</a>(
      vm,
      user_addr,
      auth_key,
    );
  };


  // mint coins again <b>to</b> migrate balance, and all
  // system tracking of balances
  <b>if</b> (balance == 0) {
    <b>return</b>
  };
  // scale up by the <a href="coin.md#0x1_coin">coin</a> split factor
  <b>let</b> new_balance = <a href="globals.md#0x1_globals_get_coin_split_factor">globals::get_coin_split_factor</a>() * balance;

  <a href="gas_coin.md#0x1_gas_coin_mint">gas_coin::mint</a>(vm, user_addr, new_balance);

  <b>let</b> balance = <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;GasCoin&gt;(user_addr);
  <b>assert</b>!(balance == new_balance, <a href="_invalid_state">error::invalid_state</a>(<a href="genesis_migration.md#0x1_genesis_migration_EBALANCE_MISMATCH">EBALANCE_MISMATCH</a>));

  // establish the infrastructure escrow pledge
  <b>if</b> (is_validator) {
    <b>let</b> to_escrow = (balance * <a href="genesis_migration.md#0x1_genesis_migration_VAL_ESCROW_PCT">VAL_ESCROW_PCT</a>) / 100;
    <a href="infra_escrow.md#0x1_infra_escrow_user_pledge_infra">infra_escrow::user_pledge_infra</a>(user_sig, to_escrow)
  };
}
</code></pre>



</details>

<a name="0x1_genesis_migration_is_genesis_val"></a>

## Function `is_genesis_val`



<pre><code><b>fun</b> <a href="genesis_migration.md#0x1_genesis_migration_is_genesis_val">is_genesis_val</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="genesis_migration.md#0x1_genesis_migration_is_genesis_val">is_genesis_val</a>(addr: <b>address</b>): bool {
  // TODO: other checks?
  <a href="validator_universe.md#0x1_validator_universe_is_in_universe">validator_universe::is_in_universe</a>(addr)
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
