
<a name="0x1_multisig_account"></a>

# Module `0x1::multisig_account`

Enhanced multisig account standard on Aptos. This is different from the native multisig scheme support enforced via
the account's auth key.

This module allows creating a flexible and powerful multisig account with seamless support for updating owners
without changing the auth key. Users can choose to store transaction payloads waiting for owner signatures on chain
or off chain (primary consideration is decentralization/transparency vs gas cost).

The multisig account is a resource account underneath. By default, it has no auth key and can only be controlled via
the special multisig transaction flow. However, owners can create a transaction to change the auth key to match a
private key off chain if so desired.

Transactions need to be executed in order of creation, similar to transactions for a normal Aptos account (enforced
with account nonce).

The flow is like below:
1. Owners can create a new multisig account by calling create (signer is default single owner) or with
create_with_owners where multiple initial owner addresses can be specified. This is different (and easier) from
the native multisig scheme where the owners' public keys have to be specified. Here, only addresses are needed.
2. Owners can be added/removed any time by calling add_owners or remove_owners. The transactions to do still need
to follow the k-of-n scheme specified for the multisig account.
3. To create a new transaction, an owner can call create_transaction with the transaction payload. This will store
the full transaction payload on chain, which adds decentralization (censorship is not possible as the data is
available on chain) and makes it easier to fetch all transactions waiting for execution. If saving gas is desired,
an owner can alternatively call create_transaction_with_hash where only the payload hash is stored. Later execution
will be verified using the hash. Only owners can create transactions and a transaction id (incremeting id) will be
assigned.
4. To approve or reject a transaction, other owners can call approve() or reject() with the transaction id.
5. If there are enough approvals, any owner can execute the transaction using the special MultisigTransaction type
with the transaction id if the full payload is already stored on chain or with the transaction payload if only a
hash is stored. Transaction execution will first check with this module that the transaction payload has gotten
enough signatures. If so, it will be executed as the multisig account. The owner who executes will pay for gas.
6. If there are enough rejections, any owner can finalize the rejection by calling execute_rejected_transaction().

Note that this multisig account model is not designed to use with a large number of owners. The more owners there
are, the more expensive voting on transactions will become. If a large number of owners is designed, such as in a
flat governance structure, clients are encouraged to write their own modules on top of this multisig account module
and implement the governance voting logic on top.


-  [Resource `MultisigAccount`](#0x1_multisig_account_MultisigAccount)
-  [Struct `MultisigTransaction`](#0x1_multisig_account_MultisigTransaction)
-  [Struct `ExecutionError`](#0x1_multisig_account_ExecutionError)
-  [Struct `MultisigAccountCreationMessage`](#0x1_multisig_account_MultisigAccountCreationMessage)
-  [Struct `AddOwnersEvent`](#0x1_multisig_account_AddOwnersEvent)
-  [Struct `RemoveOwnersEvent`](#0x1_multisig_account_RemoveOwnersEvent)
-  [Struct `UpdateSignaturesRequiredEvent`](#0x1_multisig_account_UpdateSignaturesRequiredEvent)
-  [Struct `CreateTransactionEvent`](#0x1_multisig_account_CreateTransactionEvent)
-  [Struct `VoteEvent`](#0x1_multisig_account_VoteEvent)
-  [Struct `ExecuteRejectedTransactionEvent`](#0x1_multisig_account_ExecuteRejectedTransactionEvent)
-  [Struct `TransactionExecutionSucceededEvent`](#0x1_multisig_account_TransactionExecutionSucceededEvent)
-  [Struct `TransactionExecutionFailedEvent`](#0x1_multisig_account_TransactionExecutionFailedEvent)
-  [Struct `MetadataUpdatedEvent`](#0x1_multisig_account_MetadataUpdatedEvent)
-  [Constants](#@Constants_0)
-  [Function `metadata`](#0x1_multisig_account_metadata)
-  [Function `num_signatures_required`](#0x1_multisig_account_num_signatures_required)
-  [Function `owners`](#0x1_multisig_account_owners)
-  [Function `get_transaction`](#0x1_multisig_account_get_transaction)
-  [Function `get_pending_transactions`](#0x1_multisig_account_get_pending_transactions)
-  [Function `get_next_transaction_payload`](#0x1_multisig_account_get_next_transaction_payload)
-  [Function `can_be_executed`](#0x1_multisig_account_can_be_executed)
-  [Function `can_be_rejected`](#0x1_multisig_account_can_be_rejected)
-  [Function `get_next_multisig_account_address`](#0x1_multisig_account_get_next_multisig_account_address)
-  [Function `last_resolved_sequence_number`](#0x1_multisig_account_last_resolved_sequence_number)
-  [Function `next_sequence_number`](#0x1_multisig_account_next_sequence_number)
-  [Function `vote`](#0x1_multisig_account_vote)
-  [Function `create_with_existing_account`](#0x1_multisig_account_create_with_existing_account)
-  [Function `create`](#0x1_multisig_account_create)
-  [Function `create_with_owners`](#0x1_multisig_account_create_with_owners)
-  [Function `create_with_owners_internal`](#0x1_multisig_account_create_with_owners_internal)
-  [Function `add_owner`](#0x1_multisig_account_add_owner)
-  [Function `add_owners`](#0x1_multisig_account_add_owners)
-  [Function `remove_owner`](#0x1_multisig_account_remove_owner)
-  [Function `remove_owners`](#0x1_multisig_account_remove_owners)
-  [Function `update_signatures_required`](#0x1_multisig_account_update_signatures_required)
-  [Function `update_metadata`](#0x1_multisig_account_update_metadata)
-  [Function `update_metadata_internal`](#0x1_multisig_account_update_metadata_internal)
-  [Function `create_transaction`](#0x1_multisig_account_create_transaction)
-  [Function `create_transaction_with_hash`](#0x1_multisig_account_create_transaction_with_hash)
-  [Function `approve_transaction`](#0x1_multisig_account_approve_transaction)
-  [Function `reject_transaction`](#0x1_multisig_account_reject_transaction)
-  [Function `vote_transanction`](#0x1_multisig_account_vote_transanction)
-  [Function `execute_rejected_transaction`](#0x1_multisig_account_execute_rejected_transaction)
-  [Function `validate_multisig_transaction`](#0x1_multisig_account_validate_multisig_transaction)
-  [Function `successful_transaction_execution_cleanup`](#0x1_multisig_account_successful_transaction_execution_cleanup)
-  [Function `failed_transaction_execution_cleanup`](#0x1_multisig_account_failed_transaction_execution_cleanup)
-  [Function `remove_executed_transaction`](#0x1_multisig_account_remove_executed_transaction)
-  [Function `add_transaction`](#0x1_multisig_account_add_transaction)
-  [Function `create_multisig_account`](#0x1_multisig_account_create_multisig_account)
-  [Function `create_multisig_account_seed`](#0x1_multisig_account_create_multisig_account_seed)
-  [Function `validate_owners`](#0x1_multisig_account_validate_owners)
-  [Function `assert_is_owner`](#0x1_multisig_account_assert_is_owner)
-  [Function `num_approvals_and_rejections`](#0x1_multisig_account_num_approvals_and_rejections)
-  [Function `assert_multisig_account_exists`](#0x1_multisig_account_assert_multisig_account_exists)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="aptos_coin.md#0x1_aptos_coin">0x1::aptos_coin</a>;
<b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="">0x1::features</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::simple_map</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::table</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_multisig_account_MultisigAccount"></a>

## Resource `MultisigAccount`

Represents a multisig account's configurations and transactions.
This will be stored in the multisig account (created as a resource account separate from any owner accounts).


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owners: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>num_signatures_required: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>transactions: <a href="_Table">table::Table</a>&lt;u64, <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">multisig_account::MultisigTransaction</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>last_executed_sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>next_sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>signer_cap: <a href="_Option">option::Option</a>&lt;<a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>metadata: <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>add_owners_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_AddOwnersEvent">multisig_account::AddOwnersEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>remove_owners_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_RemoveOwnersEvent">multisig_account::RemoveOwnersEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>update_signature_required_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_UpdateSignaturesRequiredEvent">multisig_account::UpdateSignaturesRequiredEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>create_transaction_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_CreateTransactionEvent">multisig_account::CreateTransactionEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>vote_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_VoteEvent">multisig_account::VoteEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>execute_rejected_transaction_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_ExecuteRejectedTransactionEvent">multisig_account::ExecuteRejectedTransactionEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>execute_transaction_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_TransactionExecutionSucceededEvent">multisig_account::TransactionExecutionSucceededEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>transaction_execution_failed_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_TransactionExecutionFailedEvent">multisig_account::TransactionExecutionFailedEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>metadata_updated_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="multisig_account.md#0x1_multisig_account_MetadataUpdatedEvent">multisig_account::MetadataUpdatedEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_MultisigTransaction"></a>

## Struct `MultisigTransaction`

A transaction to be executed in a multisig account.
This must contain either the full transaction payload or its hash (stored as bytes).


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>payload: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>payload_hash: <a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>votes: <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<b>address</b>, bool&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>creator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>creation_time_secs: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_ExecutionError"></a>

## Struct `ExecutionError`

Contains information about execution failure.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_ExecutionError">ExecutionError</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>abort_location: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>error_type: <a href="_String">string::String</a></code>
</dt>
<dd>

</dd>
<dt>
<code>error_code: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_MultisigAccountCreationMessage"></a>

## Struct `MultisigAccountCreationMessage`

Used only for verifying multisig account creation on top of existing accounts.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccountCreationMessage">MultisigAccountCreationMessage</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="chain_id.md#0x1_chain_id">chain_id</a>: u8</code>
</dt>
<dd>

</dd>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>owners: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>num_signatures_required: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_AddOwnersEvent"></a>

## Struct `AddOwnersEvent`

Event emitted when new owners are added to the multisig account.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_AddOwnersEvent">AddOwnersEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owners_added: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_RemoveOwnersEvent"></a>

## Struct `RemoveOwnersEvent`

Event emitted when new owners are removed from the multisig account.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_RemoveOwnersEvent">RemoveOwnersEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owners_removed: <a href="">vector</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_UpdateSignaturesRequiredEvent"></a>

## Struct `UpdateSignaturesRequiredEvent`

Event emitted when the number of signatures required is updated.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_UpdateSignaturesRequiredEvent">UpdateSignaturesRequiredEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>old_num_signatures_required: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>new_num_signatures_required: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_CreateTransactionEvent"></a>

## Struct `CreateTransactionEvent`

Event emitted when a transaction is created.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_CreateTransactionEvent">CreateTransactionEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>creator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>transaction: <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">multisig_account::MultisigTransaction</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_VoteEvent"></a>

## Struct `VoteEvent`

Event emitted when an owner approves or rejects a transaction.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_VoteEvent">VoteEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owner: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>approved: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_ExecuteRejectedTransactionEvent"></a>

## Struct `ExecuteRejectedTransactionEvent`

Event emitted when a transaction is officially rejected because the number of rejections has reached the
number of signatures required.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_ExecuteRejectedTransactionEvent">ExecuteRejectedTransactionEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>num_rejections: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>executor: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_TransactionExecutionSucceededEvent"></a>

## Struct `TransactionExecutionSucceededEvent`

Event emitted when a transaction is executed.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_TransactionExecutionSucceededEvent">TransactionExecutionSucceededEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>executor: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>transaction_payload: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>num_approvals: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_TransactionExecutionFailedEvent"></a>

## Struct `TransactionExecutionFailedEvent`

Event emitted when a transaction's execution failed.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_TransactionExecutionFailedEvent">TransactionExecutionFailedEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>executor: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>transaction_payload: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>num_approvals: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>execution_error: <a href="multisig_account.md#0x1_multisig_account_ExecutionError">multisig_account::ExecutionError</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_multisig_account_MetadataUpdatedEvent"></a>

## Struct `MetadataUpdatedEvent`

Event emitted when a transaction's metadata is updated.


<pre><code><b>struct</b> <a href="multisig_account.md#0x1_multisig_account_MetadataUpdatedEvent">MetadataUpdatedEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>old_metadata: <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>new_metadata: <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_multisig_account_DOMAIN_SEPARATOR"></a>

The salt used to create a resource account during multisig account creation.
This is used to avoid conflicts with other modules that also create resource accounts with the same owner
account.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_DOMAIN_SEPARATOR">DOMAIN_SEPARATOR</a>: <a href="">vector</a>&lt;u8&gt; = [97, 112, 116, 111, 115, 95, 102, 114, 97, 109, 101, 119, 111, 114, 107, 58, 58, 109, 117, 108, 116, 105, 115, 105, 103, 95, 97, 99, 99, 111, 117, 110, 116];
</code></pre>



<a name="0x1_multisig_account_EACCOUNT_NOT_MULTISIG"></a>

Specified account is not a multisig account.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EACCOUNT_NOT_MULTISIG">EACCOUNT_NOT_MULTISIG</a>: u64 = 2002;
</code></pre>



<a name="0x1_multisig_account_EDUPLICATE_METADATA_KEY"></a>

The specified metadata contains duplicate attributes (keys).


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EDUPLICATE_METADATA_KEY">EDUPLICATE_METADATA_KEY</a>: u64 = 16;
</code></pre>



<a name="0x1_multisig_account_EDUPLICATE_OWNER"></a>

Owner list cannot contain the same address more than once.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EDUPLICATE_OWNER">EDUPLICATE_OWNER</a>: u64 = 1;
</code></pre>



<a name="0x1_multisig_account_EINVALID_PAYLOAD_HASH"></a>

Payload hash must be exactly 32 bytes (sha3-256).


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EINVALID_PAYLOAD_HASH">EINVALID_PAYLOAD_HASH</a>: u64 = 12;
</code></pre>



<a name="0x1_multisig_account_EINVALID_SEQUENCE_NUMBER"></a>

The sequence number provided is invalid. It must be between [1, next pending transaction - 1].


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EINVALID_SEQUENCE_NUMBER">EINVALID_SEQUENCE_NUMBER</a>: u64 = 17;
</code></pre>



<a name="0x1_multisig_account_EINVALID_SIGNATURES_REQUIRED"></a>

Number of signatures required must be more than zero and at most the total number of owners.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EINVALID_SIGNATURES_REQUIRED">EINVALID_SIGNATURES_REQUIRED</a>: u64 = 11;
</code></pre>



<a name="0x1_multisig_account_EMULTISIG_ACCOUNTS_NOT_ENABLED_YET"></a>

Multisig accounts has not been enabled on this current network yet.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EMULTISIG_ACCOUNTS_NOT_ENABLED_YET">EMULTISIG_ACCOUNTS_NOT_ENABLED_YET</a>: u64 = 14;
</code></pre>



<a name="0x1_multisig_account_ENOT_ENOUGH_APPROVALS"></a>

Transaction has not received enough approvals to be executed.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_ENOT_ENOUGH_APPROVALS">ENOT_ENOUGH_APPROVALS</a>: u64 = 2009;
</code></pre>



<a name="0x1_multisig_account_ENOT_ENOUGH_OWNERS"></a>

Multisig account must have at least one owner.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_ENOT_ENOUGH_OWNERS">ENOT_ENOUGH_OWNERS</a>: u64 = 5;
</code></pre>



<a name="0x1_multisig_account_ENOT_ENOUGH_REJECTIONS"></a>

Transaction has not received enough rejections to be officially rejected.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_ENOT_ENOUGH_REJECTIONS">ENOT_ENOUGH_REJECTIONS</a>: u64 = 10;
</code></pre>



<a name="0x1_multisig_account_ENOT_OWNER"></a>

Account executing this operation is not an owner of the multisig account.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_ENOT_OWNER">ENOT_OWNER</a>: u64 = 2003;
</code></pre>



<a name="0x1_multisig_account_ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH"></a>

The number of metadata keys and values don't match.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH">ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH</a>: u64 = 15;
</code></pre>



<a name="0x1_multisig_account_EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF"></a>

The multisig account itself cannot be an owner.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF">EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF</a>: u64 = 13;
</code></pre>



<a name="0x1_multisig_account_EPAYLOAD_CANNOT_BE_EMPTY"></a>

Transaction payload cannot be empty.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EPAYLOAD_CANNOT_BE_EMPTY">EPAYLOAD_CANNOT_BE_EMPTY</a>: u64 = 4;
</code></pre>



<a name="0x1_multisig_account_EPAYLOAD_DOES_NOT_MATCH_HASH"></a>

Provided target function does not match the hash stored in the on-chain transaction.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_EPAYLOAD_DOES_NOT_MATCH_HASH">EPAYLOAD_DOES_NOT_MATCH_HASH</a>: u64 = 2008;
</code></pre>



<a name="0x1_multisig_account_ETRANSACTION_NOT_FOUND"></a>

Transaction with specified id cannot be found.


<pre><code><b>const</b> <a href="multisig_account.md#0x1_multisig_account_ETRANSACTION_NOT_FOUND">ETRANSACTION_NOT_FOUND</a>: u64 = 2006;
</code></pre>



<a name="0x1_multisig_account_metadata"></a>

## Function `metadata`

Return the multisig account's metadata.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_metadata">metadata</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): <a href="_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="_String">string::String</a>, <a href="">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_metadata">metadata</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): SimpleMap&lt;String, <a href="">vector</a>&lt;u8&gt;&gt; <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>).metadata
}
</code></pre>



</details>

<a name="0x1_multisig_account_num_signatures_required"></a>

## Function `num_signatures_required`

Return the number of signatures required to execute or execute-reject a transaction in the provided
multisig account.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_num_signatures_required">num_signatures_required</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_num_signatures_required">num_signatures_required</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): u64 <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>).num_signatures_required
}
</code></pre>



</details>

<a name="0x1_multisig_account_owners"></a>

## Function `owners`

Return a vector of all of the provided multisig account's owners.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_owners">owners</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_owners">owners</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): <a href="">vector</a>&lt;<b>address</b>&gt; <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>).owners
}
</code></pre>



</details>

<a name="0x1_multisig_account_get_transaction"></a>

## Function `get_transaction`

Return the transaction with the given transaction id.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_transaction">get_transaction</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64): <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">multisig_account::MultisigTransaction</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_transaction">get_transaction</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>,
    sequence_number: u64,
): <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a> <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>assert</b>!(
        sequence_number &gt; 0 && sequence_number &lt; multisig_account_resource.next_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_SEQUENCE_NUMBER">EINVALID_SEQUENCE_NUMBER</a>),
    );
    *<a href="_borrow">table::borrow</a>(&multisig_account_resource.transactions, sequence_number)
}
</code></pre>



</details>

<a name="0x1_multisig_account_get_pending_transactions"></a>

## Function `get_pending_transactions`

Return all pending transactions.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_pending_transactions">get_pending_transactions</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): <a href="">vector</a>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">multisig_account::MultisigTransaction</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_pending_transactions">get_pending_transactions</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): <a href="">vector</a>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a>&gt; <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> pending_transactions: <a href="">vector</a>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a>&gt; = <a href="">vector</a>[];
    <b>let</b> <a href="multisig_account.md#0x1_multisig_account">multisig_account</a> = <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> i = <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.last_executed_sequence_number + 1;
    <b>let</b> next_sequence_number = <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.next_sequence_number;
    <b>while</b> (i &lt; next_sequence_number) {
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> pending_transactions, *<a href="_borrow">table::borrow</a>(&<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.transactions, i));
        i = i + 1;
    };
    pending_transactions
}
</code></pre>



</details>

<a name="0x1_multisig_account_get_next_transaction_payload"></a>

## Function `get_next_transaction_payload`

Return the payload for the next transaction in the queue.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_next_transaction_payload">get_next_transaction_payload</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, provided_payload: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_next_transaction_payload">get_next_transaction_payload</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, provided_payload: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> sequence_number = multisig_account_resource.last_executed_sequence_number + 1;
    <b>let</b> transaction = <a href="_borrow">table::borrow</a>(&multisig_account_resource.transactions, sequence_number);

    <b>if</b> (<a href="_is_some">option::is_some</a>(&transaction.payload)) {
        *<a href="_borrow">option::borrow</a>(&transaction.payload)
    } <b>else</b> {
        provided_payload
    }
}
</code></pre>



</details>

<a name="0x1_multisig_account_can_be_executed"></a>

## Function `can_be_executed`

Return true if the transaction with given transaction id can be executed now.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_can_be_executed">can_be_executed</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_can_be_executed">can_be_executed</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64): bool <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>assert</b>!(
        sequence_number &gt; 0 && sequence_number &lt; multisig_account_resource.next_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_SEQUENCE_NUMBER">EINVALID_SEQUENCE_NUMBER</a>),
    );
    <b>let</b> transaction = <a href="_borrow">table::borrow</a>(&multisig_account_resource.transactions, sequence_number);
    <b>let</b> (num_approvals, _) = <a href="multisig_account.md#0x1_multisig_account_num_approvals_and_rejections">num_approvals_and_rejections</a>(&multisig_account_resource.owners, transaction);
    sequence_number == multisig_account_resource.last_executed_sequence_number + 1 &&
        num_approvals &gt;= multisig_account_resource.num_signatures_required
}
</code></pre>



</details>

<a name="0x1_multisig_account_can_be_rejected"></a>

## Function `can_be_rejected`

Return true if the transaction with given transaction id can be officially rejected.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_can_be_rejected">can_be_rejected</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_can_be_rejected">can_be_rejected</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64): bool <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>assert</b>!(
        sequence_number &gt; 0 && sequence_number &lt; multisig_account_resource.next_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_SEQUENCE_NUMBER">EINVALID_SEQUENCE_NUMBER</a>),
    );
    <b>let</b> transaction = <a href="_borrow">table::borrow</a>(&multisig_account_resource.transactions, sequence_number);
    <b>let</b> (_, num_rejections) = <a href="multisig_account.md#0x1_multisig_account_num_approvals_and_rejections">num_approvals_and_rejections</a>(&multisig_account_resource.owners, transaction);
    sequence_number == multisig_account_resource.last_executed_sequence_number + 1 &&
        num_rejections &gt;= multisig_account_resource.num_signatures_required
}
</code></pre>



</details>

<a name="0x1_multisig_account_get_next_multisig_account_address"></a>

## Function `get_next_multisig_account_address`

Return the predicted address for the next multisig account if created from the given creator address.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_next_multisig_account_address">get_next_multisig_account_address</a>(creator: <b>address</b>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_get_next_multisig_account_address">get_next_multisig_account_address</a>(creator: <b>address</b>): <b>address</b> {
    <b>let</b> owner_nonce = <a href="account.md#0x1_account_get_sequence_number">account::get_sequence_number</a>(creator);
    create_resource_address(&creator, <a href="multisig_account.md#0x1_multisig_account_create_multisig_account_seed">create_multisig_account_seed</a>(to_bytes(&owner_nonce)))
}
</code></pre>



</details>

<a name="0x1_multisig_account_last_resolved_sequence_number"></a>

## Function `last_resolved_sequence_number`

Return the id of the last transaction that was executed (successful or failed) or removed.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_last_resolved_sequence_number">last_resolved_sequence_number</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_last_resolved_sequence_number">last_resolved_sequence_number</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): u64 <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    multisig_account_resource.last_executed_sequence_number
}
</code></pre>



</details>

<a name="0x1_multisig_account_next_sequence_number"></a>

## Function `next_sequence_number`

Return the id of the next transaction created.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_next_sequence_number">next_sequence_number</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_next_sequence_number">next_sequence_number</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>): u64 <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    multisig_account_resource.next_sequence_number
}
</code></pre>



</details>

<a name="0x1_multisig_account_vote"></a>

## Function `vote`

Return a bool tuple indicating whether an owner has voted and if so, whether they voted yes or no.


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_vote">vote</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64, owner: <b>address</b>): (bool, bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_vote">vote</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64, owner: <b>address</b>): (bool, bool) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>assert</b>!(
        sequence_number &gt; 0 && sequence_number &lt; multisig_account_resource.next_sequence_number,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_SEQUENCE_NUMBER">EINVALID_SEQUENCE_NUMBER</a>),
    );
    <b>let</b> transaction = <a href="_borrow">table::borrow</a>(&multisig_account_resource.transactions, sequence_number);
    <b>let</b> votes = &transaction.votes;
    <b>let</b> voted = <a href="_contains_key">simple_map::contains_key</a>(votes, &owner);
    <b>let</b> vote = voted && *<a href="_borrow">simple_map::borrow</a>(votes, &owner);
    (voted, vote)
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_with_existing_account"></a>

## Function `create_with_existing_account`

Creates a new multisig account on top of an existing account.

This offers a migration path for an existing account with a multi-ed25519 auth key (native multisig account).
In order to ensure a malicious module cannot obtain backdoor control over an existing account, a signed message
with a valid signature from the account's auth key is required.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_with_existing_account">create_with_existing_account</a>(multisig_address: <b>address</b>, owners: <a href="">vector</a>&lt;<b>address</b>&gt;, num_signatures_required: u64, account_scheme: u8, account_public_key: <a href="">vector</a>&lt;u8&gt;, create_multisig_account_signed_message: <a href="">vector</a>&lt;u8&gt;, metadata_keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_with_existing_account">create_with_existing_account</a>(
    multisig_address: <b>address</b>,
    owners: <a href="">vector</a>&lt;<b>address</b>&gt;,
    num_signatures_required: u64,
    account_scheme: u8,
    account_public_key: <a href="">vector</a>&lt;u8&gt;,
    create_multisig_account_signed_message: <a href="">vector</a>&lt;u8&gt;,
    metadata_keys: <a href="">vector</a>&lt;String&gt;,
    metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    // Verify that the `<a href="multisig_account.md#0x1_multisig_account_MultisigAccountCreationMessage">MultisigAccountCreationMessage</a>` <b>has</b> the right information and is signed by the <a href="account.md#0x1_account">account</a>
    // owner's key.
    <b>let</b> proof_challenge = <a href="multisig_account.md#0x1_multisig_account_MultisigAccountCreationMessage">MultisigAccountCreationMessage</a> {
        <a href="chain_id.md#0x1_chain_id">chain_id</a>: <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>(),
        account_address: multisig_address,
        sequence_number: <a href="account.md#0x1_account_get_sequence_number">account::get_sequence_number</a>(multisig_address),
        owners,
        num_signatures_required,
    };
    <a href="account.md#0x1_account_verify_signed_message">account::verify_signed_message</a>(
        multisig_address,
        account_scheme,
        account_public_key,
        create_multisig_account_signed_message,
        proof_challenge,
    );

    // We create the <a href="">signer</a> for the multisig <a href="account.md#0x1_account">account</a> here since this is required <b>to</b> add the <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> resource
    // This should be safe and authorized because we have verified the signed message from the existing <a href="account.md#0x1_account">account</a>
    // that authorizes creating a multisig <a href="account.md#0x1_account">account</a> <b>with</b> the specified owners and signature threshold.
    <b>let</b> <a href="multisig_account.md#0x1_multisig_account">multisig_account</a> = &<a href="create_signer.md#0x1_create_signer">create_signer</a>(multisig_address);
    <a href="multisig_account.md#0x1_multisig_account_create_with_owners_internal">create_with_owners_internal</a>(
        <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>,
        owners,
        num_signatures_required,
        <a href="_none">option::none</a>&lt;SignerCapability&gt;(),
        metadata_keys,
        metadata_values,
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_create"></a>

## Function `create`

Creates a new multisig account and add the signer as a single owner.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create">create</a>(owner: &<a href="">signer</a>, num_signatures_required: u64, metadata_keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create">create</a>(
    owner: &<a href="">signer</a>,
    num_signatures_required: u64,
    metadata_keys: <a href="">vector</a>&lt;String&gt;,
    metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_create_with_owners">create_with_owners</a>(owner, <a href="">vector</a>[], num_signatures_required, metadata_keys, metadata_values);
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_with_owners"></a>

## Function `create_with_owners`

Creates a new multisig account with the specified additional owner list and signatures required.

@param additional_owners The owner account who calls this function cannot be in the additional_owners and there
cannot be any duplicate owners in the list.
@param num_signatures_required The number of signatures required to execute a transaction. Must be at least 1 and
at most the total number of owners.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_with_owners">create_with_owners</a>(owner: &<a href="">signer</a>, additional_owners: <a href="">vector</a>&lt;<b>address</b>&gt;, num_signatures_required: u64, metadata_keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_with_owners">create_with_owners</a>(
    owner: &<a href="">signer</a>,
    additional_owners: <a href="">vector</a>&lt;<b>address</b>&gt;,
    num_signatures_required: u64,
    metadata_keys: <a href="">vector</a>&lt;String&gt;,
    metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> (<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, multisig_signer_cap) = <a href="multisig_account.md#0x1_multisig_account_create_multisig_account">create_multisig_account</a>(owner);
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> additional_owners, address_of(owner));
    <a href="multisig_account.md#0x1_multisig_account_create_with_owners_internal">create_with_owners_internal</a>(
        &<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>,
        additional_owners,
        num_signatures_required,
        <a href="_some">option::some</a>(multisig_signer_cap),
        metadata_keys,
        metadata_values,
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_with_owners_internal"></a>

## Function `create_with_owners_internal`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_with_owners_internal">create_with_owners_internal</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, owners: <a href="">vector</a>&lt;<b>address</b>&gt;, num_signatures_required: u64, multisig_account_signer_cap: <a href="_Option">option::Option</a>&lt;<a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>&gt;, metadata_keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_with_owners_internal">create_with_owners_internal</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>,
    owners: <a href="">vector</a>&lt;<b>address</b>&gt;,
    num_signatures_required: u64,
    multisig_account_signer_cap: Option&lt;SignerCapability&gt;,
    metadata_keys: <a href="">vector</a>&lt;String&gt;,
    metadata_values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>assert</b>!(<a href="_multisig_accounts_enabled">features::multisig_accounts_enabled</a>(), <a href="_unavailable">error::unavailable</a>(<a href="multisig_account.md#0x1_multisig_account_EMULTISIG_ACCOUNTS_NOT_ENABLED_YET">EMULTISIG_ACCOUNTS_NOT_ENABLED_YET</a>));
    <b>assert</b>!(
        num_signatures_required &gt; 0 && <a href="multisig_account.md#0x1_multisig_account_num_signatures_required">num_signatures_required</a> &lt;= <a href="_length">vector::length</a>(&owners),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_SIGNATURES_REQUIRED">EINVALID_SIGNATURES_REQUIRED</a>),
    );

    <b>let</b> multisig_address = address_of(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_validate_owners">validate_owners</a>(&owners, multisig_address);
    <b>move_to</b>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
        owners,
        num_signatures_required,
        transactions: <a href="_new">table::new</a>&lt;u64, <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a>&gt;(),
        metadata: <a href="_create">simple_map::create</a>&lt;String, <a href="">vector</a>&lt;u8&gt;&gt;(),
        // First transaction will start at id 1 instead of 0.
        last_executed_sequence_number: 0,
        next_sequence_number: 1,
        signer_cap: multisig_account_signer_cap,
        add_owners_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_AddOwnersEvent">AddOwnersEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        remove_owners_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_RemoveOwnersEvent">RemoveOwnersEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        update_signature_required_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_UpdateSignaturesRequiredEvent">UpdateSignaturesRequiredEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        create_transaction_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_CreateTransactionEvent">CreateTransactionEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        vote_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_VoteEvent">VoteEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        execute_rejected_transaction_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_ExecuteRejectedTransactionEvent">ExecuteRejectedTransactionEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        execute_transaction_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_TransactionExecutionSucceededEvent">TransactionExecutionSucceededEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        transaction_execution_failed_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_TransactionExecutionFailedEvent">TransactionExecutionFailedEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
        metadata_updated_events: new_event_handle&lt;<a href="multisig_account.md#0x1_multisig_account_MetadataUpdatedEvent">MetadataUpdatedEvent</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>),
    });

    <a href="multisig_account.md#0x1_multisig_account_update_metadata_internal">update_metadata_internal</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, metadata_keys, metadata_values, <b>false</b>);
}
</code></pre>



</details>

<a name="0x1_multisig_account_add_owner"></a>

## Function `add_owner`

Similar to add_owners, but only allow adding one owner.


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_add_owner">add_owner</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, new_owner: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_add_owner">add_owner</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, new_owner: <b>address</b>) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_add_owners">add_owners</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, <a href="">vector</a>[new_owner]);
}
</code></pre>



</details>

<a name="0x1_multisig_account_add_owners"></a>

## Function `add_owners`

Add new owners to the multisig account. This can only be invoked by the multisig account itself, through the
proposal flow.

Note that this function is not public so it can only be invoked directly instead of via a module or script. This
ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
maliciously alter the owners list.


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_add_owners">add_owners</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, new_owners: <a href="">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_add_owners">add_owners</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, new_owners: <a href="">vector</a>&lt;<b>address</b>&gt;) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    // Short circuit <b>if</b> new owners list is empty.
    // This avoids emitting an <a href="event.md#0x1_event">event</a> <b>if</b> no changes happen, which is confusing <b>to</b> off-chain components.
    <b>if</b> (<a href="_length">vector::length</a>(&new_owners) == 0) {
        <b>return</b>
    };

    <b>let</b> multisig_address = address_of(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(multisig_address);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(multisig_address);

    <a href="_append">vector::append</a>(&<b>mut</b> multisig_account_resource.owners, new_owners);
    // This will fail <b>if</b> an existing owner is added again.
    <a href="multisig_account.md#0x1_multisig_account_validate_owners">validate_owners</a>(&multisig_account_resource.owners, multisig_address);
    emit_event(&<b>mut</b> multisig_account_resource.add_owners_events, <a href="multisig_account.md#0x1_multisig_account_AddOwnersEvent">AddOwnersEvent</a> {
        owners_added: new_owners,
    });
}
</code></pre>



</details>

<a name="0x1_multisig_account_remove_owner"></a>

## Function `remove_owner`

Similar to remove_owners, but only allow removing one owner.


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_remove_owner">remove_owner</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, owner_to_remove: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_remove_owner">remove_owner</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, owner_to_remove: <b>address</b>) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_remove_owners">remove_owners</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, <a href="">vector</a>[owner_to_remove]);
}
</code></pre>



</details>

<a name="0x1_multisig_account_remove_owners"></a>

## Function `remove_owners`

Remove owners from the multisig account. This can only be invoked by the multisig account itself, through the
proposal flow.

This function skips any owners who are not in the multisig account's list of owners.
Note that this function is not public so it can only be invoked directly instead of via a module or script. This
ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
maliciously alter the owners list.


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_remove_owners">remove_owners</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, owners_to_remove: <a href="">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_remove_owners">remove_owners</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, owners_to_remove: <a href="">vector</a>&lt;<b>address</b>&gt;) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    // Short circuit <b>if</b> the list of owners <b>to</b> remove is empty.
    // This avoids emitting an <a href="event.md#0x1_event">event</a> <b>if</b> no changes happen, which is confusing <b>to</b> off-chain components.
    <b>if</b> (<a href="_length">vector::length</a>(&owners_to_remove) == 0) {
        <b>return</b>
    };

    <b>let</b> multisig_address = address_of(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(multisig_address);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(multisig_address);

    <b>let</b> owners = &<b>mut</b> multisig_account_resource.owners;
    <b>let</b> owners_removed = <a href="_empty">vector::empty</a>&lt;<b>address</b>&gt;();
    vector::for_each_ref(&owners_to_remove, |owner_to_remove| {
        <b>let</b> owner_to_remove = *owner_to_remove;
        <b>let</b> (found, index) = <a href="_index_of">vector::index_of</a>(owners, &owner_to_remove);
        // Only remove an owner <b>if</b> they're present in the owners list.
        <b>if</b> (found) {
            <a href="_push_back">vector::push_back</a>(&<b>mut</b> owners_removed, owner_to_remove);
            <a href="_swap_remove">vector::swap_remove</a>(owners, index);
        };
    });

    // Make sure there's still at least <b>as</b> many owners <b>as</b> the number of signatures required.
    // This also <b>ensures</b> that there's at least one owner left <b>as</b> signature threshold must be &gt; 0.
    <b>assert</b>!(
        <a href="_length">vector::length</a>(owners) &gt;= multisig_account_resource.num_signatures_required,
        <a href="_invalid_state">error::invalid_state</a>(<a href="multisig_account.md#0x1_multisig_account_ENOT_ENOUGH_OWNERS">ENOT_ENOUGH_OWNERS</a>),
    );

    emit_event(&<b>mut</b> multisig_account_resource.remove_owners_events, <a href="multisig_account.md#0x1_multisig_account_RemoveOwnersEvent">RemoveOwnersEvent</a> { owners_removed });
}
</code></pre>



</details>

<a name="0x1_multisig_account_update_signatures_required"></a>

## Function `update_signatures_required`

Update the number of signatures required to execute transaction in the specified multisig account.

This can only be invoked by the multisig account itself, through the proposal flow.
Note that this function is not public so it can only be invoked directly instead of via a module or script. This
ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
maliciously alter the number of signatures required.


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_update_signatures_required">update_signatures_required</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, new_num_signatures_required: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_update_signatures_required">update_signatures_required</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, new_num_signatures_required: u64) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_address = address_of(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(multisig_address);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(multisig_address);
    // Short-circuit <b>if</b> the new number of signatures required is the same <b>as</b> before.
    // This avoids emitting an <a href="event.md#0x1_event">event</a>.
    <b>if</b> (multisig_account_resource.num_signatures_required == new_num_signatures_required) {
        <b>return</b>
    };
    <b>let</b> num_owners = <a href="_length">vector::length</a>(&multisig_account_resource.owners);
    <b>assert</b>!(
        new_num_signatures_required &gt; 0 && new_num_signatures_required &lt;= num_owners,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_SIGNATURES_REQUIRED">EINVALID_SIGNATURES_REQUIRED</a>),
    );

    <b>let</b> old_num_signatures_required = multisig_account_resource.num_signatures_required;
    multisig_account_resource.num_signatures_required = new_num_signatures_required;
    emit_event(
        &<b>mut</b> multisig_account_resource.update_signature_required_events,
        <a href="multisig_account.md#0x1_multisig_account_UpdateSignaturesRequiredEvent">UpdateSignaturesRequiredEvent</a> {
            old_num_signatures_required,
            new_num_signatures_required,
        }
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_update_metadata"></a>

## Function `update_metadata`

Allow the multisig account to update its own metadata. Note that this overrides the entire existing metadata.
If any attributes are not specified in the metadata, they will be removed!

This can only be invoked by the multisig account itself, through the proposal flow.
Note that this function is not public so it can only be invoked directly instead of via a module or script. This
ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
maliciously alter the number of signatures required.


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_update_metadata">update_metadata</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code>entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_update_metadata">update_metadata</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, keys: <a href="">vector</a>&lt;String&gt;, values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_update_metadata_internal">update_metadata_internal</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, keys, values, <b>true</b>);
}
</code></pre>



</details>

<a name="0x1_multisig_account_update_metadata_internal"></a>

## Function `update_metadata_internal`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_update_metadata_internal">update_metadata_internal</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>, keys: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, emit_event: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_update_metadata_internal">update_metadata_internal</a>(
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="">signer</a>,
    keys: <a href="">vector</a>&lt;String&gt;,
    values: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;,
    emit_event: bool,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> num_attributes = <a href="_length">vector::length</a>(&keys);
    <b>assert</b>!(
        num_attributes == <a href="_length">vector::length</a>(&values),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH">ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH</a>),
    );

    <b>let</b> multisig_address = address_of(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(multisig_address);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(multisig_address);
    <b>let</b> old_metadata = multisig_account_resource.metadata;
    multisig_account_resource.metadata = <a href="_create">simple_map::create</a>&lt;String, <a href="">vector</a>&lt;u8&gt;&gt;();
    <b>let</b> metadata = &<b>mut</b> multisig_account_resource.metadata;
    <b>let</b> i = 0;
    <b>while</b> (i &lt; num_attributes) {
        <b>let</b> key = *<a href="_borrow">vector::borrow</a>(&keys, i);
        <b>let</b> value = *<a href="_borrow">vector::borrow</a>(&values, i);
        <b>assert</b>!(
            !<a href="_contains_key">simple_map::contains_key</a>(metadata, &key),
            <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EDUPLICATE_METADATA_KEY">EDUPLICATE_METADATA_KEY</a>),
        );

        <a href="_add">simple_map::add</a>(metadata, key, value);
        i = i + 1;
    };

    <b>if</b> (emit_event) {
        emit_event(
            &<b>mut</b> multisig_account_resource.metadata_updated_events,
            <a href="multisig_account.md#0x1_multisig_account_MetadataUpdatedEvent">MetadataUpdatedEvent</a> {
                old_metadata,
                new_metadata: multisig_account_resource.metadata,
            }
        );
    };
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_transaction"></a>

## Function `create_transaction`

Create a multisig transaction, which will have one approval initially (from the creator).

@param target_function The target function to call such as 0x123::module_to_call::function_to_call.
@param args Vector of BCS-encoded argument values to invoke the target function with.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_transaction">create_transaction</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_transaction">create_transaction</a>(
    owner: &<a href="">signer</a>,
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>,
    payload: <a href="">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>assert</b>!(<a href="_length">vector::length</a>(&payload) &gt; 0, <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EPAYLOAD_CANNOT_BE_EMPTY">EPAYLOAD_CANNOT_BE_EMPTY</a>));

    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner, multisig_account_resource);

    <b>let</b> creator = address_of(owner);
    <b>let</b> transaction = <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a> {
        payload: <a href="_some">option::some</a>(payload),
        payload_hash: <a href="_none">option::none</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;(),
        votes: <a href="_create">simple_map::create</a>&lt;<b>address</b>, bool&gt;(),
        creator,
        creation_time_secs: now_seconds(),
    };
    <a href="multisig_account.md#0x1_multisig_account_add_transaction">add_transaction</a>(creator, multisig_account_resource, transaction);
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_transaction_with_hash"></a>

## Function `create_transaction_with_hash`

Create a multisig transaction with a transaction hash instead of the full payload.
This means the payload will be stored off chain for gas saving. Later, during execution, the executor will need
to provide the full payload, which will be validated against the hash stored on-chain.

@param function_hash The sha-256 hash of the function to invoke, e.g. 0x123::module_to_call::function_to_call.
@param args_hash The sha-256 hash of the function arguments - a concatenated vector of the bcs-encoded
function arguments.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_transaction_with_hash">create_transaction_with_hash</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, payload_hash: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_transaction_with_hash">create_transaction_with_hash</a>(
    owner: &<a href="">signer</a>,
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>,
    payload_hash: <a href="">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    // Payload <a href="">hash</a> is a sha3-256 <a href="">hash</a>, so it must be exactly 32 bytes.
    <b>assert</b>!(<a href="_length">vector::length</a>(&payload_hash) == 32, <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EINVALID_PAYLOAD_HASH">EINVALID_PAYLOAD_HASH</a>));

    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner, multisig_account_resource);

    <b>let</b> creator = address_of(owner);
    <b>let</b> transaction = <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a> {
        payload: <a href="_none">option::none</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;(),
        payload_hash: <a href="_some">option::some</a>(payload_hash),
        votes: <a href="_create">simple_map::create</a>&lt;<b>address</b>, bool&gt;(),
        creator,
        creation_time_secs: now_seconds(),
    };
    <a href="multisig_account.md#0x1_multisig_account_add_transaction">add_transaction</a>(creator, multisig_account_resource, transaction);
}
</code></pre>



</details>

<a name="0x1_multisig_account_approve_transaction"></a>

## Function `approve_transaction`

Approve a multisig transaction.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_approve_transaction">approve_transaction</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_approve_transaction">approve_transaction</a>(
    owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_vote_transanction">vote_transanction</a>(owner, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, sequence_number, <b>true</b>);
}
</code></pre>



</details>

<a name="0x1_multisig_account_reject_transaction"></a>

## Function `reject_transaction`

Reject a multisig transaction.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_reject_transaction">reject_transaction</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_reject_transaction">reject_transaction</a>(
    owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_vote_transanction">vote_transanction</a>(owner, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, sequence_number, <b>false</b>);
}
</code></pre>



</details>

<a name="0x1_multisig_account_vote_transanction"></a>

## Function `vote_transanction`

Generic function that can be used to either approve or reject a multisig transaction


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_vote_transanction">vote_transanction</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64, approved: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_vote_transanction">vote_transanction</a>(
    owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, sequence_number: u64, approved: bool) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner, multisig_account_resource);

    <b>assert</b>!(
        <a href="_contains">table::contains</a>(&multisig_account_resource.transactions, sequence_number),
        <a href="_not_found">error::not_found</a>(<a href="multisig_account.md#0x1_multisig_account_ETRANSACTION_NOT_FOUND">ETRANSACTION_NOT_FOUND</a>),
    );
    <b>let</b> transaction = <a href="_borrow_mut">table::borrow_mut</a>(&<b>mut</b> multisig_account_resource.transactions, sequence_number);
    <b>let</b> votes = &<b>mut</b> transaction.votes;
    <b>let</b> owner_addr = address_of(owner);

    <b>if</b> (<a href="_contains_key">simple_map::contains_key</a>(votes, &owner_addr)) {
        *<a href="_borrow_mut">simple_map::borrow_mut</a>(votes, &owner_addr) = approved;
    } <b>else</b> {
        <a href="_add">simple_map::add</a>(votes, owner_addr, approved);
    };

    emit_event(
        &<b>mut</b> multisig_account_resource.vote_events,
        <a href="multisig_account.md#0x1_multisig_account_VoteEvent">VoteEvent</a> {
            owner: owner_addr,
            sequence_number,
            approved,
        }
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_execute_rejected_transaction"></a>

## Function `execute_rejected_transaction`

Remove the next transaction if it has sufficient owner rejections.


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_execute_rejected_transaction">execute_rejected_transaction</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="multisig_account.md#0x1_multisig_account_execute_rejected_transaction">execute_rejected_transaction</a>(
    owner: &<a href="">signer</a>,
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner, multisig_account_resource);
    <b>let</b> sequence_number = multisig_account_resource.last_executed_sequence_number + 1;
    <b>assert</b>!(
        <a href="_contains">table::contains</a>(&multisig_account_resource.transactions, sequence_number),
        <a href="_not_found">error::not_found</a>(<a href="multisig_account.md#0x1_multisig_account_ETRANSACTION_NOT_FOUND">ETRANSACTION_NOT_FOUND</a>),
    );
    <b>let</b> (_, num_rejections) = <a href="multisig_account.md#0x1_multisig_account_remove_executed_transaction">remove_executed_transaction</a>(multisig_account_resource);
    <b>assert</b>!(
        num_rejections &gt;= multisig_account_resource.num_signatures_required,
        <a href="_invalid_state">error::invalid_state</a>(<a href="multisig_account.md#0x1_multisig_account_ENOT_ENOUGH_REJECTIONS">ENOT_ENOUGH_REJECTIONS</a>),
    );

    emit_event(
        &<b>mut</b> multisig_account_resource.execute_rejected_transaction_events,
        <a href="multisig_account.md#0x1_multisig_account_ExecuteRejectedTransactionEvent">ExecuteRejectedTransactionEvent</a> {
            sequence_number,
            num_rejections,
            executor: address_of(owner),
        }
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_validate_multisig_transaction"></a>

## Function `validate_multisig_transaction`

Called by the VM as part of transaction prologue, which is invoked during mempool transaction validation and as
the first step of transaction execution.

Transaction payload is optional if it's already stored on chain for the transaction.


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_validate_multisig_transaction">validate_multisig_transaction</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_validate_multisig_transaction">validate_multisig_transaction</a>(
    owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, payload: <a href="">vector</a>&lt;u8&gt;) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> multisig_account_resource = <b>borrow_global</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner, multisig_account_resource);
    <b>let</b> sequence_number = multisig_account_resource.last_executed_sequence_number + 1;
    <b>assert</b>!(
        <a href="_contains">table::contains</a>(&multisig_account_resource.transactions, sequence_number),
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_ETRANSACTION_NOT_FOUND">ETRANSACTION_NOT_FOUND</a>),
    );
    <b>let</b> transaction = <a href="_borrow">table::borrow</a>(&multisig_account_resource.transactions, sequence_number);
    <b>let</b> (num_approvals, _) = <a href="multisig_account.md#0x1_multisig_account_num_approvals_and_rejections">num_approvals_and_rejections</a>(&multisig_account_resource.owners, transaction);
    <b>assert</b>!(
        num_approvals &gt;= multisig_account_resource.num_signatures_required,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_ENOT_ENOUGH_APPROVALS">ENOT_ENOUGH_APPROVALS</a>),
    );

    // If the transaction payload is not stored on chain, verify that the provided payload matches the hashes stored
    // on chain.
    <b>if</b> (<a href="_is_some">option::is_some</a>(&transaction.payload_hash)) {
        <b>let</b> payload_hash = <a href="_borrow">option::borrow</a>(&transaction.payload_hash);
        <b>assert</b>!(
            sha3_256(payload) == *payload_hash,
            <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EPAYLOAD_DOES_NOT_MATCH_HASH">EPAYLOAD_DOES_NOT_MATCH_HASH</a>),
        );
    };
}
</code></pre>



</details>

<a name="0x1_multisig_account_successful_transaction_execution_cleanup"></a>

## Function `successful_transaction_execution_cleanup`

Post-execution cleanup for a successful multisig transaction execution.
This function is private so no other code can call this beside the VM itself as part of MultisigTransaction.


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_successful_transaction_execution_cleanup">successful_transaction_execution_cleanup</a>(executor: <b>address</b>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, transaction_payload: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_successful_transaction_execution_cleanup">successful_transaction_execution_cleanup</a>(
    executor: <b>address</b>,
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>,
    transaction_payload: <a href="">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> (num_approvals, _) = <a href="multisig_account.md#0x1_multisig_account_remove_executed_transaction">remove_executed_transaction</a>(multisig_account_resource);
    emit_event(
        &<b>mut</b> multisig_account_resource.execute_transaction_events,
        <a href="multisig_account.md#0x1_multisig_account_TransactionExecutionSucceededEvent">TransactionExecutionSucceededEvent</a> {
            sequence_number: multisig_account_resource.last_executed_sequence_number,
            transaction_payload,
            num_approvals,
            executor,
        }
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_failed_transaction_execution_cleanup"></a>

## Function `failed_transaction_execution_cleanup`

Post-execution cleanup for a failed multisig transaction execution.
This function is private so no other code can call this beside the VM itself as part of MultisigTransaction.


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_failed_transaction_execution_cleanup">failed_transaction_execution_cleanup</a>(executor: <b>address</b>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>, transaction_payload: <a href="">vector</a>&lt;u8&gt;, execution_error: <a href="multisig_account.md#0x1_multisig_account_ExecutionError">multisig_account::ExecutionError</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_failed_transaction_execution_cleanup">failed_transaction_execution_cleanup</a>(
    executor: <b>address</b>,
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>,
    transaction_payload: <a href="">vector</a>&lt;u8&gt;,
    execution_error: <a href="multisig_account.md#0x1_multisig_account_ExecutionError">ExecutionError</a>,
) <b>acquires</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a> {
    <b>let</b> multisig_account_resource = <b>borrow_global_mut</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>);
    <b>let</b> (num_approvals, _) = <a href="multisig_account.md#0x1_multisig_account_remove_executed_transaction">remove_executed_transaction</a>(multisig_account_resource);
    emit_event(
        &<b>mut</b> multisig_account_resource.transaction_execution_failed_events,
        <a href="multisig_account.md#0x1_multisig_account_TransactionExecutionFailedEvent">TransactionExecutionFailedEvent</a> {
            executor,
            sequence_number: multisig_account_resource.last_executed_sequence_number,
            transaction_payload,
            num_approvals,
            execution_error,
        }
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_remove_executed_transaction"></a>

## Function `remove_executed_transaction`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_remove_executed_transaction">remove_executed_transaction</a>(multisig_account_resource: &<b>mut</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">multisig_account::MultisigAccount</a>): (u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_remove_executed_transaction">remove_executed_transaction</a>(multisig_account_resource: &<b>mut</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>): (u64, u64) {
    <b>let</b> sequence_number = multisig_account_resource.last_executed_sequence_number + 1;
    <b>let</b> transaction = <a href="_remove">table::remove</a>(&<b>mut</b> multisig_account_resource.transactions, sequence_number);
    multisig_account_resource.last_executed_sequence_number = sequence_number;
    <a href="multisig_account.md#0x1_multisig_account_num_approvals_and_rejections">num_approvals_and_rejections</a>(&multisig_account_resource.owners, &transaction)
}
</code></pre>



</details>

<a name="0x1_multisig_account_add_transaction"></a>

## Function `add_transaction`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_add_transaction">add_transaction</a>(creator: <b>address</b>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<b>mut</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">multisig_account::MultisigAccount</a>, transaction: <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">multisig_account::MultisigTransaction</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_add_transaction">add_transaction</a>(creator: <b>address</b>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<b>mut</b> <a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>, transaction: <a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a>) {
    // The transaction creator also automatically votes for the transaction.
    <a href="_add">simple_map::add</a>(&<b>mut</b> transaction.votes, creator, <b>true</b>);

    <b>let</b> sequence_number = <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.next_sequence_number;
    <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.next_sequence_number = sequence_number + 1;
    <a href="_add">table::add</a>(&<b>mut</b> <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.transactions, sequence_number, transaction);
    emit_event(
        &<b>mut</b> <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.create_transaction_events,
        <a href="multisig_account.md#0x1_multisig_account_CreateTransactionEvent">CreateTransactionEvent</a> { creator, sequence_number, transaction },
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_multisig_account"></a>

## Function `create_multisig_account`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_multisig_account">create_multisig_account</a>(owner: &<a href="">signer</a>): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_multisig_account">create_multisig_account</a>(owner: &<a href="">signer</a>): (<a href="">signer</a>, SignerCapability) {
    <b>let</b> owner_nonce = <a href="account.md#0x1_account_get_sequence_number">account::get_sequence_number</a>(address_of(owner));
    <b>let</b> (multisig_signer, multisig_signer_cap) =
        <a href="account.md#0x1_account_create_resource_account">account::create_resource_account</a>(owner, <a href="multisig_account.md#0x1_multisig_account_create_multisig_account_seed">create_multisig_account_seed</a>(to_bytes(&owner_nonce)));
    // Register the <a href="account.md#0x1_account">account</a> <b>to</b> receive APT <b>as</b> this is not done by default <b>as</b> part of the resource <a href="account.md#0x1_account">account</a> creation
    // flow.
    <b>if</b> (!<a href="coin.md#0x1_coin_is_account_registered">coin::is_account_registered</a>&lt;AptosCoin&gt;(address_of(&multisig_signer))) {
        <a href="coin.md#0x1_coin_register">coin::register</a>&lt;AptosCoin&gt;(&multisig_signer);
    };

    (multisig_signer, multisig_signer_cap)
}
</code></pre>



</details>

<a name="0x1_multisig_account_create_multisig_account_seed"></a>

## Function `create_multisig_account_seed`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_multisig_account_seed">create_multisig_account_seed</a>(seed: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_create_multisig_account_seed">create_multisig_account_seed</a>(seed: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt; {
    // Generate a seed that will be used <b>to</b> create the resource <a href="account.md#0x1_account">account</a> that hosts the multisig <a href="account.md#0x1_account">account</a>.
    <b>let</b> multisig_account_seed = <a href="_empty">vector::empty</a>&lt;u8&gt;();
    <a href="_append">vector::append</a>(&<b>mut</b> multisig_account_seed, <a href="multisig_account.md#0x1_multisig_account_DOMAIN_SEPARATOR">DOMAIN_SEPARATOR</a>);
    <a href="_append">vector::append</a>(&<b>mut</b> multisig_account_seed, seed);

    multisig_account_seed
}
</code></pre>



</details>

<a name="0x1_multisig_account_validate_owners"></a>

## Function `validate_owners`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_validate_owners">validate_owners</a>(owners: &<a href="">vector</a>&lt;<b>address</b>&gt;, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_validate_owners">validate_owners</a>(owners: &<a href="">vector</a>&lt;<b>address</b>&gt;, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>) {
    <b>let</b> distinct_owners: <a href="">vector</a>&lt;<b>address</b>&gt; = <a href="">vector</a>[];
    vector::for_each_ref(owners, |owner| {
        <b>let</b> owner = *owner;
        <b>assert</b>!(owner != <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>, <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF">EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF</a>));
        <b>let</b> (found, _) = <a href="_index_of">vector::index_of</a>(&distinct_owners, &owner);
        <b>assert</b>!(!found, <a href="_invalid_argument">error::invalid_argument</a>(<a href="multisig_account.md#0x1_multisig_account_EDUPLICATE_OWNER">EDUPLICATE_OWNER</a>));
        <a href="_push_back">vector::push_back</a>(&<b>mut</b> distinct_owners, owner);
    });
}
</code></pre>



</details>

<a name="0x1_multisig_account_assert_is_owner"></a>

## Function `assert_is_owner`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">multisig_account::MultisigAccount</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_assert_is_owner">assert_is_owner</a>(owner: &<a href="">signer</a>, <a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: &<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>) {
    <b>assert</b>!(
        <a href="_contains">vector::contains</a>(&<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>.owners, &address_of(owner)),
        <a href="_permission_denied">error::permission_denied</a>(<a href="multisig_account.md#0x1_multisig_account_ENOT_OWNER">ENOT_OWNER</a>),
    );
}
</code></pre>



</details>

<a name="0x1_multisig_account_num_approvals_and_rejections"></a>

## Function `num_approvals_and_rejections`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_num_approvals_and_rejections">num_approvals_and_rejections</a>(owners: &<a href="">vector</a>&lt;<b>address</b>&gt;, transaction: &<a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">multisig_account::MultisigTransaction</a>): (u64, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_num_approvals_and_rejections">num_approvals_and_rejections</a>(owners: &<a href="">vector</a>&lt;<b>address</b>&gt;, transaction: &<a href="multisig_account.md#0x1_multisig_account_MultisigTransaction">MultisigTransaction</a>): (u64, u64) {
    <b>let</b> num_approvals = 0;
    <b>let</b> num_rejections = 0;

    <b>let</b> votes = &transaction.votes;
    vector::for_each_ref(owners, |owner| {
        <b>if</b> (<a href="_contains_key">simple_map::contains_key</a>(votes, owner)) {
            <b>if</b> (*<a href="_borrow">simple_map::borrow</a>(votes, owner)) {
                num_approvals = num_approvals + 1;
            } <b>else</b> {
                num_rejections = num_rejections + 1;
            };
        }
    });

    (num_approvals, num_rejections)
}
</code></pre>



</details>

<a name="0x1_multisig_account_assert_multisig_account_exists"></a>

## Function `assert_multisig_account_exists`



<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="multisig_account.md#0x1_multisig_account_assert_multisig_account_exists">assert_multisig_account_exists</a>(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>: <b>address</b>) {
    <b>assert</b>!(<b>exists</b>&lt;<a href="multisig_account.md#0x1_multisig_account_MultisigAccount">MultisigAccount</a>&gt;(<a href="multisig_account.md#0x1_multisig_account">multisig_account</a>), <a href="_invalid_state">error::invalid_state</a>(<a href="multisig_account.md#0x1_multisig_account_EACCOUNT_NOT_MULTISIG">EACCOUNT_NOT_MULTISIG</a>));
}
</code></pre>



</details>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
