
<a name="0x1_ancestry"></a>

# Module `0x1::ancestry`



-  [Resource `Ancestry`](#0x1_ancestry_Ancestry)
-  [Function `init`](#0x1_ancestry_init)
-  [Function `set_tree`](#0x1_ancestry_set_tree)
-  [Function `get_tree`](#0x1_ancestry_get_tree)
-  [Function `is_family`](#0x1_ancestry_is_family)
-  [Function `is_family_one_in_list`](#0x1_ancestry_is_family_one_in_list)
-  [Function `any_family_in_list`](#0x1_ancestry_any_family_in_list)
-  [Function `fork_migrate`](#0x1_ancestry_fork_migrate)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_ancestry_Ancestry"></a>

## Resource `Ancestry`



<pre><code><b>struct</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>tree: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ancestry_init"></a>

## Function `init`



<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_init">init</a>(new_account_sig: &<a href="">signer</a>, onboarder_sig: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_init">init</a>(new_account_sig: &<a href="">signer</a>, onboarder_sig: &<a href="">signer</a> ) <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>{

    <b>let</b> parent = <a href="_address_of">signer::address_of</a>(onboarder_sig);
    <a href="ancestry.md#0x1_ancestry_set_tree">set_tree</a>(new_account_sig, parent);
}
</code></pre>



</details>

<a name="0x1_ancestry_set_tree"></a>

## Function `set_tree`



<pre><code><b>fun</b> <a href="ancestry.md#0x1_ancestry_set_tree">set_tree</a>(new_account_sig: &<a href="">signer</a>, parent: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ancestry.md#0x1_ancestry_set_tree">set_tree</a>(new_account_sig: &<a href="">signer</a>, parent: <b>address</b> ) <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
  <b>let</b> child = <a href="_address_of">signer::address_of</a>(new_account_sig);

  <b>let</b> new_tree = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();

  // get the parent's <a href="ancestry.md#0x1_ancestry">ancestry</a> <b>if</b> initialized.
  // <b>if</b> not then this is an edge case possibly a migration <a href="">error</a>,
  // and we'll just <b>use</b> the parent.
  <b>if</b> (<b>exists</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(parent)) {
    <b>let</b> parent_state = <b>borrow_global_mut</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(parent);
    <b>let</b> parent_tree = *&parent_state.tree;

    <b>if</b> (<a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&parent_tree) &gt; 0) {
      <a href="_append">vector::append</a>(&<b>mut</b> new_tree, parent_tree);
    };

  };

  // add the parent <b>to</b> the tree
  <a href="_push_back">vector::push_back</a>(&<b>mut</b> new_tree, parent);


  <b>if</b> (!<b>exists</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(child)) {
    <b>move_to</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(new_account_sig, <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
      tree: new_tree,
    });


  } <b>else</b> {
    // this is only for migration <a href="cases.md#0x1_cases">cases</a>.
    <b>let</b> child_ancestry = <b>borrow_global_mut</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(child);
    child_ancestry.tree = new_tree;


  };


}
</code></pre>



</details>

<a name="0x1_ancestry_get_tree"></a>

## Function `get_tree`



<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_get_tree">get_tree</a>(addr: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_get_tree">get_tree</a>(addr: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
  <b>if</b> (<b>exists</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(addr)) {
    *&<b>borrow_global</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(addr).tree
  } <b>else</b> {
    <a href="_empty">vector::empty</a>()
  }

}
</code></pre>



</details>

<a name="0x1_ancestry_is_family"></a>

## Function `is_family`



<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_is_family">is_family</a>(left: <b>address</b>, right: <b>address</b>): (bool, <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_is_family">is_family</a>(left: <b>address</b>, right: <b>address</b>): (bool, <b>address</b>) <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
  <b>let</b> is_family = <b>false</b>;
  <b>let</b> common_ancestor = @0x0;




  // <b>if</b> (<b>exists</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(left) && <b>exists</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(right)) {
    // <b>if</b> tree is empty it will still work.

    <b>let</b> left_tree = <a href="ancestry.md#0x1_ancestry_get_tree">get_tree</a>(left);

    <b>let</b> right_tree = <a href="ancestry.md#0x1_ancestry_get_tree">get_tree</a>(right);



    // check for direct relationship.
    <b>if</b> (<a href="_contains">vector::contains</a>(&left_tree, &right)) <b>return</b> (<b>true</b>, right);
    <b>if</b> (<a href="_contains">vector::contains</a>(&right_tree, &left)) <b>return</b> (<b>true</b>, left);


    <b>let</b> i = 0;
    // check every <b>address</b> on the list <b>if</b> there are overlaps.
    <b>while</b> (i &lt; <a href="_length">vector::length</a>&lt;<b>address</b>&gt;(&left_tree)) {

      <b>let</b> family_addr = <a href="_borrow">vector::borrow</a>(&left_tree, i);
      <b>if</b> (<a href="_contains">vector::contains</a>(&right_tree, family_addr)) {
        is_family = <b>true</b>;
        common_ancestor = *family_addr;

        <b>break</b>
      };
      i = i + 1;
    };

  // };

  (is_family, common_ancestor)
}
</code></pre>



</details>

<a name="0x1_ancestry_is_family_one_in_list"></a>

## Function `is_family_one_in_list`



<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_is_family_one_in_list">is_family_one_in_list</a>(left: <b>address</b>, list: &<a href="">vector</a>&lt;<b>address</b>&gt;): (bool, <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;, <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_is_family_one_in_list">is_family_one_in_list</a>(
  left: <b>address</b>,
  list: &<a href="">vector</a>&lt;<b>address</b>&gt;
):(bool, Option&lt;<b>address</b>&gt;, Option&lt;<b>address</b>&gt;) <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
  <b>let</b> k = 0;
  <b>while</b> (k &lt; <a href="_length">vector::length</a>(list)) {
    <b>let</b> right = <a href="_borrow">vector::borrow</a>(list, k);
    <b>let</b> (fam, _) = <a href="ancestry.md#0x1_ancestry_is_family">is_family</a>(left, *right);
    <b>if</b> (fam) {
      <b>return</b> (<b>true</b>, <a href="_some">option::some</a>(left), <a href="_some">option::some</a>(*right))
    };
    k = k + 1;
  };

  (<b>false</b>, <a href="_none">option::none</a>(), <a href="_none">option::none</a>())
}
</code></pre>



</details>

<a name="0x1_ancestry_any_family_in_list"></a>

## Function `any_family_in_list`



<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_any_family_in_list">any_family_in_list</a>(addr_vec: <a href="">vector</a>&lt;<b>address</b>&gt;): (bool, <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;, <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_any_family_in_list">any_family_in_list</a>(
  addr_vec: <a href="">vector</a>&lt;<b>address</b>&gt;
):(bool, Option&lt;<b>address</b>&gt;, Option&lt;<b>address</b>&gt;) <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>  {
  <b>let</b> i = 0;
  <b>while</b> (<a href="_length">vector::length</a>(&addr_vec) &gt; 1) {
    <b>let</b> left = <a href="_pop_back">vector::pop_back</a>(&<b>mut</b> addr_vec);
    <b>let</b> (fam, left_opt, right_opt) = <a href="ancestry.md#0x1_ancestry_is_family_one_in_list">is_family_one_in_list</a>(left, &addr_vec);
    <b>if</b> (fam) {
      <b>return</b> (fam, left_opt, right_opt)
    };
    i = i + 1;
  };

  (<b>false</b>, <a href="_none">option::none</a>(), <a href="_none">option::none</a>())
}
</code></pre>



</details>

<a name="0x1_ancestry_fork_migrate"></a>

## Function `fork_migrate`



<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_fork_migrate">fork_migrate</a>(vm: &<a href="">signer</a>, child_sig: &<a href="">signer</a>, migrate_tree: <a href="">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ancestry.md#0x1_ancestry_fork_migrate">fork_migrate</a>(
  vm: &<a href="">signer</a>,
  child_sig: &<a href="">signer</a>,
  migrate_tree: <a href="">vector</a>&lt;<b>address</b>&gt;
) <b>acquires</b> <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_vm">system_addresses::assert_vm</a>(vm);
  <b>let</b> child = <a href="_address_of">signer::address_of</a>(child_sig);

  <b>if</b> (!<b>exists</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(child)) {
    <b>move_to</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(child_sig, <a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a> {
      tree: migrate_tree,
    });


  } <b>else</b> {
    // this is only for migration <a href="cases.md#0x1_cases">cases</a>.
    <b>let</b> child_ancestry = <b>borrow_global_mut</b>&lt;<a href="ancestry.md#0x1_ancestry_Ancestry">Ancestry</a>&gt;(child);
    child_ancestry.tree = migrate_tree;

  };
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
