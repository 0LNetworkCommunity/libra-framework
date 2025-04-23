# Libra Framework Entry Functions

This document lists all entry functions in the Libra Framework codebase. Entry functions can be called directly by end users with a transaction signature.

## Core Framework Modules

### account.move

- `rotate_authentication_key(account: &signer, from_scheme: u8, from_public_key_bytes: vector<u8>, to_scheme: u8, to_public_key_bytes: vector<u8>, cap_rotate_key: vector<u8>, cap_update_table: vector<u8>)`
  - Rotates the authentication key for an account

- `rotate_authentication_key_with_rotation_capability(account_with_cap: &signer, rotate_key_cap: vector<u8>, new_scheme: u8, new_public_key_bytes: vector<u8>, cap_update_table: vector<u8>)`
  - Rotates authentication key using rotation capability

- `offer_rotation_capability(account: &signer, rotation_capability_sig_bytes: vector<u8>, account_scheme: u8, account_public_key_bytes: vector<u8>, recipient_address: address)`
  - Offers rotation capability to another account

- `revoke_rotation_capability(account: &signer, to_be_revoked_address: address)`
  - Revokes rotation capability from a specific address

- `revoke_any_rotation_capability(account: &signer)`
  - Revokes any rotation capability

- `offer_signer_capability(account: &signer, signer_capability_sig_bytes: vector<u8>, account_scheme: u8, account_public_key_bytes: vector<u8>, recipient_address: address)`
  - Offers signer capability to another account

- `revoke_signer_capability(account: &signer, to_be_revoked_address: address)`
  - Revokes signer capability from a specific address

- `revoke_any_signer_capability(account: &signer)`
  - Revokes any signer capability

- `create_account_from_ed25519_public_key(pk_bytes: vector<u8>): signer`
  - Creates an account from ED25519 public key

### fungible_asset.move

- `transfer<T: key>(from: &signer, to: address, amount: u64)`
  - Transfers fungible assets between accounts

### diem_governance.move

- `create_proposal_v2(proposer: &signer, execution_hash: vector<u8>, metadata_location: vector<u8>, metadata_hash: vector<u8>, is_multi_step_proposal: bool)`
  - Creates a new governance proposal

- `ol_create_proposal_v2(proposer: &signer, execution_hash: vector<u8>, metadata_location: vector<u8>, metadata_hash: vector<u8>, is_multi_step_proposal: bool)`
  - 0L specific variant for creating proposals

- `ol_vote(voter: &signer, proposal_id: u64, should_pass: bool)`
  - 0L specific variant for voting on proposals

- `vote(voter: &signer, proposal_id: u64, should_pass: bool)`
  - Votes on a governance proposal

- `add_approved_script_hash_script(proposal_id: u64)`
  - Adds an approved script hash

- `assert_can_resolve(proposal_id: u64)`
  - Verifies if a proposal can be resolved

- `trigger_epoch(_sig: &signer)`
  - Triggers a new epoch

### multisig_account.move

- `create_with_existing_account(creator: &signer, owners: vector<address>, num_signatures_required: u64)`
  - Creates a multisig account with an existing account

- `create(creator: &signer, owners: vector<address>, num_signatures_required: u64): address`
  - Creates a new multisig account

- `create_with_owners(creator: &signer, owners: vector<address>, num_signatures_required: u64): address`
  - Creates a multisig account with specified owners

- `migrate_with_owners(creator: &signer, multisig_address: address, new_owners: vector<address>, num_signatures_required: u64)`
  - Migrates a multisig account to new owners

- `create_transaction(owner: &signer, multisig_address: address, target_function: String, args: vector<vector<u8>>)`
  - Creates a new transaction for multisig approval

- `create_transaction_with_hash(owner: &signer, multisig_address: address, metadata_hash: vector<u8>)`
  - Creates a transaction with a specific hash

- `approve_transaction(owner: &signer, multisig_address: address, sequence_number: u64)`
  - Approves a pending multisig transaction

- `reject_transaction(owner: &signer, multisig_address: address, sequence_number: u64)`
  - Rejects a pending multisig transaction

- `vote_transanction(owner: &signer, multisig_address: address, sequence_number: u64, approve: bool)`
  - Votes on a multisig transaction

- `execute_rejected_transaction(owner: &signer, multisig_address: address, sequence_number: u64)`
  - Executes a rejected transaction

## OL Sources Modules

### community_wallet_init.move

- `init_community(sig: &signer, ...)`
  - Initializes a community wallet

- `propose_offer(sig: &signer, new_signers: vector<address>, num_signers: u64)`
  - Proposes to offer new signers to a community wallet

- `finalize_and_cage(sig: &signer, num_signers: u64)`
  - Finalizes and cages a community wallet

- `change_signer_community_multisig(sig: &signer, ...)`
  - Changes signers in a community multisig wallet

### donor_voice_txs.move

- `propose_payment_tx(donor: &signer, multisig_address: address, ...)`
  - Proposes a payment transaction

- `propose_veto_tx(donor: &signer, multisig_address: address, id: u64)`
  - Proposes a veto transaction

- `vote_veto_tx(donor: &signer, multisig_address: address, id: u64)`
  - Votes on a veto transaction

- `propose_advance_tx(donor: &signer, multisig_address: address, ...)`
  - Proposes an advance transaction

- `vote_reauth_tx(donor: &signer, multisig_address: address)`
  - Votes on a reauthorization transaction

- `propose_liquidate_tx(donor: &signer, multisig_address: address)`
  - Proposes a liquidation transaction

- `vote_liquidation_tx(donor: &signer, multisig_address: address)`
  - Votes on a liquidation transaction

### multi_action.move

- `claim_offer(sig: &signer, multisig_address: address)`
  - Claims an offer for a multisig account

- `finalize_and_cage_deprecated(sig: &signer, initial_authorities: vector<address>, num_signers: u64)`
  - Finalizes and cages a multisig account (deprecated)

### proof_of_fee.move

- `init_bidding(sender: &signer)`
  - Initializes bidding process

- `pof_update_bid(sender: &signer, bid: u64, epoch_expiry: u64)`
  - Updates a Proof of Fee bid

- `pof_update_bid_net_reward(sender: &signer, net_reward: u64, ...)`
  - Updates a Proof of Fee bid with net reward

- `pof_retract_bid(sender: signer)`
  - Retracts a Proof of Fee bid

- `thermostat_unit_happy(vm: signer)`
  - Thermostat unit test function

### community_wallet_advance.move

- `maybe_deauthorize(user: &signer, dv_account: address)`
  - Potentially deauthorizes a community wallet advance

### ol_account.move

- `create_account(root: &signer, auth_key: address)`
  - Creates a new account

- `batch_transfer(source: &signer, recipients: ...)`
  - Transfers funds to multiple recipients in a batch

- `transfer(sender: &signer, to: address, amount: u64)`
  - Transfers funds from sender to recipient

- `set_allow_direct_coin_transfers(account: &signer, allow: bool)`
  - Sets whether direct coin transfers are allowed for an account

## Note

This document lists entry functions that are publicly callable by end users in the Libra Framework's production code. Test-only functions have been excluded.

The entry functions are organized by their module location and include a brief description of their purpose. For detailed parameter lists and implementation details, please refer to the source code of each module.
