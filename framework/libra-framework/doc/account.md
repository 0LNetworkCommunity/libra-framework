
<a name="0x1_account"></a>

# Module `0x1::account`



-  [Resource `Account`](#0x1_account_Account)
-  [Struct `KeyRotationEvent`](#0x1_account_KeyRotationEvent)
-  [Struct `CoinRegisterEvent`](#0x1_account_CoinRegisterEvent)
-  [Struct `CapabilityOffer`](#0x1_account_CapabilityOffer)
-  [Struct `RotationCapability`](#0x1_account_RotationCapability)
-  [Struct `SignerCapability`](#0x1_account_SignerCapability)
-  [Resource `OriginatingAddress`](#0x1_account_OriginatingAddress)
-  [Struct `RotationProofChallenge`](#0x1_account_RotationProofChallenge)
-  [Struct `RotationCapabilityOfferProofChallenge`](#0x1_account_RotationCapabilityOfferProofChallenge)
-  [Struct `SignerCapabilityOfferProofChallenge`](#0x1_account_SignerCapabilityOfferProofChallenge)
-  [Struct `RotationCapabilityOfferProofChallengeV2`](#0x1_account_RotationCapabilityOfferProofChallengeV2)
-  [Struct `SignerCapabilityOfferProofChallengeV2`](#0x1_account_SignerCapabilityOfferProofChallengeV2)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_account_initialize)
-  [Function `vm_create_account`](#0x1_account_vm_create_account)
-  [Function `ol_create_account_with_auth`](#0x1_account_ol_create_account_with_auth)
-  [Function `create_account`](#0x1_account_create_account)
-  [Function `create_account_unchecked`](#0x1_account_create_account_unchecked)
-  [Function `exists_at`](#0x1_account_exists_at)
-  [Function `get_guid_next_creation_num`](#0x1_account_get_guid_next_creation_num)
-  [Function `get_sequence_number`](#0x1_account_get_sequence_number)
-  [Function `increment_sequence_number`](#0x1_account_increment_sequence_number)
-  [Function `get_authentication_key`](#0x1_account_get_authentication_key)
-  [Function `rotate_authentication_key_internal`](#0x1_account_rotate_authentication_key_internal)
-  [Function `rotate_authentication_key`](#0x1_account_rotate_authentication_key)
-  [Function `rotate_authentication_key_with_rotation_capability`](#0x1_account_rotate_authentication_key_with_rotation_capability)
-  [Function `offer_rotation_capability`](#0x1_account_offer_rotation_capability)
-  [Function `revoke_rotation_capability`](#0x1_account_revoke_rotation_capability)
-  [Function `revoke_any_rotation_capability`](#0x1_account_revoke_any_rotation_capability)
-  [Function `offer_signer_capability`](#0x1_account_offer_signer_capability)
-  [Function `is_signer_capability_offered`](#0x1_account_is_signer_capability_offered)
-  [Function `get_signer_capability_offer_for`](#0x1_account_get_signer_capability_offer_for)
-  [Function `revoke_signer_capability`](#0x1_account_revoke_signer_capability)
-  [Function `revoke_any_signer_capability`](#0x1_account_revoke_any_signer_capability)
-  [Function `create_authorized_signer`](#0x1_account_create_authorized_signer)
-  [Function `assert_valid_rotation_proof_signature_and_get_auth_key`](#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key)
-  [Function `update_auth_key_and_originating_address_table`](#0x1_account_update_auth_key_and_originating_address_table)
-  [Function `create_resource_address`](#0x1_account_create_resource_address)
-  [Function `create_resource_account`](#0x1_account_create_resource_account)
-  [Function `create_framework_reserved_account`](#0x1_account_create_framework_reserved_account)
-  [Function `create_guid`](#0x1_account_create_guid)
-  [Function `new_event_handle`](#0x1_account_new_event_handle)
-  [Function `register_coin`](#0x1_account_register_coin)
-  [Function `create_signer_with_capability`](#0x1_account_create_signer_with_capability)
-  [Function `get_signer_capability_address`](#0x1_account_get_signer_capability_address)
-  [Function `verify_signed_message`](#0x1_account_verify_signed_message)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `create_account`](#@Specification_1_create_account)
    -  [Function `create_account_unchecked`](#@Specification_1_create_account_unchecked)
    -  [Function `get_guid_next_creation_num`](#@Specification_1_get_guid_next_creation_num)
    -  [Function `get_sequence_number`](#@Specification_1_get_sequence_number)
    -  [Function `increment_sequence_number`](#@Specification_1_increment_sequence_number)
    -  [Function `get_authentication_key`](#@Specification_1_get_authentication_key)
    -  [Function `rotate_authentication_key_internal`](#@Specification_1_rotate_authentication_key_internal)
    -  [Function `rotate_authentication_key`](#@Specification_1_rotate_authentication_key)
    -  [Function `rotate_authentication_key_with_rotation_capability`](#@Specification_1_rotate_authentication_key_with_rotation_capability)
    -  [Function `offer_rotation_capability`](#@Specification_1_offer_rotation_capability)
    -  [Function `revoke_rotation_capability`](#@Specification_1_revoke_rotation_capability)
    -  [Function `revoke_any_rotation_capability`](#@Specification_1_revoke_any_rotation_capability)
    -  [Function `offer_signer_capability`](#@Specification_1_offer_signer_capability)
    -  [Function `is_signer_capability_offered`](#@Specification_1_is_signer_capability_offered)
    -  [Function `get_signer_capability_offer_for`](#@Specification_1_get_signer_capability_offer_for)
    -  [Function `revoke_signer_capability`](#@Specification_1_revoke_signer_capability)
    -  [Function `revoke_any_signer_capability`](#@Specification_1_revoke_any_signer_capability)
    -  [Function `create_authorized_signer`](#@Specification_1_create_authorized_signer)
    -  [Function `assert_valid_rotation_proof_signature_and_get_auth_key`](#@Specification_1_assert_valid_rotation_proof_signature_and_get_auth_key)
    -  [Function `update_auth_key_and_originating_address_table`](#@Specification_1_update_auth_key_and_originating_address_table)
    -  [Function `create_resource_address`](#@Specification_1_create_resource_address)
    -  [Function `create_resource_account`](#@Specification_1_create_resource_account)
    -  [Function `create_framework_reserved_account`](#@Specification_1_create_framework_reserved_account)
    -  [Function `create_guid`](#@Specification_1_create_guid)
    -  [Function `new_event_handle`](#@Specification_1_new_event_handle)
    -  [Function `register_coin`](#@Specification_1_register_coin)
    -  [Function `create_signer_with_capability`](#@Specification_1_create_signer_with_capability)
    -  [Function `verify_signed_message`](#@Specification_1_verify_signed_message)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="">0x1::ed25519</a>;
<b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="">0x1::from_bcs</a>;
<b>use</b> <a href="guid.md#0x1_guid">0x1::guid</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::multi_ed25519</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="">0x1::table</a>;
<b>use</b> <a href="">0x1::type_info</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a name="0x1_account_Account"></a>

## Resource `Account`

Resource representing an account.


<pre><code><b>struct</b> <a href="account.md#0x1_account_Account">Account</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>guid_creation_num: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>coin_register_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="account.md#0x1_account_CoinRegisterEvent">account::CoinRegisterEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>key_rotation_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="account.md#0x1_account_KeyRotationEvent">account::KeyRotationEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>rotation_capability_offer: <a href="account.md#0x1_account_CapabilityOffer">account::CapabilityOffer</a>&lt;<a href="account.md#0x1_account_RotationCapability">account::RotationCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>signer_capability_offer: <a href="account.md#0x1_account_CapabilityOffer">account::CapabilityOffer</a>&lt;<a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_KeyRotationEvent"></a>

## Struct `KeyRotationEvent`



<pre><code><b>struct</b> <a href="account.md#0x1_account_KeyRotationEvent">KeyRotationEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>old_authentication_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>new_authentication_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_CoinRegisterEvent"></a>

## Struct `CoinRegisterEvent`



<pre><code><b>struct</b> <a href="account.md#0x1_account_CoinRegisterEvent">CoinRegisterEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="">type_info</a>: <a href="_TypeInfo">type_info::TypeInfo</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_CapabilityOffer"></a>

## Struct `CapabilityOffer`



<pre><code><b>struct</b> <a href="account.md#0x1_account_CapabilityOffer">CapabilityOffer</a>&lt;T&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>for: <a href="_Option">option::Option</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_RotationCapability"></a>

## Struct `RotationCapability`



<pre><code><b>struct</b> <a href="account.md#0x1_account_RotationCapability">RotationCapability</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="account.md#0x1_account">account</a>: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_SignerCapability"></a>

## Struct `SignerCapability`



<pre><code><b>struct</b> <a href="account.md#0x1_account_SignerCapability">SignerCapability</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="account.md#0x1_account">account</a>: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_OriginatingAddress"></a>

## Resource `OriginatingAddress`

It is easy to fetch the authentication key of an address by simply reading it from the <code><a href="account.md#0x1_account_Account">Account</a></code> struct at that address.
The table in this struct makes it possible to do a reverse lookup: it maps an authentication key, to the address of the account which has that authentication key set.

This mapping is needed when recovering wallets for accounts whose authentication key has been rotated.

For example, imagine a freshly-created wallet with address <code>a</code> and thus also with authentication key <code>a</code>, derived from a PK <code>pk_a</code> with corresponding SK <code>sk_a</code>.
It is easy to recover such a wallet given just the secret key <code>sk_a</code>, since the PK can be derived from the SK, the authentication key can then be derived from the PK, and the address equals the authentication key (since there was no key rotation).

However, if such a wallet rotates its authentication key to <code>b</code> derived from a different PK <code>pk_b</code> with SK <code>sk_b</code>, how would account recovery work?
The recovered address would no longer be 'a'; it would be <code>b</code>, which is incorrect.
This struct solves this problem by mapping the new authentication key <code>b</code> to the original address <code>a</code> and thus helping the wallet software during recovery find the correct address.


<pre><code><b>struct</b> <a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>address_map: <a href="_Table">table::Table</a>&lt;<b>address</b>, <b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_RotationProofChallenge"></a>

## Struct `RotationProofChallenge`

This structs stores the challenge message that should be signed during key rotation. First, this struct is
signed by the account owner's current public key, which proves possession of a capability to rotate the key.
Second, this struct is signed by the new public key that the account owner wants to rotate to, which proves
knowledge of this new public key's associated secret key. These two signatures cannot be replayed in another
context because they include the TXN's unique sequence number.


<pre><code><b>struct</b> <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a> <b>has</b> <b>copy</b>, drop
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
<code>originator: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>current_auth_key: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>new_public_key: <a href="">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_RotationCapabilityOfferProofChallenge"></a>

## Struct `RotationCapabilityOfferProofChallenge`

Deprecated struct - newest version is <code><a href="account.md#0x1_account_RotationCapabilityOfferProofChallengeV2">RotationCapabilityOfferProofChallengeV2</a></code>


<pre><code><b>struct</b> <a href="account.md#0x1_account_RotationCapabilityOfferProofChallenge">RotationCapabilityOfferProofChallenge</a> <b>has</b> drop
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
<code>recipient_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_SignerCapabilityOfferProofChallenge"></a>

## Struct `SignerCapabilityOfferProofChallenge`

Deprecated struct - newest version is <code><a href="account.md#0x1_account_SignerCapabilityOfferProofChallengeV2">SignerCapabilityOfferProofChallengeV2</a></code>


<pre><code><b>struct</b> <a href="account.md#0x1_account_SignerCapabilityOfferProofChallenge">SignerCapabilityOfferProofChallenge</a> <b>has</b> drop
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
<code>recipient_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_RotationCapabilityOfferProofChallengeV2"></a>

## Struct `RotationCapabilityOfferProofChallengeV2`

This struct stores the challenge message that should be signed by the source account, when the source account
is delegating its rotation capability to the <code>recipient_address</code>.
This V2 struct adds the <code><a href="chain_id.md#0x1_chain_id">chain_id</a></code> and <code>source_address</code> to the challenge message, which prevents replaying the challenge message.


<pre><code><b>struct</b> <a href="account.md#0x1_account_RotationCapabilityOfferProofChallengeV2">RotationCapabilityOfferProofChallengeV2</a> <b>has</b> drop
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
<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>source_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>recipient_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_account_SignerCapabilityOfferProofChallengeV2"></a>

## Struct `SignerCapabilityOfferProofChallengeV2`



<pre><code><b>struct</b> <a href="account.md#0x1_account_SignerCapabilityOfferProofChallengeV2">SignerCapabilityOfferProofChallengeV2</a> <b>has</b> <b>copy</b>, drop
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
<code>source_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>recipient_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_account_MAX_U64"></a>



<pre><code><b>const</b> <a href="account.md#0x1_account_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_account_DERIVE_RESOURCE_ACCOUNT_SCHEME"></a>

Scheme identifier used when hashing an account's address together with a seed to derive the address (not the
authentication key) of a resource account. This is an abuse of the notion of a scheme identifier which, for now,
serves to domain separate hashes used to derive resource account addresses from hashes used to derive
authentication keys. Without such separation, an adversary could create (and get a signer for) a resource account
whose address matches an existing address of a MultiEd25519 wallet.


<pre><code><b>const</b> <a href="account.md#0x1_account_DERIVE_RESOURCE_ACCOUNT_SCHEME">DERIVE_RESOURCE_ACCOUNT_SCHEME</a>: u8 = 255;
</code></pre>



<a name="0x1_account_EACCOUNT_ALREADY_EXISTS"></a>

Account already exists


<pre><code><b>const</b> <a href="account.md#0x1_account_EACCOUNT_ALREADY_EXISTS">EACCOUNT_ALREADY_EXISTS</a>: u64 = 1;
</code></pre>



<a name="0x1_account_EACCOUNT_ALREADY_USED"></a>

An attempt to create a resource account on an account that has a committed transaction


<pre><code><b>const</b> <a href="account.md#0x1_account_EACCOUNT_ALREADY_USED">EACCOUNT_ALREADY_USED</a>: u64 = 16;
</code></pre>



<a name="0x1_account_EACCOUNT_DOES_NOT_EXIST"></a>

Account does not exist


<pre><code><b>const</b> <a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>: u64 = 2;
</code></pre>



<a name="0x1_account_ECANNOT_RESERVED_ADDRESS"></a>

Cannot create account because address is reserved


<pre><code><b>const</b> <a href="account.md#0x1_account_ECANNOT_RESERVED_ADDRESS">ECANNOT_RESERVED_ADDRESS</a>: u64 = 5;
</code></pre>



<a name="0x1_account_ED25519_SCHEME"></a>

Scheme identifier for Ed25519 signatures used to derive authentication keys for Ed25519 public keys.


<pre><code><b>const</b> <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a>: u8 = 0;
</code></pre>



<a name="0x1_account_EEXCEEDED_MAX_GUID_CREATION_NUM"></a>



<pre><code><b>const</b> <a href="account.md#0x1_account_EEXCEEDED_MAX_GUID_CREATION_NUM">EEXCEEDED_MAX_GUID_CREATION_NUM</a>: u64 = 20;
</code></pre>



<a name="0x1_account_EINVALID_ACCEPT_ROTATION_CAPABILITY"></a>

The caller does not have a valid rotation capability offer from the other account


<pre><code><b>const</b> <a href="account.md#0x1_account_EINVALID_ACCEPT_ROTATION_CAPABILITY">EINVALID_ACCEPT_ROTATION_CAPABILITY</a>: u64 = 10;
</code></pre>



<a name="0x1_account_EINVALID_ORIGINATING_ADDRESS"></a>

Abort the transaction if the expected originating address is different from the originating addres on-chain


<pre><code><b>const</b> <a href="account.md#0x1_account_EINVALID_ORIGINATING_ADDRESS">EINVALID_ORIGINATING_ADDRESS</a>: u64 = 13;
</code></pre>



<a name="0x1_account_EINVALID_PROOF_OF_KNOWLEDGE"></a>

Specified proof of knowledge required to prove ownership of a public key is invalid


<pre><code><b>const</b> <a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>: u64 = 8;
</code></pre>



<a name="0x1_account_EINVALID_SCHEME"></a>

Specified scheme required to proceed with the smart contract operation - can only be ED25519_SCHEME(0) OR MULTI_ED25519_SCHEME(1)


<pre><code><b>const</b> <a href="account.md#0x1_account_EINVALID_SCHEME">EINVALID_SCHEME</a>: u64 = 12;
</code></pre>



<a name="0x1_account_EMALFORMED_AUTHENTICATION_KEY"></a>

The provided authentication key has an invalid length


<pre><code><b>const</b> <a href="account.md#0x1_account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>: u64 = 4;
</code></pre>



<a name="0x1_account_ENO_CAPABILITY"></a>

The caller does not have a digital-signature-based capability to call this function


<pre><code><b>const</b> <a href="account.md#0x1_account_ENO_CAPABILITY">ENO_CAPABILITY</a>: u64 = 9;
</code></pre>



<a name="0x1_account_ENO_SIGNER_CAPABILITY_OFFERED"></a>



<pre><code><b>const</b> <a href="account.md#0x1_account_ENO_SIGNER_CAPABILITY_OFFERED">ENO_SIGNER_CAPABILITY_OFFERED</a>: u64 = 19;
</code></pre>



<a name="0x1_account_ENO_SUCH_ROTATION_CAPABILITY_OFFER"></a>

The specified rotation capablity offer does not exist at the specified offerer address


<pre><code><b>const</b> <a href="account.md#0x1_account_ENO_SUCH_ROTATION_CAPABILITY_OFFER">ENO_SUCH_ROTATION_CAPABILITY_OFFER</a>: u64 = 18;
</code></pre>



<a name="0x1_account_ENO_SUCH_SIGNER_CAPABILITY"></a>

The signer capability offer doesn't exist at the given address


<pre><code><b>const</b> <a href="account.md#0x1_account_ENO_SUCH_SIGNER_CAPABILITY">ENO_SUCH_SIGNER_CAPABILITY</a>: u64 = 14;
</code></pre>



<a name="0x1_account_ENO_VALID_FRAMEWORK_RESERVED_ADDRESS"></a>

Address to create is not a valid reserved address for Aptos framework


<pre><code><b>const</b> <a href="account.md#0x1_account_ENO_VALID_FRAMEWORK_RESERVED_ADDRESS">ENO_VALID_FRAMEWORK_RESERVED_ADDRESS</a>: u64 = 11;
</code></pre>



<a name="0x1_account_EOFFERER_ADDRESS_DOES_NOT_EXIST"></a>

Offerer address doesn't exist


<pre><code><b>const</b> <a href="account.md#0x1_account_EOFFERER_ADDRESS_DOES_NOT_EXIST">EOFFERER_ADDRESS_DOES_NOT_EXIST</a>: u64 = 17;
</code></pre>



<a name="0x1_account_EOUT_OF_GAS"></a>

Transaction exceeded its allocated max gas


<pre><code><b>const</b> <a href="account.md#0x1_account_EOUT_OF_GAS">EOUT_OF_GAS</a>: u64 = 6;
</code></pre>



<a name="0x1_account_ERESOURCE_ACCCOUNT_EXISTS"></a>

An attempt to create a resource account on a claimed account


<pre><code><b>const</b> <a href="account.md#0x1_account_ERESOURCE_ACCCOUNT_EXISTS">ERESOURCE_ACCCOUNT_EXISTS</a>: u64 = 15;
</code></pre>



<a name="0x1_account_ESEQUENCE_NUMBER_TOO_BIG"></a>

Sequence number exceeds the maximum value for a u64


<pre><code><b>const</b> <a href="account.md#0x1_account_ESEQUENCE_NUMBER_TOO_BIG">ESEQUENCE_NUMBER_TOO_BIG</a>: u64 = 3;
</code></pre>



<a name="0x1_account_EWRONG_CURRENT_PUBLIC_KEY"></a>

Specified current public key is not correct


<pre><code><b>const</b> <a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>: u64 = 7;
</code></pre>



<a name="0x1_account_MAX_GUID_CREATION_NUM"></a>

Explicitly separate the GUID space between Object and Account to prevent accidental overlap.


<pre><code><b>const</b> <a href="account.md#0x1_account_MAX_GUID_CREATION_NUM">MAX_GUID_CREATION_NUM</a>: u64 = 1125899906842624;
</code></pre>



<a name="0x1_account_MULTI_ED25519_SCHEME"></a>

Scheme identifier for MultiEd25519 signatures used to derive authentication keys for MultiEd25519 public keys.


<pre><code><b>const</b> <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>: u8 = 1;
</code></pre>



<a name="0x1_account_ZERO_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="account.md#0x1_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a name="0x1_account_initialize"></a>

## Function `initialize`

Only called during genesis to initialize system resources for this module.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_initialize">initialize</a>(aptos_framework: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_initialize">initialize</a>(aptos_framework: &<a href="">signer</a>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_aptos_framework">system_addresses::assert_aptos_framework</a>(aptos_framework);
    <b>move_to</b>(aptos_framework, <a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a> {
        address_map: <a href="_new">table::new</a>(),
    });
}
</code></pre>



</details>

<a name="0x1_account_vm_create_account"></a>

## Function `vm_create_account`

0L: creates account from VM signer, for use at genesis, e.g. hard fork migration.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_vm_create_account">vm_create_account</a>(root: &<a href="">signer</a>, new_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> (<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_vm_create_account">vm_create_account</a>(root: &<a href="">signer</a>, new_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="">signer</a> {
  <a href="system_addresses.md#0x1_system_addresses_assert_ol">system_addresses::assert_ol</a>(root);
  <a href="account.md#0x1_account_ol_create_account_with_auth">ol_create_account_with_auth</a>(new_address, authentication_key)
}
</code></pre>



</details>

<a name="0x1_account_ol_create_account_with_auth"></a>

## Function `ol_create_account_with_auth`

0L: account creation uses the address of the sender as the address of the new account.


<pre><code><b>fun</b> <a href="account.md#0x1_account_ol_create_account_with_auth">ol_create_account_with_auth</a>(new_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="account.md#0x1_account_ol_create_account_with_auth">ol_create_account_with_auth</a>(new_address: <b>address</b>, authentication_key: <a href="">vector</a>&lt;u8&gt;): <a href="">signer</a> {
    <b>let</b> new_account = <a href="create_signer.md#0x1_create_signer">create_signer</a>(new_address);

    // there cannot be an <a href="account.md#0x1_account_Account">Account</a> resource under new_addr already.
    <b>assert</b>!(!<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(new_address), <a href="_already_exists">error::already_exists</a>(<a href="account.md#0x1_account_EACCOUNT_ALREADY_EXISTS">EACCOUNT_ALREADY_EXISTS</a>));


    // NOTE: @core_resources gets created via a `create_account` call, so we do not <b>include</b> it below.
    <b>assert</b>!(
        new_address != @vm_reserved && new_address != @aptos_framework && new_address != @aptos_token,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_ECANNOT_RESERVED_ADDRESS">ECANNOT_RESERVED_ADDRESS</a>)
    );

    <b>assert</b>!(
        <a href="_length">vector::length</a>(&authentication_key) == 32,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );

    // TODO: duplicated <b>with</b> _unchecked() below
    <b>let</b> guid_creation_num = 0;

    <b>let</b> guid_for_coin = <a href="guid.md#0x1_guid_create">guid::create</a>(new_address, &<b>mut</b> guid_creation_num);
    <b>let</b> coin_register_events = <a href="event.md#0x1_event_new_event_handle">event::new_event_handle</a>&lt;<a href="account.md#0x1_account_CoinRegisterEvent">CoinRegisterEvent</a>&gt;(guid_for_coin);

    <b>let</b> guid_for_rotation = <a href="guid.md#0x1_guid_create">guid::create</a>(new_address, &<b>mut</b> guid_creation_num);
    <b>let</b> key_rotation_events = <a href="event.md#0x1_event_new_event_handle">event::new_event_handle</a>&lt;<a href="account.md#0x1_account_KeyRotationEvent">KeyRotationEvent</a>&gt;(guid_for_rotation);

    <b>move_to</b>(
        &new_account,
        <a href="account.md#0x1_account_Account">Account</a> {
            authentication_key,
            sequence_number: 0,
            guid_creation_num,
            coin_register_events,
            key_rotation_events,
            rotation_capability_offer: <a href="account.md#0x1_account_CapabilityOffer">CapabilityOffer</a> { for: <a href="_none">option::none</a>() },
            signer_capability_offer: <a href="account.md#0x1_account_CapabilityOffer">CapabilityOffer</a> { for: <a href="_none">option::none</a>() },
        }
    );

    new_account
}
</code></pre>



</details>

<a name="0x1_account_create_account"></a>

## Function `create_account`

Publishes a new <code><a href="account.md#0x1_account_Account">Account</a></code> resource under <code>new_address</code>. A signer representing <code>new_address</code>
is returned. This way, the caller of this function can publish additional resources under
<code>new_address</code>.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_create_account">create_account</a>(new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_create_account">create_account</a>(new_address: <b>address</b>): <a href="">signer</a> {
    // there cannot be an <a href="account.md#0x1_account_Account">Account</a> resource under new_addr already.
    <b>assert</b>!(!<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(new_address), <a href="_already_exists">error::already_exists</a>(<a href="account.md#0x1_account_EACCOUNT_ALREADY_EXISTS">EACCOUNT_ALREADY_EXISTS</a>));

    // NOTE: @core_resources gets created via a `create_account` call, so we do not <b>include</b> it below.
    <b>assert</b>!(
        new_address != @vm_reserved && new_address != @aptos_framework && new_address != @aptos_token,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_ECANNOT_RESERVED_ADDRESS">ECANNOT_RESERVED_ADDRESS</a>)
    );

    <a href="account.md#0x1_account_create_account_unchecked">create_account_unchecked</a>(new_address)
}
</code></pre>



</details>

<a name="0x1_account_create_account_unchecked"></a>

## Function `create_account_unchecked`



<pre><code><b>fun</b> <a href="account.md#0x1_account_create_account_unchecked">create_account_unchecked</a>(new_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="account.md#0x1_account_create_account_unchecked">create_account_unchecked</a>(new_address: <b>address</b>): <a href="">signer</a> {
    <b>let</b> new_account = <a href="create_signer.md#0x1_create_signer">create_signer</a>(new_address);
    <b>let</b> authentication_key = <a href="_to_bytes">bcs::to_bytes</a>(&new_address);
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&authentication_key) == 32,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );

    <b>let</b> guid_creation_num = 0;

    <b>let</b> guid_for_coin = <a href="guid.md#0x1_guid_create">guid::create</a>(new_address, &<b>mut</b> guid_creation_num);
    <b>let</b> coin_register_events = <a href="event.md#0x1_event_new_event_handle">event::new_event_handle</a>&lt;<a href="account.md#0x1_account_CoinRegisterEvent">CoinRegisterEvent</a>&gt;(guid_for_coin);

    <b>let</b> guid_for_rotation = <a href="guid.md#0x1_guid_create">guid::create</a>(new_address, &<b>mut</b> guid_creation_num);
    <b>let</b> key_rotation_events = <a href="event.md#0x1_event_new_event_handle">event::new_event_handle</a>&lt;<a href="account.md#0x1_account_KeyRotationEvent">KeyRotationEvent</a>&gt;(guid_for_rotation);

    <b>move_to</b>(
        &new_account,
        <a href="account.md#0x1_account_Account">Account</a> {
            authentication_key,
            sequence_number: 0,
            guid_creation_num,
            coin_register_events,
            key_rotation_events,
            rotation_capability_offer: <a href="account.md#0x1_account_CapabilityOffer">CapabilityOffer</a> { for: <a href="_none">option::none</a>() },
            signer_capability_offer: <a href="account.md#0x1_account_CapabilityOffer">CapabilityOffer</a> { for: <a href="_none">option::none</a>() },
        }
    );

    new_account
}
</code></pre>



</details>

<a name="0x1_account_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_exists_at">exists_at</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_exists_at">exists_at</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_account_get_guid_next_creation_num"></a>

## Function `get_guid_next_creation_num`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_guid_next_creation_num">get_guid_next_creation_num</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_guid_next_creation_num">get_guid_next_creation_num</a>(addr: <b>address</b>): u64 <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).guid_creation_num
}
</code></pre>



</details>

<a name="0x1_account_get_sequence_number"></a>

## Function `get_sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_sequence_number">get_sequence_number</a>(addr: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_sequence_number">get_sequence_number</a>(addr: <b>address</b>): u64 <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).sequence_number
}
</code></pre>



</details>

<a name="0x1_account_increment_sequence_number"></a>

## Function `increment_sequence_number`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_increment_sequence_number">increment_sequence_number</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_increment_sequence_number">increment_sequence_number</a>(addr: <b>address</b>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> sequence_number = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).sequence_number;

    <b>assert</b>!(
        (*sequence_number <b>as</b> u128) &lt; <a href="account.md#0x1_account_MAX_U64">MAX_U64</a>,
        <a href="_out_of_range">error::out_of_range</a>(<a href="account.md#0x1_account_ESEQUENCE_NUMBER_TOO_BIG">ESEQUENCE_NUMBER_TOO_BIG</a>)
    );

    *sequence_number = *sequence_number + 1;
}
</code></pre>



</details>

<a name="0x1_account_get_authentication_key"></a>

## Function `get_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_authentication_key">get_authentication_key</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_authentication_key">get_authentication_key</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt; <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_account_rotate_authentication_key_internal"></a>

## Function `rotate_authentication_key_internal`

This function is used to rotate a resource account's authentication key to 0, so that no private key can control
the resource account.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, new_auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, new_auth_key: <a href="">vector</a>&lt;u8&gt;) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(addr), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>));
    <b>assert</b>!(
        <a href="_length">vector::length</a>(&new_auth_key) == 32,
        <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    account_resource.authentication_key = new_auth_key;
}
</code></pre>



</details>

<a name="0x1_account_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

Generic authentication key rotation function that allows the user to rotate their authentication key from any scheme to any scheme.
To authorize the rotation, we need two signatures:
- the first signature <code>cap_rotate_key</code> refers to the signature by the account owner's current key on a valid <code><a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a></code>,
demonstrating that the user intends to and has the capability to rotate the authentication key of this account;
- the second signature <code>cap_update_table</code> refers to the signature by the new key (that the account owner wants to rotate to) on a
valid <code><a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a></code>, demonstrating that the user owns the new private key, and has the authority to update the
<code><a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a></code> map with the new address mapping <code>&lt;new_address, originating_address&gt;</code>.
To verify these two signatures, we need their corresponding public key and public key scheme: we use <code>from_scheme</code> and <code>from_public_key_bytes</code>
to verify <code>cap_rotate_key</code>, and <code>to_scheme</code> and <code>to_public_key_bytes</code> to verify <code>cap_update_table</code>.
A scheme of 0 refers to an Ed25519 key and a scheme of 1 refers to Multi-Ed25519 keys.
<code>originating <b>address</b></code> refers to an account's original/first address.

Here is an example attack if we don't ask for the second signature <code>cap_update_table</code>:
Alice has rotated her account <code>addr_a</code> to <code>new_addr_a</code>. As a result, the following entry is created, to help Alice when recovering her wallet:
<code><a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>[new_addr_a]</code> -> <code>addr_a</code>
Alice has had bad day: her laptop blew up and she needs to reset her account on a new one.
(Fortunately, she still has her secret key <code>new_sk_a</code> associated with her new address <code>new_addr_a</code>, so she can do this.)

But Bob likes to mess with Alice.
Bob creates an account <code>addr_b</code> and maliciously rotates it to Alice's new address <code>new_addr_a</code>. Since we are no longer checking a PoK,
Bob can easily do this.

Now, the table will be updated to make Alice's new address point to Bob's address: <code><a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>[new_addr_a]</code> -> <code>addr_b</code>.
When Alice recovers her account, her wallet will display the attacker's address (Bob's) <code>addr_b</code> as her address.
Now Alice will give <code>addr_b</code> to everyone to pay her, but the money will go to Bob.

Because we ask for a valid <code>cap_update_table</code>, this kind of attack is not possible. Bob would not have the secret key of Alice's address
to rotate his address to Alice's address in the first place.


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key">rotate_authentication_key</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, from_scheme: u8, from_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, to_scheme: u8, to_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, cap_rotate_key: <a href="">vector</a>&lt;u8&gt;, cap_update_table: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key">rotate_authentication_key</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>,
    from_scheme: u8,
    from_public_key_bytes: <a href="">vector</a>&lt;u8&gt;,
    to_scheme: u8,
    to_public_key_bytes: <a href="">vector</a>&lt;u8&gt;,
    cap_rotate_key: <a href="">vector</a>&lt;u8&gt;,
    cap_update_table: <a href="">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a>, <a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a> {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(addr), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>));
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);

    // Verify the given `from_public_key_bytes` matches this <a href="account.md#0x1_account">account</a>'s current authentication key.
    <b>if</b> (from_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a>) {
        <b>let</b> from_pk = <a href="_new_unvalidated_public_key_from_bytes">ed25519::new_unvalidated_public_key_from_bytes</a>(from_public_key_bytes);
        <b>let</b> from_auth_key = <a href="_unvalidated_public_key_to_authentication_key">ed25519::unvalidated_public_key_to_authentication_key</a>(&from_pk);
        <b>assert</b>!(account_resource.authentication_key == from_auth_key, <a href="_unauthenticated">error::unauthenticated</a>(<a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>));
    } <b>else</b> <b>if</b> (from_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>) {
        <b>let</b> from_pk = <a href="_new_unvalidated_public_key_from_bytes">multi_ed25519::new_unvalidated_public_key_from_bytes</a>(from_public_key_bytes);
        <b>let</b> from_auth_key = <a href="_unvalidated_public_key_to_authentication_key">multi_ed25519::unvalidated_public_key_to_authentication_key</a>(&from_pk);
        <b>assert</b>!(account_resource.authentication_key == from_auth_key, <a href="_unauthenticated">error::unauthenticated</a>(<a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>));
    } <b>else</b> {
        <b>abort</b> <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_SCHEME">EINVALID_SCHEME</a>)
    };

    // Construct a valid `<a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a>` that `cap_rotate_key` and `cap_update_table` will validate against.
    <b>let</b> curr_auth_key_as_address = <a href="_to_address">from_bcs::to_address</a>(account_resource.authentication_key);
    <b>let</b> challenge = <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a> {
        sequence_number: account_resource.sequence_number,
        originator: addr,
        current_auth_key: curr_auth_key_as_address,
        new_public_key: to_public_key_bytes,
    };

    // Assert the challenges signed by the current and new keys are valid
    <a href="account.md#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key">assert_valid_rotation_proof_signature_and_get_auth_key</a>(from_scheme, from_public_key_bytes, cap_rotate_key, &challenge);
    <b>let</b> new_auth_key = <a href="account.md#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key">assert_valid_rotation_proof_signature_and_get_auth_key</a>(to_scheme, to_public_key_bytes, cap_update_table, &challenge);

    // Update the `<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>` <a href="">table</a>.
    <a href="account.md#0x1_account_update_auth_key_and_originating_address_table">update_auth_key_and_originating_address_table</a>(addr, account_resource, new_auth_key);
}
</code></pre>



</details>

<a name="0x1_account_rotate_authentication_key_with_rotation_capability"></a>

## Function `rotate_authentication_key_with_rotation_capability`



<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key_with_rotation_capability">rotate_authentication_key_with_rotation_capability</a>(delegate_signer: &<a href="">signer</a>, rotation_cap_offerer_address: <b>address</b>, new_scheme: u8, new_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, cap_update_table: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key_with_rotation_capability">rotate_authentication_key_with_rotation_capability</a>(
    delegate_signer: &<a href="">signer</a>,
    rotation_cap_offerer_address: <b>address</b>,
    new_scheme: u8,
    new_public_key_bytes: <a href="">vector</a>&lt;u8&gt;,
    cap_update_table: <a href="">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a>, <a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a> {
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(rotation_cap_offerer_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EOFFERER_ADDRESS_DOES_NOT_EXIST">EOFFERER_ADDRESS_DOES_NOT_EXIST</a>));

    // Check that there <b>exists</b> a rotation capability offer at the offerer's <a href="account.md#0x1_account">account</a> resource for the delegate.
    <b>let</b> delegate_address = <a href="_address_of">signer::address_of</a>(delegate_signer);
    <b>let</b> offerer_account_resource = <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(rotation_cap_offerer_address);
    <b>assert</b>!(<a href="_contains">option::contains</a>(&offerer_account_resource.rotation_capability_offer.for, &delegate_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_ENO_SUCH_ROTATION_CAPABILITY_OFFER">ENO_SUCH_ROTATION_CAPABILITY_OFFER</a>));

    <b>let</b> curr_auth_key = <a href="_to_address">from_bcs::to_address</a>(offerer_account_resource.authentication_key);
    <b>let</b> challenge = <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a> {
        sequence_number: <a href="account.md#0x1_account_get_sequence_number">get_sequence_number</a>(delegate_address),
        originator: rotation_cap_offerer_address,
        current_auth_key: curr_auth_key,
        new_public_key: new_public_key_bytes,
    };

    // Verifies that the `<a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a>` from above is signed under the new <b>public</b> key that we are rotating <b>to</b>.        l
    <b>let</b> new_auth_key = <a href="account.md#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key">assert_valid_rotation_proof_signature_and_get_auth_key</a>(new_scheme, new_public_key_bytes, cap_update_table, &challenge);

    // Update the `<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>` <a href="">table</a>, so we can find the originating <b>address</b> using the new <b>address</b>.
    <b>let</b> offerer_account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(rotation_cap_offerer_address);
    <a href="account.md#0x1_account_update_auth_key_and_originating_address_table">update_auth_key_and_originating_address_table</a>(rotation_cap_offerer_address, offerer_account_resource, new_auth_key);
}
</code></pre>



</details>

<a name="0x1_account_offer_rotation_capability"></a>

## Function `offer_rotation_capability`

Offers rotation capability on behalf of <code><a href="account.md#0x1_account">account</a></code> to the account at address <code>recipient_address</code>.
An account can delegate its rotation capability to only one other address at one time. If the account
has an existing rotation capability offer, calling this function will update the rotation capability offer with
the new <code>recipient_address</code>.
Here, <code>rotation_capability_sig_bytes</code> signature indicates that this key rotation is authorized by the account owner,
and prevents the classic "time-of-check time-of-use" attack.
For example, users usually rely on what the wallet displays to them as the transaction's outcome. Consider a contract that with 50% probability
(based on the current timestamp in Move), rotates somebody's key. The wallet might be unlucky and get an outcome where nothing is rotated,
incorrectly telling the user nothing bad will happen. But when the transaction actually gets executed, the attacker gets lucky and
the execution path triggers the account key rotation.
We prevent such attacks by asking for this extra signature authorizing the key rotation.

@param rotation_capability_sig_bytes is the signature by the account owner's key on <code><a href="account.md#0x1_account_RotationCapabilityOfferProofChallengeV2">RotationCapabilityOfferProofChallengeV2</a></code>.
@param account_scheme is the scheme of the account (ed25519 or multi_ed25519).
@param account_public_key_bytes is the public key of the account owner.
@param recipient_address is the address of the recipient of the rotation capability - note that if there's an existing rotation capability
offer, calling this function will replace the previous <code>recipient_address</code> upon successful verification.


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_offer_rotation_capability">offer_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, rotation_capability_sig_bytes: <a href="">vector</a>&lt;u8&gt;, account_scheme: u8, account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, recipient_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_offer_rotation_capability">offer_rotation_capability</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>,
    rotation_capability_sig_bytes: <a href="">vector</a>&lt;u8&gt;,
    account_scheme: u8,
    account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;,
    recipient_address: <b>address</b>,
) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(recipient_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>));

    // proof that this <a href="account.md#0x1_account">account</a> intends <b>to</b> delegate its rotation capability <b>to</b> another <a href="account.md#0x1_account">account</a>
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    <b>let</b> proof_challenge = <a href="account.md#0x1_account_RotationCapabilityOfferProofChallengeV2">RotationCapabilityOfferProofChallengeV2</a> {
        <a href="chain_id.md#0x1_chain_id">chain_id</a>: <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>(),
        sequence_number: account_resource.sequence_number,
        source_address: addr,
        recipient_address,
    };

    // verify the signature on `<a href="account.md#0x1_account_RotationCapabilityOfferProofChallengeV2">RotationCapabilityOfferProofChallengeV2</a>` by the <a href="account.md#0x1_account">account</a> owner
    <b>if</b> (account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a>) {
        <b>let</b> pubkey = <a href="_new_unvalidated_public_key_from_bytes">ed25519::new_unvalidated_public_key_from_bytes</a>(account_public_key_bytes);
        <b>let</b> expected_auth_key = <a href="_unvalidated_public_key_to_authentication_key">ed25519::unvalidated_public_key_to_authentication_key</a>(&pubkey);
        <b>assert</b>!(account_resource.authentication_key == expected_auth_key, <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>));

        <b>let</b> rotation_capability_sig = <a href="_new_signature_from_bytes">ed25519::new_signature_from_bytes</a>(rotation_capability_sig_bytes);
        <b>assert</b>!(<a href="_signature_verify_strict_t">ed25519::signature_verify_strict_t</a>(&rotation_capability_sig, &pubkey, proof_challenge), <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>));
    } <b>else</b> <b>if</b> (account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>) {
        <b>let</b> pubkey = <a href="_new_unvalidated_public_key_from_bytes">multi_ed25519::new_unvalidated_public_key_from_bytes</a>(account_public_key_bytes);
        <b>let</b> expected_auth_key = <a href="_unvalidated_public_key_to_authentication_key">multi_ed25519::unvalidated_public_key_to_authentication_key</a>(&pubkey);
        <b>assert</b>!(account_resource.authentication_key == expected_auth_key, <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>));

        <b>let</b> rotation_capability_sig = <a href="_new_signature_from_bytes">multi_ed25519::new_signature_from_bytes</a>(rotation_capability_sig_bytes);
        <b>assert</b>!(<a href="_signature_verify_strict_t">multi_ed25519::signature_verify_strict_t</a>(&rotation_capability_sig, &pubkey, proof_challenge), <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>));
    } <b>else</b> {
        <b>abort</b> <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_SCHEME">EINVALID_SCHEME</a>)
    };

    // <b>update</b> the existing rotation capability offer or put in a new rotation capability offer for the current <a href="account.md#0x1_account">account</a>
    <a href="_swap_or_fill">option::swap_or_fill</a>(&<b>mut</b> account_resource.rotation_capability_offer.for, recipient_address);
}
</code></pre>



</details>

<a name="0x1_account_revoke_rotation_capability"></a>

## Function `revoke_rotation_capability`

Revoke the rotation capability offer given to <code>to_be_revoked_recipient_address</code> from <code><a href="account.md#0x1_account">account</a></code>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_rotation_capability">revoke_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, to_be_revoked_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_rotation_capability">revoke_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, to_be_revoked_address: <b>address</b>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(to_be_revoked_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>));
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    <b>assert</b>!(<a href="_contains">option::contains</a>(&account_resource.rotation_capability_offer.for, &to_be_revoked_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_ENO_SUCH_ROTATION_CAPABILITY_OFFER">ENO_SUCH_ROTATION_CAPABILITY_OFFER</a>));
    <a href="account.md#0x1_account_revoke_any_rotation_capability">revoke_any_rotation_capability</a>(<a href="account.md#0x1_account">account</a>);
}
</code></pre>



</details>

<a name="0x1_account_revoke_any_rotation_capability"></a>

## Function `revoke_any_rotation_capability`

Revoke any rotation capability offer in the specified account.


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_any_rotation_capability">revoke_any_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_any_rotation_capability">revoke_any_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <a href="_extract">option::extract</a>(&<b>mut</b> account_resource.rotation_capability_offer.for);
}
</code></pre>



</details>

<a name="0x1_account_offer_signer_capability"></a>

## Function `offer_signer_capability`

Offers signer capability on behalf of <code><a href="account.md#0x1_account">account</a></code> to the account at address <code>recipient_address</code>.
An account can delegate its signer capability to only one other address at one time.
<code>signer_capability_key_bytes</code> is the <code><a href="account.md#0x1_account_SignerCapabilityOfferProofChallengeV2">SignerCapabilityOfferProofChallengeV2</a></code> signed by the account owner's key
<code>account_scheme</code> is the scheme of the account (ed25519 or multi_ed25519).
<code>account_public_key_bytes</code> is the public key of the account owner.
<code>recipient_address</code> is the address of the recipient of the signer capability - note that if there's an existing
<code>recipient_address</code> in the account owner's <code>SignerCapabilityOffer</code>, this will replace the
previous <code>recipient_address</code> upon successful verification (the previous recipient will no longer have access
to the account owner's signer capability).


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_offer_signer_capability">offer_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, signer_capability_sig_bytes: <a href="">vector</a>&lt;u8&gt;, account_scheme: u8, account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, recipient_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_offer_signer_capability">offer_signer_capability</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>,
    signer_capability_sig_bytes: <a href="">vector</a>&lt;u8&gt;,
    account_scheme: u8,
    account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;,
    recipient_address: <b>address</b>
) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> source_address = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(recipient_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>));

    // Proof that this <a href="account.md#0x1_account">account</a> intends <b>to</b> delegate its <a href="">signer</a> capability <b>to</b> another <a href="account.md#0x1_account">account</a>.
    <b>let</b> proof_challenge = <a href="account.md#0x1_account_SignerCapabilityOfferProofChallengeV2">SignerCapabilityOfferProofChallengeV2</a> {
        sequence_number: <a href="account.md#0x1_account_get_sequence_number">get_sequence_number</a>(source_address),
        source_address,
        recipient_address,
    };
    <a href="account.md#0x1_account_verify_signed_message">verify_signed_message</a>(
        source_address, account_scheme, account_public_key_bytes, signer_capability_sig_bytes, proof_challenge);

    // Update the existing <a href="">signer</a> capability offer or put in a new <a href="">signer</a> capability offer for the recipient.
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
    <a href="_swap_or_fill">option::swap_or_fill</a>(&<b>mut</b> account_resource.signer_capability_offer.for, recipient_address);
}
</code></pre>



</details>

<a name="0x1_account_is_signer_capability_offered"></a>

## Function `is_signer_capability_offered`

Returns true if the account at <code>account_addr</code> has a signer capability offer.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_is_signer_capability_offered">is_signer_capability_offered</a>(account_addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_is_signer_capability_offered">is_signer_capability_offered</a>(account_addr: <b>address</b>): bool <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> account_resource = <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
    <a href="_is_some">option::is_some</a>(&account_resource.signer_capability_offer.for)
}
</code></pre>



</details>

<a name="0x1_account_get_signer_capability_offer_for"></a>

## Function `get_signer_capability_offer_for`

Returns the address of the account that has a signer capability offer from the account at <code>account_addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_signer_capability_offer_for">get_signer_capability_offer_for</a>(account_addr: <b>address</b>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_signer_capability_offer_for">get_signer_capability_offer_for</a>(account_addr: <b>address</b>): <b>address</b> <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> account_resource = <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
    <b>assert</b>!(<a href="_is_some">option::is_some</a>(&account_resource.signer_capability_offer.for), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_ENO_SIGNER_CAPABILITY_OFFERED">ENO_SIGNER_CAPABILITY_OFFERED</a>));
    *<a href="_borrow">option::borrow</a>(&account_resource.signer_capability_offer.for)
}
</code></pre>



</details>

<a name="0x1_account_revoke_signer_capability"></a>

## Function `revoke_signer_capability`

Revoke the account owner's signer capability offer for <code>to_be_revoked_address</code> (i.e., the address that
has a signer capability offer from <code><a href="account.md#0x1_account">account</a></code> but will be revoked in this function).


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_signer_capability">revoke_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, to_be_revoked_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_signer_capability">revoke_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, to_be_revoked_address: <b>address</b>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(to_be_revoked_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EACCOUNT_DOES_NOT_EXIST">EACCOUNT_DOES_NOT_EXIST</a>));
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    <b>assert</b>!(<a href="_contains">option::contains</a>(&account_resource.signer_capability_offer.for, &to_be_revoked_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_ENO_SUCH_SIGNER_CAPABILITY">ENO_SUCH_SIGNER_CAPABILITY</a>));
    <a href="account.md#0x1_account_revoke_any_signer_capability">revoke_any_signer_capability</a>(<a href="account.md#0x1_account">account</a>);
}
</code></pre>



</details>

<a name="0x1_account_revoke_any_signer_capability"></a>

## Function `revoke_any_signer_capability`

Revoke any signer capability offer in the specified account.


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_any_signer_capability">revoke_any_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_any_signer_capability">revoke_any_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <a href="_extract">option::extract</a>(&<b>mut</b> account_resource.signer_capability_offer.for);
}
</code></pre>



</details>

<a name="0x1_account_create_authorized_signer"></a>

## Function `create_authorized_signer`

Return an authorized signer of the offerer, if there's an existing signer capability offer for <code><a href="account.md#0x1_account">account</a></code>
at the offerer's address.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_authorized_signer">create_authorized_signer</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, offerer_address: <b>address</b>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_authorized_signer">create_authorized_signer</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, offerer_address: <b>address</b>): <a href="">signer</a> <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">exists_at</a>(offerer_address), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EOFFERER_ADDRESS_DOES_NOT_EXIST">EOFFERER_ADDRESS_DOES_NOT_EXIST</a>));

    // Check <b>if</b> there's an existing <a href="">signer</a> capability offer from the offerer.
    <b>let</b> account_resource = <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(offerer_address);
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<a href="_contains">option::contains</a>(&account_resource.signer_capability_offer.for, &addr), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_ENO_SUCH_SIGNER_CAPABILITY">ENO_SUCH_SIGNER_CAPABILITY</a>));

    <a href="create_signer.md#0x1_create_signer">create_signer</a>(offerer_address)
}
</code></pre>



</details>

<a name="0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key"></a>

## Function `assert_valid_rotation_proof_signature_and_get_auth_key`

Helper functions for authentication key rotation.


<pre><code><b>fun</b> <a href="account.md#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key">assert_valid_rotation_proof_signature_and_get_auth_key</a>(scheme: u8, public_key_bytes: <a href="">vector</a>&lt;u8&gt;, signature: <a href="">vector</a>&lt;u8&gt;, challenge: &<a href="account.md#0x1_account_RotationProofChallenge">account::RotationProofChallenge</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="account.md#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key">assert_valid_rotation_proof_signature_and_get_auth_key</a>(scheme: u8, public_key_bytes: <a href="">vector</a>&lt;u8&gt;, signature: <a href="">vector</a>&lt;u8&gt;, challenge: &<a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a>): <a href="">vector</a>&lt;u8&gt; {
    <b>if</b> (scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a>) {
        <b>let</b> pk = <a href="_new_unvalidated_public_key_from_bytes">ed25519::new_unvalidated_public_key_from_bytes</a>(public_key_bytes);
        <b>let</b> sig = <a href="_new_signature_from_bytes">ed25519::new_signature_from_bytes</a>(signature);
        <b>assert</b>!(<a href="_signature_verify_strict_t">ed25519::signature_verify_strict_t</a>(&sig, &pk, *challenge), std::error::invalid_argument(<a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>));
        <a href="_unvalidated_public_key_to_authentication_key">ed25519::unvalidated_public_key_to_authentication_key</a>(&pk)
    } <b>else</b> <b>if</b> (scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>) {
        <b>let</b> pk = <a href="_new_unvalidated_public_key_from_bytes">multi_ed25519::new_unvalidated_public_key_from_bytes</a>(public_key_bytes);
        <b>let</b> sig = <a href="_new_signature_from_bytes">multi_ed25519::new_signature_from_bytes</a>(signature);
        <b>assert</b>!(<a href="_signature_verify_strict_t">multi_ed25519::signature_verify_strict_t</a>(&sig, &pk, *challenge), std::error::invalid_argument(<a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>));
        <a href="_unvalidated_public_key_to_authentication_key">multi_ed25519::unvalidated_public_key_to_authentication_key</a>(&pk)
    } <b>else</b> {
        <b>abort</b> <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_SCHEME">EINVALID_SCHEME</a>)
    }
}
</code></pre>



</details>

<a name="0x1_account_update_auth_key_and_originating_address_table"></a>

## Function `update_auth_key_and_originating_address_table`

Update the <code><a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a></code> table, so that we can find the originating address using the latest address
in the event of key recovery.


<pre><code><b>fun</b> <a href="account.md#0x1_account_update_auth_key_and_originating_address_table">update_auth_key_and_originating_address_table</a>(originating_addr: <b>address</b>, account_resource: &<b>mut</b> <a href="account.md#0x1_account_Account">account::Account</a>, new_auth_key_vector: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="account.md#0x1_account_update_auth_key_and_originating_address_table">update_auth_key_and_originating_address_table</a>(
    originating_addr: <b>address</b>,
    account_resource: &<b>mut</b> <a href="account.md#0x1_account_Account">Account</a>,
    new_auth_key_vector: <a href="">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a> {
    <b>let</b> address_map = &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(@aptos_framework).address_map;
    <b>let</b> curr_auth_key = <a href="_to_address">from_bcs::to_address</a>(account_resource.authentication_key);

    // Checks `<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>[curr_auth_key]` is either unmapped, or mapped <b>to</b> `originating_address`.
    // If it's mapped <b>to</b> the originating <b>address</b>, removes that mapping.
    // Otherwise, <b>abort</b> <b>if</b> it's mapped <b>to</b> a different <b>address</b>.
    <b>if</b> (<a href="_contains">table::contains</a>(address_map, curr_auth_key)) {
        // If account_a <b>with</b> address_a is rotating its keypair from keypair_a <b>to</b> keypair_b, we expect
        // the <b>address</b> of the <a href="account.md#0x1_account">account</a> <b>to</b> stay the same, <b>while</b> its keypair updates <b>to</b> keypair_b.
        // Here, by asserting that we're calling from the <a href="account.md#0x1_account">account</a> <b>with</b> the originating <b>address</b>, we enforce
        // the standard of keeping the same <b>address</b> and updating the keypair at the contract level.
        // Without this assertion, the dapps could also <b>update</b> the <a href="account.md#0x1_account">account</a>'s <b>address</b> <b>to</b> address_b (the <b>address</b> that
        // is programmatically related <b>to</b> keypaier_b) and <b>update</b> the keypair <b>to</b> keypair_b. This causes problems
        // for interoperability because different dapps can implement this in different ways.
        // If the <a href="account.md#0x1_account">account</a> <b>with</b> <b>address</b> b calls this function <b>with</b> two valid signatures, it will <b>abort</b> at this step,
        // because <b>address</b> b is not the <a href="account.md#0x1_account">account</a>'s originating <b>address</b>.
        <b>assert</b>!(originating_addr == <a href="_remove">table::remove</a>(address_map, curr_auth_key), <a href="_not_found">error::not_found</a>(<a href="account.md#0x1_account_EINVALID_ORIGINATING_ADDRESS">EINVALID_ORIGINATING_ADDRESS</a>));
    };

    // Set `<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>[new_auth_key] = originating_address`.
    <b>let</b> new_auth_key = <a href="_to_address">from_bcs::to_address</a>(new_auth_key_vector);
    <a href="_add">table::add</a>(address_map, new_auth_key, originating_addr);

    <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="account.md#0x1_account_KeyRotationEvent">KeyRotationEvent</a>&gt;(
        &<b>mut</b> account_resource.key_rotation_events,
        <a href="account.md#0x1_account_KeyRotationEvent">KeyRotationEvent</a> {
            old_authentication_key: account_resource.authentication_key,
            new_authentication_key: new_auth_key_vector,
        }
    );

    // Update the <a href="account.md#0x1_account">account</a> resource's authentication key.
    account_resource.authentication_key = new_auth_key_vector;
}
</code></pre>



</details>

<a name="0x1_account_create_resource_address"></a>

## Function `create_resource_address`

Basic account creation methods.
This is a helper function to compute resource addresses. Computation of the address
involves the use of a cryptographic hash operation and should be use thoughtfully.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_resource_address">create_resource_address</a>(source: &<b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_resource_address">create_resource_address</a>(source: &<b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b> {
    <b>let</b> bytes = <a href="_to_bytes">bcs::to_bytes</a>(source);
    <a href="_append">vector::append</a>(&<b>mut</b> bytes, seed);
    <a href="_push_back">vector::push_back</a>(&<b>mut</b> bytes, <a href="account.md#0x1_account_DERIVE_RESOURCE_ACCOUNT_SCHEME">DERIVE_RESOURCE_ACCOUNT_SCHEME</a>);
    <a href="_to_address">from_bcs::to_address</a>(<a href="_sha3_256">hash::sha3_256</a>(bytes))
}
</code></pre>



</details>

<a name="0x1_account_create_resource_account"></a>

## Function `create_resource_account`

A resource account is used to manage resources independent of an account managed by a user.
In Aptos a resource account is created based upon the sha3 256 of the source's address and additional seed data.
A resource account can only be created once, this is designated by setting the
<code>Account::signer_capability_offer::for</code> to the address of the resource account. While an entity may call
<code>create_account</code> to attempt to claim an account ahead of the creation of a resource account, if found Aptos will
transition ownership of the account over to the resource account. This is done by validating that the account has
yet to execute any transactions and that the <code>Account::signer_capability_offer::for</code> is none. The probability of a
collision where someone has legitimately produced a private key that maps to a resource account address is less
than <code>(1/2)^(256)</code>.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_resource_account">create_resource_account</a>(source: &<a href="">signer</a>, seed: <a href="">vector</a>&lt;u8&gt;): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_resource_account">create_resource_account</a>(source: &<a href="">signer</a>, seed: <a href="">vector</a>&lt;u8&gt;): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">SignerCapability</a>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> resource_addr = <a href="account.md#0x1_account_create_resource_address">create_resource_address</a>(&<a href="_address_of">signer::address_of</a>(source), seed);
    <b>let</b> resource = <b>if</b> (<a href="account.md#0x1_account_exists_at">exists_at</a>(resource_addr)) {
        <b>let</b> <a href="account.md#0x1_account">account</a> = <b>borrow_global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(resource_addr);
        <b>assert</b>!(
            <a href="_is_none">option::is_none</a>(&<a href="account.md#0x1_account">account</a>.signer_capability_offer.for),
            <a href="_already_exists">error::already_exists</a>(<a href="account.md#0x1_account_ERESOURCE_ACCCOUNT_EXISTS">ERESOURCE_ACCCOUNT_EXISTS</a>),
        );
        <b>assert</b>!(
            <a href="account.md#0x1_account">account</a>.sequence_number == 0,
            <a href="_invalid_state">error::invalid_state</a>(<a href="account.md#0x1_account_EACCOUNT_ALREADY_USED">EACCOUNT_ALREADY_USED</a>),
        );
        <a href="create_signer.md#0x1_create_signer">create_signer</a>(resource_addr)
    } <b>else</b> {
        <a href="account.md#0x1_account_create_account_unchecked">create_account_unchecked</a>(resource_addr)
    };

    // By default, only the <a href="account.md#0x1_account_SignerCapability">SignerCapability</a> should have control over the resource <a href="account.md#0x1_account">account</a> and not the auth key.
    // If the source <a href="account.md#0x1_account">account</a> wants direct control via auth key, they would need <b>to</b> explicitly rotate the auth key
    // of the resource <a href="account.md#0x1_account">account</a> using the <a href="account.md#0x1_account_SignerCapability">SignerCapability</a>.
    <a href="account.md#0x1_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(&resource, <a href="account.md#0x1_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>);

    <b>let</b> <a href="account.md#0x1_account">account</a> = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(resource_addr);
    <a href="account.md#0x1_account">account</a>.signer_capability_offer.for = <a href="_some">option::some</a>(resource_addr);
    <b>let</b> signer_cap = <a href="account.md#0x1_account_SignerCapability">SignerCapability</a> { <a href="account.md#0x1_account">account</a>: resource_addr };
    (resource, signer_cap)
}
</code></pre>



</details>

<a name="0x1_account_create_framework_reserved_account"></a>

## Function `create_framework_reserved_account`

create the account for system reserved addresses


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_create_framework_reserved_account">create_framework_reserved_account</a>(addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_create_framework_reserved_account">create_framework_reserved_account</a>(addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">SignerCapability</a>) {
    <b>assert</b>!(
        addr == @0x1 ||
            addr == @0x2 ||
            addr == @0x3 ||
            addr == @0x4 ||
            addr == @0x5 ||
            addr == @0x6 ||
            addr == @0x7 ||
            addr == @0x8 ||
            addr == @0x9 ||
            addr == @0xa,
        <a href="_permission_denied">error::permission_denied</a>(<a href="account.md#0x1_account_ENO_VALID_FRAMEWORK_RESERVED_ADDRESS">ENO_VALID_FRAMEWORK_RESERVED_ADDRESS</a>),
    );
    <b>let</b> <a href="">signer</a> = <a href="account.md#0x1_account_create_account_unchecked">create_account_unchecked</a>(addr);
    <b>let</b> signer_cap = <a href="account.md#0x1_account_SignerCapability">SignerCapability</a> { <a href="account.md#0x1_account">account</a>: addr };
    (<a href="">signer</a>, signer_cap)
}
</code></pre>



</details>

<a name="0x1_account_create_guid"></a>

## Function `create_guid`

GUID management methods.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_guid">create_guid</a>(account_signer: &<a href="">signer</a>): <a href="guid.md#0x1_guid_GUID">guid::GUID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_guid">create_guid</a>(account_signer: &<a href="">signer</a>): <a href="guid.md#0x1_guid_GUID">guid::GUID</a> <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(account_signer);
    <b>let</b> <a href="account.md#0x1_account">account</a> = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    <b>let</b> <a href="guid.md#0x1_guid">guid</a> = <a href="guid.md#0x1_guid_create">guid::create</a>(addr, &<b>mut</b> <a href="account.md#0x1_account">account</a>.guid_creation_num);
    <b>assert</b>!(
        <a href="account.md#0x1_account">account</a>.guid_creation_num &lt; <a href="account.md#0x1_account_MAX_GUID_CREATION_NUM">MAX_GUID_CREATION_NUM</a>,
        <a href="_out_of_range">error::out_of_range</a>(<a href="account.md#0x1_account_EEXCEEDED_MAX_GUID_CREATION_NUM">EEXCEEDED_MAX_GUID_CREATION_NUM</a>),
    );
    <a href="guid.md#0x1_guid">guid</a>
}
</code></pre>



</details>

<a name="0x1_account_new_event_handle"></a>

## Function `new_event_handle`

GUID management methods.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_new_event_handle">new_event_handle</a>&lt;T: drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>): <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_new_event_handle">new_event_handle</a>&lt;T: drop + store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>): EventHandle&lt;T&gt; <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <a href="event.md#0x1_event_new_event_handle">event::new_event_handle</a>(<a href="account.md#0x1_account_create_guid">create_guid</a>(<a href="account.md#0x1_account">account</a>))
}
</code></pre>



</details>

<a name="0x1_account_register_coin"></a>

## Function `register_coin`

Coin management methods.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_register_coin">register_coin</a>&lt;CoinType&gt;(account_addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_register_coin">register_coin</a>&lt;CoinType&gt;(account_addr: <b>address</b>) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> <a href="account.md#0x1_account">account</a> = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="account.md#0x1_account_CoinRegisterEvent">CoinRegisterEvent</a>&gt;(
        &<b>mut</b> <a href="account.md#0x1_account">account</a>.coin_register_events,
        <a href="account.md#0x1_account_CoinRegisterEvent">CoinRegisterEvent</a> {
            <a href="">type_info</a>: <a href="_type_of">type_info::type_of</a>&lt;CoinType&gt;(),
        },
    );
}
</code></pre>



</details>

<a name="0x1_account_create_signer_with_capability"></a>

## Function `create_signer_with_capability`

Capability based functions for efficient use.


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>): <a href="">signer</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x1_account_SignerCapability">SignerCapability</a>): <a href="">signer</a> {
    <b>let</b> addr = &capability.<a href="account.md#0x1_account">account</a>;
    <a href="create_signer.md#0x1_create_signer">create_signer</a>(*addr)
}
</code></pre>



</details>

<a name="0x1_account_get_signer_capability_address"></a>

## Function `get_signer_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_signer_capability_address">get_signer_capability_address</a>(capability: &<a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_signer_capability_address">get_signer_capability_address</a>(capability: &<a href="account.md#0x1_account_SignerCapability">SignerCapability</a>): <b>address</b> {
    capability.<a href="account.md#0x1_account">account</a>
}
</code></pre>



</details>

<a name="0x1_account_verify_signed_message"></a>

## Function `verify_signed_message`



<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_verify_signed_message">verify_signed_message</a>&lt;T: drop&gt;(<a href="account.md#0x1_account">account</a>: <b>address</b>, account_scheme: u8, account_public_key: <a href="">vector</a>&lt;u8&gt;, signed_message_bytes: <a href="">vector</a>&lt;u8&gt;, message: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_verify_signed_message">verify_signed_message</a>&lt;T: drop&gt;(
    <a href="account.md#0x1_account">account</a>: <b>address</b>,
    account_scheme: u8,
    account_public_key: <a href="">vector</a>&lt;u8&gt;,
    signed_message_bytes: <a href="">vector</a>&lt;u8&gt;,
    message: T,
) <b>acquires</b> <a href="account.md#0x1_account_Account">Account</a> {
    <b>let</b> account_resource = <b>borrow_global_mut</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="account.md#0x1_account">account</a>);
    // Verify that the `<a href="account.md#0x1_account_SignerCapabilityOfferProofChallengeV2">SignerCapabilityOfferProofChallengeV2</a>` <b>has</b> the right information and is signed by the <a href="account.md#0x1_account">account</a> owner's key
    <b>if</b> (account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a>) {
        <b>let</b> pubkey = <a href="_new_unvalidated_public_key_from_bytes">ed25519::new_unvalidated_public_key_from_bytes</a>(account_public_key);
        <b>let</b> expected_auth_key = <a href="_unvalidated_public_key_to_authentication_key">ed25519::unvalidated_public_key_to_authentication_key</a>(&pubkey);
        <b>assert</b>!(
            account_resource.authentication_key == expected_auth_key,
            <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>),
        );

        <b>let</b> signer_capability_sig = <a href="_new_signature_from_bytes">ed25519::new_signature_from_bytes</a>(signed_message_bytes);
        <b>assert</b>!(
            <a href="_signature_verify_strict_t">ed25519::signature_verify_strict_t</a>(&signer_capability_sig, &pubkey, message),
            <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>),
        );
    } <b>else</b> <b>if</b> (account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>) {
        <b>let</b> pubkey = <a href="_new_unvalidated_public_key_from_bytes">multi_ed25519::new_unvalidated_public_key_from_bytes</a>(account_public_key);
        <b>let</b> expected_auth_key = <a href="_unvalidated_public_key_to_authentication_key">multi_ed25519::unvalidated_public_key_to_authentication_key</a>(&pubkey);
        <b>assert</b>!(
            account_resource.authentication_key == expected_auth_key,
            <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EWRONG_CURRENT_PUBLIC_KEY">EWRONG_CURRENT_PUBLIC_KEY</a>),
        );

        <b>let</b> signer_capability_sig = <a href="_new_signature_from_bytes">multi_ed25519::new_signature_from_bytes</a>(signed_message_bytes);
        <b>assert</b>!(
            <a href="_signature_verify_strict_t">multi_ed25519::signature_verify_strict_t</a>(&signer_capability_sig, &pubkey, message),
            <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_PROOF_OF_KNOWLEDGE">EINVALID_PROOF_OF_KNOWLEDGE</a>),
        );
    } <b>else</b> {
        <b>abort</b> <a href="_invalid_argument">error::invalid_argument</a>(<a href="account.md#0x1_account_EINVALID_SCHEME">EINVALID_SCHEME</a>)
    };
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_initialize">initialize</a>(aptos_framework: &<a href="">signer</a>)
</code></pre>


Only the address <code>@aptos_framework</code> can call.
OriginatingAddress does not exist under <code>@aptos_framework</code> before the call.


<pre><code><b>let</b> aptos_addr = <a href="_address_of">signer::address_of</a>(aptos_framework);
<b>aborts_if</b> !<a href="system_addresses.md#0x1_system_addresses_is_aptos_framework_address">system_addresses::is_aptos_framework_address</a>(aptos_addr);
<b>aborts_if</b> <b>exists</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(aptos_addr);
<b>ensures</b> <b>exists</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(aptos_addr);
</code></pre>



<a name="@Specification_1_create_account"></a>

### Function `create_account`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_create_account">create_account</a>(new_address: <b>address</b>): <a href="">signer</a>
</code></pre>


Check if the bytes of the new address is 32.
The Account does not exist under the new address before creating the account.
Limit the new account address is not @vm_reserved / @aptos_framework / @aptos_toke.


<pre><code><b>include</b> <a href="account.md#0x1_account_CreateAccountAbortsIf">CreateAccountAbortsIf</a> {addr: new_address};
<b>aborts_if</b> new_address == @vm_reserved || new_address == @aptos_framework || new_address == @aptos_token;
<b>ensures</b> <a href="_address_of">signer::address_of</a>(result) == new_address;
</code></pre>



<a name="@Specification_1_create_account_unchecked"></a>

### Function `create_account_unchecked`


<pre><code><b>fun</b> <a href="account.md#0x1_account_create_account_unchecked">create_account_unchecked</a>(new_address: <b>address</b>): <a href="">signer</a>
</code></pre>


Check if the bytes of the new address is 32.
The Account does not exist under the new address before creating the account.


<pre><code><b>include</b> <a href="account.md#0x1_account_CreateAccountAbortsIf">CreateAccountAbortsIf</a> {addr: new_address};
<b>ensures</b> <a href="_address_of">signer::address_of</a>(result) == new_address;
</code></pre>




<a name="0x1_account_CreateAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="account.md#0x1_account_CreateAccountAbortsIf">CreateAccountAbortsIf</a> {
    addr: <b>address</b>;
    <b>let</b> authentication_key = <a href="_to_bytes">bcs::to_bytes</a>(addr);
    <b>aborts_if</b> len(authentication_key) != 32;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
}
</code></pre>



<a name="@Specification_1_get_guid_next_creation_num"></a>

### Function `get_guid_next_creation_num`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_guid_next_creation_num">get_guid_next_creation_num</a>(addr: <b>address</b>): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>ensures</b> result == <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).guid_creation_num;
</code></pre>



<a name="@Specification_1_get_sequence_number"></a>

### Function `get_sequence_number`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_sequence_number">get_sequence_number</a>(addr: <b>address</b>): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>ensures</b> result == <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).sequence_number;
</code></pre>



<a name="@Specification_1_increment_sequence_number"></a>

### Function `increment_sequence_number`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_increment_sequence_number">increment_sequence_number</a>(addr: <b>address</b>)
</code></pre>


The Account existed under the address.
The sequence_number of the Account is up to MAX_U64.


<pre><code><b>let</b> sequence_number = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).sequence_number;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> sequence_number == <a href="account.md#0x1_account_MAX_U64">MAX_U64</a>;
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>let</b> <b>post</b> post_sequence_number = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).sequence_number;
<b>ensures</b> post_sequence_number == sequence_number + 1;
</code></pre>



<a name="@Specification_1_get_authentication_key"></a>

### Function `get_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_authentication_key">get_authentication_key</a>(addr: <b>address</b>): <a href="">vector</a>&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>ensures</b> result == <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr).authentication_key;
</code></pre>



<a name="@Specification_1_rotate_authentication_key_internal"></a>

### Function `rotate_authentication_key_internal`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key_internal">rotate_authentication_key_internal</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, new_auth_key: <a href="">vector</a>&lt;u8&gt;)
</code></pre>


The Account existed under the signer before the call.
The length of new_auth_key is 32.


<pre><code><b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> <b>post</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> <a href="_length">vector::length</a>(new_auth_key) != 32;
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>ensures</b> account_resource.authentication_key == new_auth_key;
</code></pre>




<a name="0x1_account_spec_assert_valid_rotation_proof_signature_and_get_auth_key"></a>


<pre><code><b>fun</b> <a href="account.md#0x1_account_spec_assert_valid_rotation_proof_signature_and_get_auth_key">spec_assert_valid_rotation_proof_signature_and_get_auth_key</a>(scheme: u8, public_key_bytes: <a href="">vector</a>&lt;u8&gt;, signature: <a href="">vector</a>&lt;u8&gt;, challenge: <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a>): <a href="">vector</a>&lt;u8&gt;;
</code></pre>



<a name="@Specification_1_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key">rotate_authentication_key</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, from_scheme: u8, from_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, to_scheme: u8, to_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, cap_rotate_key: <a href="">vector</a>&lt;u8&gt;, cap_update_table: <a href="">vector</a>&lt;u8&gt;)
</code></pre>


The Account existed under the signer
The authentication scheme is ED25519_SCHEME and MULTI_ED25519_SCHEME


<pre><code><b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>include</b> from_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: from_public_key_bytes };
<b>aborts_if</b> from_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && ({
    <b>let</b> expected_auth_key = <a href="_spec_public_key_bytes_to_authentication_key">ed25519::spec_public_key_bytes_to_authentication_key</a>(from_public_key_bytes);
    account_resource.authentication_key != expected_auth_key
});
<b>include</b> from_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">multi_ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: from_public_key_bytes };
<b>aborts_if</b> from_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> && ({
    <b>let</b> from_auth_key = <a href="_spec_public_key_bytes_to_authentication_key">multi_ed25519::spec_public_key_bytes_to_authentication_key</a>(from_public_key_bytes);
    account_resource.authentication_key != from_auth_key
});
<b>aborts_if</b> from_scheme != <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && from_scheme != <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>;
<b>let</b> curr_auth_key = <a href="_deserialize">from_bcs::deserialize</a>&lt;<b>address</b>&gt;(account_resource.authentication_key);
<b>aborts_if</b> !<a href="_deserializable">from_bcs::deserializable</a>&lt;<b>address</b>&gt;(account_resource.authentication_key);
<b>let</b> challenge = <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a> {
    sequence_number: account_resource.sequence_number,
    originator: addr,
    current_auth_key: curr_auth_key,
    new_public_key: to_public_key_bytes,
};
<b>include</b> <a href="account.md#0x1_account_AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf">AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf</a> {
    scheme: from_scheme,
    public_key_bytes: from_public_key_bytes,
    signature: cap_rotate_key,
    challenge: challenge,
};
<b>include</b> <a href="account.md#0x1_account_AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf">AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf</a> {
    scheme: to_scheme,
    public_key_bytes: to_public_key_bytes,
    signature: cap_update_table,
    challenge: challenge,
};
<b>pragma</b> aborts_if_is_partial;
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(@aptos_framework);
</code></pre>



<a name="@Specification_1_rotate_authentication_key_with_rotation_capability"></a>

### Function `rotate_authentication_key_with_rotation_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_rotate_authentication_key_with_rotation_capability">rotate_authentication_key_with_rotation_capability</a>(delegate_signer: &<a href="">signer</a>, rotation_cap_offerer_address: <b>address</b>, new_scheme: u8, new_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, cap_update_table: <a href="">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(rotation_cap_offerer_address);
<b>let</b> delegate_address = <a href="_address_of">signer::address_of</a>(delegate_signer);
<b>let</b> offerer_account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(rotation_cap_offerer_address);
<b>aborts_if</b> !<a href="_deserializable">from_bcs::deserializable</a>&lt;<b>address</b>&gt;(offerer_account_resource.authentication_key);
<b>let</b> curr_auth_key = <a href="_deserialize">from_bcs::deserialize</a>&lt;<b>address</b>&gt;(offerer_account_resource.authentication_key);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(delegate_address);
<b>let</b> challenge = <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a> {
    sequence_number: <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(delegate_address).sequence_number,
    originator: rotation_cap_offerer_address,
    current_auth_key: curr_auth_key,
    new_public_key: new_public_key_bytes,
};
<b>aborts_if</b> !<a href="_spec_contains">option::spec_contains</a>(offerer_account_resource.rotation_capability_offer.for, delegate_address);
<b>include</b> <a href="account.md#0x1_account_AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf">AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf</a> {
    scheme: new_scheme,
    public_key_bytes: new_public_key_bytes,
    signature: cap_update_table,
    challenge: challenge,
};
<b>pragma</b> aborts_if_is_partial;
</code></pre>



<a name="@Specification_1_offer_rotation_capability"></a>

### Function `offer_rotation_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_offer_rotation_capability">offer_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, rotation_capability_sig_bytes: <a href="">vector</a>&lt;u8&gt;, account_scheme: u8, account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, recipient_address: <b>address</b>)
</code></pre>




<pre><code><b>let</b> source_address = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
<b>let</b> proof_challenge = <a href="account.md#0x1_account_RotationCapabilityOfferProofChallengeV2">RotationCapabilityOfferProofChallengeV2</a> {
    <a href="chain_id.md#0x1_chain_id">chain_id</a>: <b>global</b>&lt;<a href="chain_id.md#0x1_chain_id_ChainId">chain_id::ChainId</a>&gt;(@aptos_framework).id,
    sequence_number: account_resource.sequence_number,
    source_address,
    recipient_address,
};
<b>aborts_if</b> !<b>exists</b>&lt;<a href="chain_id.md#0x1_chain_id_ChainId">chain_id::ChainId</a>&gt;(@aptos_framework);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(recipient_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
<b>include</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: account_public_key_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && ({
    <b>let</b> expected_auth_key = <a href="_spec_public_key_bytes_to_authentication_key">ed25519::spec_public_key_bytes_to_authentication_key</a>(account_public_key_bytes);
    account_resource.authentication_key != expected_auth_key
});
<b>include</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewSignatureFromBytesAbortsIf">ed25519::NewSignatureFromBytesAbortsIf</a> { bytes: rotation_capability_sig_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && !<a href="_spec_signature_verify_strict_t">ed25519::spec_signature_verify_strict_t</a>(
    <a href="_Signature">ed25519::Signature</a> { bytes: rotation_capability_sig_bytes },
    <a href="_UnvalidatedPublicKey">ed25519::UnvalidatedPublicKey</a> { bytes: account_public_key_bytes },
    proof_challenge
);
<b>include</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">multi_ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: account_public_key_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> && ({
    <b>let</b> expected_auth_key = <a href="_spec_public_key_bytes_to_authentication_key">multi_ed25519::spec_public_key_bytes_to_authentication_key</a>(account_public_key_bytes);
    account_resource.authentication_key != expected_auth_key
});
<b>include</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewSignatureFromBytesAbortsIf">multi_ed25519::NewSignatureFromBytesAbortsIf</a> { bytes: rotation_capability_sig_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> && !<a href="_spec_signature_verify_strict_t">multi_ed25519::spec_signature_verify_strict_t</a>(
    <a href="_Signature">multi_ed25519::Signature</a> { bytes: rotation_capability_sig_bytes },
    <a href="_UnvalidatedPublicKey">multi_ed25519::UnvalidatedPublicKey</a> { bytes: account_public_key_bytes },
    proof_challenge
);
<b>aborts_if</b> account_scheme != <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && account_scheme != <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>;
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
</code></pre>



<a name="@Specification_1_revoke_rotation_capability"></a>

### Function `revoke_rotation_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_rotation_capability">revoke_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, to_be_revoked_address: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(to_be_revoked_address);
<b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> !<a href="_spec_contains">option::spec_contains</a>(account_resource.rotation_capability_offer.for,to_be_revoked_address);
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(to_be_revoked_address);
</code></pre>



<a name="@Specification_1_revoke_any_rotation_capability"></a>

### Function `revoke_any_rotation_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_any_rotation_capability">revoke_any_rotation_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>




<pre><code><b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> !<a href="_is_some">option::is_some</a>(account_resource.rotation_capability_offer.for);
</code></pre>



<a name="@Specification_1_offer_signer_capability"></a>

### Function `offer_signer_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_offer_signer_capability">offer_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, signer_capability_sig_bytes: <a href="">vector</a>&lt;u8&gt;, account_scheme: u8, account_public_key_bytes: <a href="">vector</a>&lt;u8&gt;, recipient_address: <b>address</b>)
</code></pre>


The Account existed under the signer.
The authentication scheme is ED25519_SCHEME and MULTI_ED25519_SCHEME.


<pre><code><b>let</b> source_address = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
<b>let</b> proof_challenge = <a href="account.md#0x1_account_SignerCapabilityOfferProofChallengeV2">SignerCapabilityOfferProofChallengeV2</a> {
    sequence_number: account_resource.sequence_number,
    source_address,
    recipient_address,
};
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(recipient_address);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
<b>include</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: account_public_key_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && ({
    <b>let</b> expected_auth_key = <a href="_spec_public_key_bytes_to_authentication_key">ed25519::spec_public_key_bytes_to_authentication_key</a>(account_public_key_bytes);
    account_resource.authentication_key != expected_auth_key
});
<b>include</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewSignatureFromBytesAbortsIf">ed25519::NewSignatureFromBytesAbortsIf</a> { bytes: signer_capability_sig_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && !<a href="_spec_signature_verify_strict_t">ed25519::spec_signature_verify_strict_t</a>(
    <a href="_Signature">ed25519::Signature</a> { bytes: signer_capability_sig_bytes },
    <a href="_UnvalidatedPublicKey">ed25519::UnvalidatedPublicKey</a> { bytes: account_public_key_bytes },
    proof_challenge
);
<b>include</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">multi_ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: account_public_key_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> && ({
    <b>let</b> expected_auth_key = <a href="_spec_public_key_bytes_to_authentication_key">multi_ed25519::spec_public_key_bytes_to_authentication_key</a>(account_public_key_bytes);
    account_resource.authentication_key != expected_auth_key
});
<b>include</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewSignatureFromBytesAbortsIf">multi_ed25519::NewSignatureFromBytesAbortsIf</a> { bytes: signer_capability_sig_bytes };
<b>aborts_if</b> account_scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> && !<a href="_spec_signature_verify_strict_t">multi_ed25519::spec_signature_verify_strict_t</a>(
    <a href="_Signature">multi_ed25519::Signature</a> { bytes: signer_capability_sig_bytes },
    <a href="_UnvalidatedPublicKey">multi_ed25519::UnvalidatedPublicKey</a> { bytes: account_public_key_bytes },
    proof_challenge
);
<b>aborts_if</b> account_scheme != <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && account_scheme != <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>;
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(source_address);
</code></pre>



<a name="@Specification_1_is_signer_capability_offered"></a>

### Function `is_signer_capability_offered`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_is_signer_capability_offered">is_signer_capability_offered</a>(account_addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
</code></pre>



<a name="@Specification_1_get_signer_capability_offer_for"></a>

### Function `get_signer_capability_offer_for`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_get_signer_capability_offer_for">get_signer_capability_offer_for</a>(account_addr: <b>address</b>): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
<b>aborts_if</b> len(account_resource.signer_capability_offer.for.vec) == 0;
</code></pre>



<a name="@Specification_1_revoke_signer_capability"></a>

### Function `revoke_signer_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_signer_capability">revoke_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, to_be_revoked_address: <b>address</b>)
</code></pre>


The Account existed under the signer.
The value of signer_capability_offer.for of Account resource under the signer is to_be_revoked_address.


<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(to_be_revoked_address);
<b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>aborts_if</b> !<a href="_spec_contains">option::spec_contains</a>(account_resource.signer_capability_offer.for,to_be_revoked_address);
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
<b>ensures</b> <b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(to_be_revoked_address);
</code></pre>



<a name="@Specification_1_revoke_any_signer_capability"></a>

### Function `revoke_any_signer_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="account.md#0x1_account_revoke_any_signer_capability">revoke_any_signer_capability</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>)
</code></pre>




<pre><code><b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> !<a href="_is_some">option::is_some</a>(account_resource.signer_capability_offer.for);
</code></pre>



<a name="@Specification_1_create_authorized_signer"></a>

### Function `create_authorized_signer`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_authorized_signer">create_authorized_signer</a>(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>, offerer_address: <b>address</b>): <a href="">signer</a>
</code></pre>


The Account existed under the signer.
The value of signer_capability_offer.for of Account resource under the signer is offerer_address.


<pre><code><b>include</b> <a href="account.md#0x1_account_AccountContainsAddr">AccountContainsAddr</a>{
    <a href="account.md#0x1_account">account</a>: <a href="account.md#0x1_account">account</a>,
    <b>address</b>: offerer_address,
};
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(offerer_address);
<b>ensures</b> <b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(offerer_address);
<b>ensures</b> <a href="_address_of">signer::address_of</a>(result) == offerer_address;
</code></pre>




<a name="0x1_account_AccountContainsAddr"></a>


<pre><code><b>schema</b> <a href="account.md#0x1_account_AccountContainsAddr">AccountContainsAddr</a> {
    <a href="account.md#0x1_account">account</a>: <a href="">signer</a>;
    <b>address</b>: <b>address</b>;
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> account_resource = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<b>address</b>);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<b>address</b>);
    <b>aborts_if</b> !<a href="_spec_contains">option::spec_contains</a>(account_resource.signer_capability_offer.for,addr);
}
</code></pre>



<a name="@Specification_1_assert_valid_rotation_proof_signature_and_get_auth_key"></a>

### Function `assert_valid_rotation_proof_signature_and_get_auth_key`


<pre><code><b>fun</b> <a href="account.md#0x1_account_assert_valid_rotation_proof_signature_and_get_auth_key">assert_valid_rotation_proof_signature_and_get_auth_key</a>(scheme: u8, public_key_bytes: <a href="">vector</a>&lt;u8&gt;, signature: <a href="">vector</a>&lt;u8&gt;, challenge: &<a href="account.md#0x1_account_RotationProofChallenge">account::RotationProofChallenge</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>




<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="account.md#0x1_account_AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf">AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf</a>;
<b>ensures</b> [abstract] result == <a href="account.md#0x1_account_spec_assert_valid_rotation_proof_signature_and_get_auth_key">spec_assert_valid_rotation_proof_signature_and_get_auth_key</a>(scheme, public_key_bytes, signature, challenge);
</code></pre>




<a name="0x1_account_AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf"></a>


<pre><code><b>schema</b> <a href="account.md#0x1_account_AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf">AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf</a> {
    scheme: u8;
    public_key_bytes: <a href="">vector</a>&lt;u8&gt;;
    signature: <a href="">vector</a>&lt;u8&gt;;
    challenge: <a href="account.md#0x1_account_RotationProofChallenge">RotationProofChallenge</a>;
    <b>include</b> scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: public_key_bytes };
    <b>include</b> scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> ==&gt; <a href="_NewSignatureFromBytesAbortsIf">ed25519::NewSignatureFromBytesAbortsIf</a> { bytes: signature };
    <b>aborts_if</b> scheme == <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && !<a href="_spec_signature_verify_strict_t">ed25519::spec_signature_verify_strict_t</a>(
        <a href="_Signature">ed25519::Signature</a> { bytes: signature },
        <a href="_UnvalidatedPublicKey">ed25519::UnvalidatedPublicKey</a> { bytes: public_key_bytes },
        challenge
    );
    <b>include</b> scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewUnvalidatedPublicKeyFromBytesAbortsIf">multi_ed25519::NewUnvalidatedPublicKeyFromBytesAbortsIf</a> { bytes: public_key_bytes };
    <b>include</b> scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> ==&gt; <a href="_NewSignatureFromBytesAbortsIf">multi_ed25519::NewSignatureFromBytesAbortsIf</a> { bytes: signature };
    <b>aborts_if</b> scheme == <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a> && !<a href="_spec_signature_verify_strict_t">multi_ed25519::spec_signature_verify_strict_t</a>(
        <a href="_Signature">multi_ed25519::Signature</a> { bytes: signature },
        <a href="_UnvalidatedPublicKey">multi_ed25519::UnvalidatedPublicKey</a> { bytes: public_key_bytes },
        challenge
    );
    <b>aborts_if</b> scheme != <a href="account.md#0x1_account_ED25519_SCHEME">ED25519_SCHEME</a> && scheme != <a href="account.md#0x1_account_MULTI_ED25519_SCHEME">MULTI_ED25519_SCHEME</a>;
}
</code></pre>



<a name="@Specification_1_update_auth_key_and_originating_address_table"></a>

### Function `update_auth_key_and_originating_address_table`


<pre><code><b>fun</b> <a href="account.md#0x1_account_update_auth_key_and_originating_address_table">update_auth_key_and_originating_address_table</a>(originating_addr: <b>address</b>, account_resource: &<b>mut</b> <a href="account.md#0x1_account_Account">account::Account</a>, new_auth_key_vector: <a href="">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(@aptos_framework);
<b>include</b> <a href="account.md#0x1_account_UpdateAuthKeyAndOriginatingAddressTableAbortsIf">UpdateAuthKeyAndOriginatingAddressTableAbortsIf</a>;
</code></pre>




<a name="0x1_account_UpdateAuthKeyAndOriginatingAddressTableAbortsIf"></a>


<pre><code><b>schema</b> <a href="account.md#0x1_account_UpdateAuthKeyAndOriginatingAddressTableAbortsIf">UpdateAuthKeyAndOriginatingAddressTableAbortsIf</a> {
    originating_addr: <b>address</b>;
    account_resource: <a href="account.md#0x1_account_Account">Account</a>;
    new_auth_key_vector: <a href="">vector</a>&lt;u8&gt;;
    <b>let</b> address_map = <b>global</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(@aptos_framework).address_map;
    <b>let</b> curr_auth_key = <a href="_deserialize">from_bcs::deserialize</a>&lt;<b>address</b>&gt;(account_resource.authentication_key);
    <b>let</b> new_auth_key = <a href="_deserialize">from_bcs::deserialize</a>&lt;<b>address</b>&gt;(new_auth_key_vector);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_OriginatingAddress">OriginatingAddress</a>&gt;(@aptos_framework);
    <b>aborts_if</b> !<a href="_deserializable">from_bcs::deserializable</a>&lt;<b>address</b>&gt;(account_resource.authentication_key);
    <b>aborts_if</b> <a href="_spec_contains">table::spec_contains</a>(address_map, curr_auth_key) &&
        <a href="_spec_get">table::spec_get</a>(address_map, curr_auth_key) != originating_addr;
    <b>aborts_if</b> !<a href="_deserializable">from_bcs::deserializable</a>&lt;<b>address</b>&gt;(new_auth_key_vector);
    <b>aborts_if</b> curr_auth_key != new_auth_key && <a href="_spec_contains">table::spec_contains</a>(address_map, new_auth_key);
}
</code></pre>



<a name="@Specification_1_create_resource_address"></a>

### Function `create_resource_address`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_resource_address">create_resource_address</a>(source: &<b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b>
</code></pre>


The Account existed under the signer
The value of signer_capability_offer.for of Account resource under the signer is to_be_revoked_address


<pre><code><b>pragma</b> opaque;
<b>pragma</b> aborts_if_is_strict = <b>false</b>;
<b>aborts_if</b> [abstract] <b>false</b>;
<b>ensures</b> [abstract] result == <a href="account.md#0x1_account_spec_create_resource_address">spec_create_resource_address</a>(source, seed);
</code></pre>




<a name="0x1_account_spec_create_resource_address"></a>


<pre><code><b>fun</b> <a href="account.md#0x1_account_spec_create_resource_address">spec_create_resource_address</a>(source: <b>address</b>, seed: <a href="">vector</a>&lt;u8&gt;): <b>address</b>;
</code></pre>



<a name="@Specification_1_create_resource_account"></a>

### Function `create_resource_account`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_resource_account">create_resource_account</a>(source: &<a href="">signer</a>, seed: <a href="">vector</a>&lt;u8&gt;): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>)
</code></pre>




<pre><code><b>let</b> source_addr = <a href="_address_of">signer::address_of</a>(source);
<b>let</b> resource_addr = <a href="account.md#0x1_account_spec_create_resource_address">spec_create_resource_address</a>(source_addr, seed);
<b>aborts_if</b> len(<a href="account.md#0x1_account_ZERO_AUTH_KEY">ZERO_AUTH_KEY</a>) != 32;
<b>include</b> <a href="account.md#0x1_account_exists_at">exists_at</a>(resource_addr) ==&gt; <a href="account.md#0x1_account_CreateResourceAccountAbortsIf">CreateResourceAccountAbortsIf</a>;
<b>include</b> !<a href="account.md#0x1_account_exists_at">exists_at</a>(resource_addr) ==&gt; <a href="account.md#0x1_account_CreateAccountAbortsIf">CreateAccountAbortsIf</a> {addr: resource_addr};
</code></pre>



<a name="@Specification_1_create_framework_reserved_account"></a>

### Function `create_framework_reserved_account`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_create_framework_reserved_account">create_framework_reserved_account</a>(addr: <b>address</b>): (<a href="">signer</a>, <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>)
</code></pre>


Check if the bytes of the new address is 32.
The Account does not exist under the new address before creating the account.
The system reserved addresses is @0x1 / @0x2 / @0x3 / @0x4 / @0x5  / @0x6 / @0x7 / @0x8 / @0x9 / @0xa.


<pre><code><b>aborts_if</b> <a href="account.md#0x1_account_spec_is_framework_address">spec_is_framework_address</a>(addr);
<b>include</b> <a href="account.md#0x1_account_CreateAccountAbortsIf">CreateAccountAbortsIf</a> {addr};
<b>ensures</b> <a href="_address_of">signer::address_of</a>(result_1) == addr;
<b>ensures</b> result_2 == <a href="account.md#0x1_account_SignerCapability">SignerCapability</a> { <a href="account.md#0x1_account">account</a>: addr };
</code></pre>




<a name="0x1_account_spec_is_framework_address"></a>


<pre><code><b>fun</b> <a href="account.md#0x1_account_spec_is_framework_address">spec_is_framework_address</a>(addr: <b>address</b>): bool{
   addr != @0x1 &&
   addr != @0x2 &&
   addr != @0x3 &&
   addr != @0x4 &&
   addr != @0x5 &&
   addr != @0x6 &&
   addr != @0x7 &&
   addr != @0x8 &&
   addr != @0x9 &&
   addr != @0xa
}
</code></pre>



<a name="@Specification_1_create_guid"></a>

### Function `create_guid`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_guid">create_guid</a>(account_signer: &<a href="">signer</a>): <a href="guid.md#0x1_guid_GUID">guid::GUID</a>
</code></pre>


The Account existed under the signer.
The guid_creation_num of the ccount resource is up to MAX_U64.


<pre><code><b>let</b> addr = <a href="_address_of">signer::address_of</a>(account_signer);
<b>include</b> <a href="account.md#0x1_account_NewEventHandleAbortsIf">NewEventHandleAbortsIf</a> {
    <a href="account.md#0x1_account">account</a>: account_signer,
};
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
</code></pre>



<a name="@Specification_1_new_event_handle"></a>

### Function `new_event_handle`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_new_event_handle">new_event_handle</a>&lt;T: drop, store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="">signer</a>): <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;T&gt;
</code></pre>


The Account existed under the signer.
The guid_creation_num of the Account is up to MAX_U64.


<pre><code><b>include</b> <a href="account.md#0x1_account_NewEventHandleAbortsIf">NewEventHandleAbortsIf</a>;
</code></pre>




<a name="0x1_account_NewEventHandleAbortsIf"></a>


<pre><code><b>schema</b> <a href="account.md#0x1_account_NewEventHandleAbortsIf">NewEventHandleAbortsIf</a> {
    <a href="account.md#0x1_account">account</a>: &<a href="">signer</a>;
    <b>let</b> addr = <a href="_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> <a href="account.md#0x1_account">account</a> = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(addr);
    <b>aborts_if</b> <a href="account.md#0x1_account">account</a>.guid_creation_num + 1 &gt; <a href="account.md#0x1_account_MAX_U64">MAX_U64</a>;
    <b>aborts_if</b> <a href="account.md#0x1_account">account</a>.guid_creation_num + 1 &gt;= <a href="account.md#0x1_account_MAX_GUID_CREATION_NUM">MAX_GUID_CREATION_NUM</a>;
}
</code></pre>



<a name="@Specification_1_register_coin"></a>

### Function `register_coin`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="account.md#0x1_account_register_coin">register_coin</a>&lt;CoinType&gt;(account_addr: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
<b>aborts_if</b> !<a href="_spec_is_struct">type_info::spec_is_struct</a>&lt;CoinType&gt;();
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(account_addr);
</code></pre>



<a name="@Specification_1_create_signer_with_capability"></a>

### Function `create_signer_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_create_signer_with_capability">create_signer_with_capability</a>(capability: &<a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>): <a href="">signer</a>
</code></pre>




<pre><code><b>let</b> addr = capability.<a href="account.md#0x1_account">account</a>;
<b>ensures</b> <a href="_address_of">signer::address_of</a>(result) == addr;
</code></pre>




<a name="0x1_account_CreateResourceAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="account.md#0x1_account_CreateResourceAccountAbortsIf">CreateResourceAccountAbortsIf</a> {
    resource_addr: <b>address</b>;
    <b>let</b> <a href="account.md#0x1_account">account</a> = <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(resource_addr);
    <b>aborts_if</b> len(<a href="account.md#0x1_account">account</a>.signer_capability_offer.for.vec) != 0;
    <b>aborts_if</b> <a href="account.md#0x1_account">account</a>.sequence_number != 0;
}
</code></pre>



<a name="@Specification_1_verify_signed_message"></a>

### Function `verify_signed_message`


<pre><code><b>public</b> <b>fun</b> <a href="account.md#0x1_account_verify_signed_message">verify_signed_message</a>&lt;T: drop&gt;(<a href="account.md#0x1_account">account</a>: <b>address</b>, account_scheme: u8, account_public_key: <a href="">vector</a>&lt;u8&gt;, signed_message_bytes: <a href="">vector</a>&lt;u8&gt;, message: T)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>modifies</b> <b>global</b>&lt;<a href="account.md#0x1_account_Account">Account</a>&gt;(<a href="account.md#0x1_account">account</a>);
</code></pre>


[move-book]: https://aptos.dev/guides/move-guides/book/SUMMARY
